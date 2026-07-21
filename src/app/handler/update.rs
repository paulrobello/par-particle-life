//! Main update loop for the application.

use std::time::Instant;

use super::AppHandler;

impl AppHandler {
    pub(crate) fn update(&mut self) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_frame).as_secs_f32();
        self.last_frame = now;

        self.update_fps(now, dt);

        let dt_capped = dt.min(1.0 / 30.0) * self.app.sim_config.time_scale; // Cap dt, apply time scale

        // Increment frame counter for GPU noise seeding
        self.app.sim_config.frame_counter = self.app.sim_config.frame_counter.wrapping_add(1);

        if (self.app.running || self.step_requested) && self.app.update_matrix_variation(dt_capped)
        {
            self.sync_interaction_matrix();
        }

        // Process brush tools (Draw/Erase modify particles)
        self.process_brush_tools();

        self.process_pending_syncs(dt_capped);

        if self.app.running || self.step_requested {
            // GPU compute physics
            self.run_gpu_compute(dt_capped);
            self.step_requested = false;
        }

        self.record_metrics(now);
    }

    /// Track the per-frame and per-second FPS counters.
    ///
    /// `fps_ema` is an exponential moving average updated every frame for a
    /// smooth HUD reading; `fps` is the raw frames-per-second computed once
    /// per wall-clock second.
    fn update_fps(&mut self, now: Instant, dt: f32) {
        let inst_fps = if dt > 0.0 {
            1.0 / dt
        } else {
            self.fps_ema.max(self.fps)
        };
        if self.fps_ema == 0.0 {
            self.fps_ema = inst_fps;
        } else {
            // Heavier smoothing to dampen spikes.
            self.fps_ema = 0.1 * inst_fps + 0.9 * self.fps_ema;
        }

        self.frame_count += 1;
        let fps_elapsed = now.duration_since(self.last_fps_time).as_secs_f32();
        if fps_elapsed >= 1.0 {
            self.fps = self.frame_count as f32 / fps_elapsed;
            self.frame_count = 0;
            self.last_fps_time = now;
        }
    }

    /// Apply pending buffer syncs and push the new SimParams uniform.
    ///
    /// Particle and spatial-hash buffer rebuilds are deferred until the start
    /// of the next frame (flags set by brush tools / cell-size changes); this
    /// is also where the per-frame `update_params` write happens so the GPU
    /// sees the user's latest slider values exactly once per frame.
    fn process_pending_syncs(&mut self, dt_capped: f32) {
        if self.needs_sync {
            self.sync_buffers();
            self.needs_sync = false;
        }

        // Sync spatial hash buffers if cell size changed (separate from particle sync)
        if self.needs_sync_spatial_buffers {
            self.sync_spatial_buffers();
            self.needs_sync_spatial_buffers = false;
        }

        // Update params for UI changes (only once per frame)
        if let Some(gpu_state_ref) = self.gpu.as_ref() {
            // Immutable borrow for update_params
            gpu_state_ref.buffers.update_params(
                &gpu_state_ref.context.queue,
                &self.app.sim_config,
                dt_capped,
            );
        }
    }

    /// Periodic metrics logging and dynamic spatial-hash cell-size tuning.
    ///
    /// Gated behind `PAR_DEBUG_METRICS=1`: the readback path
    /// (`spatial_buffers.read_bin_counts`) copies the entire bin-counts
    /// buffer to a fresh staging buffer and calls `device.poll(wait)`,
    /// producing a visible once-every-10-seconds hitch even on fast
    /// displays. Off by default; enable when profiling density or
    /// debugging the spatial-hash auto-tune heuristic.
    fn record_metrics(&mut self, now: Instant) {
        if !crate::app::gpu_state::metrics_debug_enabled()
            || now.duration_since(self.last_log_time).as_secs_f32() < 10.0
        {
            return;
        }

        let mut density_info = String::from("Density: N/A");
        let mut timings_info = String::from("Timings: N/A");

        // Access self.gpu fresh here after run_gpu_compute might have modified it
        if let Some(gpu_state) = self.gpu.as_mut() {
            // Collect GPU timings
            if !gpu_state.gpu_pass_ms.is_empty() {
                let timings: Vec<String> = gpu_state
                    .gpu_pass_ms
                    .iter()
                    .map(|(label, ms)| format!("{}: {:.2}ms", label, ms))
                    .collect();
                timings_info = format!("Timings: [{}]", timings.join(", "));
            }

            // Read bin counts (blocking!)
            let use_a = gpu_state.spatial_buffers.current_offset_buffer == 0;
            let offsets = gpu_state.spatial_buffers.read_bin_counts(
                &gpu_state.context.device,
                &gpu_state.context.queue,
                use_a,
            );

            if offsets.len() >= 2 {
                let mut max_count = 0u32;
                let mut filled_bins = 0;
                let mut total_particles_counted = 0u32;

                for i in 0..(offsets.len() - 1) {
                    let count = offsets[i + 1].saturating_sub(offsets[i]);
                    if count > 0 {
                        filled_bins += 1;
                        total_particles_counted += count;
                        if count > max_count {
                            max_count = count;
                        }
                    }
                }

                density_info = format!(
                    "Max Bin: {}, Avg Bin: {:.1}, Filled: {}/{}",
                    max_count,
                    total_particles_counted as f32 / filled_bins as f32,
                    filled_bins,
                    offsets.len() - 1
                );

                // Dynamic spatial hash cell size adjustment
                let current_cell_size = self.app.sim_config.spatial_hash_cell_size;
                let max_allowed_density = self.app.sim_config.max_bin_density;
                // Cell size can't go below max_radius - GPU will clamp it anyway
                let min_cell_size = self.app.radius_matrix.max_interaction_radius().max(20.0);

                // If max_count is significantly above target, reduce cell size
                if (max_count as f32) > max_allowed_density * 2.0 {
                    let new_cell_size = (current_cell_size * 0.8).max(min_cell_size);
                    if new_cell_size < current_cell_size {
                        log::info!(
                            "Reducing cell size from {} to {} due to high density (Max Bin: {}, min allowed: {})",
                            current_cell_size,
                            new_cell_size,
                            max_count,
                            min_cell_size
                        );
                        self.app.sim_config.spatial_hash_cell_size = new_cell_size;
                        self.app.config.render_spatial_hash_cell_size = new_cell_size;
                        self.needs_sync_spatial_buffers = true;
                    }
                } else if (max_count as f32) < max_allowed_density * 0.5
                    && current_cell_size < 100.0
                {
                    // Optionally increase cell size if density is very low to reduce overhead
                    let new_cell_size = (current_cell_size * 1.1).min(100.0);
                    if new_cell_size > current_cell_size {
                        log::info!(
                            "Increasing cell size from {} to {} due to low density (Max Bin: {})",
                            current_cell_size,
                            new_cell_size,
                            max_count
                        );
                        self.app.sim_config.spatial_hash_cell_size = new_cell_size;
                        self.app.config.render_spatial_hash_cell_size = new_cell_size;
                        self.needs_sync_spatial_buffers = true;
                    }
                }
            }
        }

        log::info!(
            "Metrics: FPS={:.1}, EMA={:.1} | {} | {}",
            self.fps,
            self.fps_ema,
            timings_info,
            density_info
        );
        self.last_log_time = now;
    }
}

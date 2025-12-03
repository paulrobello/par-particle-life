//! Main update loop for the application.

use std::time::Instant;

use super::AppHandler;

impl AppHandler {
    pub(crate) fn update(&mut self) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_frame).as_secs_f32();
        self.last_frame = now;

        // Exponential moving average for smoother HUD reading.
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

        // Update FPS counter
        self.frame_count += 1;
        let fps_elapsed = now.duration_since(self.last_fps_time).as_secs_f32();
        if fps_elapsed >= 1.0 {
            self.fps = self.frame_count as f32 / fps_elapsed;
            self.frame_count = 0;
            self.last_fps_time = now;
        }

        let dt_capped = dt.min(1.0 / 30.0); // Cap dt to avoid instability

        // Spatial hash is always enabled; enforce even if a preset/file had it off
        self.app.sim_config.use_spatial_hash = true;

        // Process brush tools (Draw/Erase modify particles)
        self.process_brush_tools();

        // Sync GPU buffers if particles were modified
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

        if self.app.running {
            // GPU compute physics
            self.run_gpu_compute(dt_capped);
        }

        // --- Start of Logging and Dynamic Adjustment Block (Moved to End) ---
        // Periodic metrics logging (every 10 seconds)
        if now.duration_since(self.last_log_time).as_secs_f32() >= 10.0 {
            let mut density_info = String::from("Density: N/A");
            let mut timings_info = String::from("Timings: N/A");

            // Access self.gpu fresh here after run_gpu_compute might have modified it
            if let Some(gpu_state) = self.gpu.as_mut() {
                // Use as_mut() to get mutable device/queue for read_bin_counts
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
        // --- End of Logging and Dynamic Adjustment Block ---
    }
}

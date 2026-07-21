//! GPU compute operations for particle physics simulation.

use super::AppHandler;
use crate::app::BrushTool;
use crate::app::gpu_state::GpuState;
use crate::simulation::SimulationConfig;

impl AppHandler {
    pub(crate) fn run_gpu_compute(&mut self, _dt: f32) {
        let Some(gpu) = &mut self.gpu else { return };

        // Params already updated in update() - no need to duplicate

        // Calculate workgroup count (256 threads per workgroup for better Apple Silicon performance)
        let workgroup_count = gpu.buffers.num_particles.div_ceil(256);

        // Always update brush params (advance shader checks is_active flag)
        gpu.brush_pipelines.update_brush(
            &gpu.context.queue,
            &self.brush,
            gpu.buffers.num_particles,
        );

        if self.brush.is_active && matches!(self.brush.tool, BrushTool::Attract | BrushTool::Repel)
        {
            log::debug!(
                "Brush active: tool={:?}, pos=({:.1}, {:.1}), force={:.1}, radius={:.1}",
                self.brush.tool,
                self.brush.position.x,
                self.brush.position.y,
                self.brush.get_force(),
                self.brush.radius
            );
        }

        // Run compute passes on the shared encoder (no individual submits).
        // Compute reads from current_particles(), writes to next_particles().
        // Brush force is now integrated into the advance shader.
        if self.app.sim_config.use_spatial_hash {
            // Spatial hash optimized path - uses separate submissions for barrier correctness.
            // The spatial hash requires transitioning buffers between atomic and non-atomic access,
            // which needs explicit barriers via separate encoder submissions.
            let max_radius = self.app.radius_matrix.max_interaction_radius();
            Self::run_gpu_compute_spatial_with_barriers(
                gpu,
                &self.app.sim_config,
                workgroup_count,
                max_radius,
            );
        } else {
            // Brute force O(n²) path - single encoder, no blocking wait
            let mut encoder = gpu.context.create_encoder("GPU Compute Encoder");
            Self::run_gpu_compute_brute_force_on_encoder(&mut encoder, gpu, workgroup_count);
            gpu.context.submit(encoder.finish());
        }

        // Create render bind groups pointing to next_particles() (the OUTPUT of compute).
        // Compute read from current, wrote to next - so render needs to use next.
        gpu.render_bind_group = gpu.render.create_render_bind_group(
            &gpu.context.device,
            gpu.buffers.next_pos_type(),
            &gpu.buffers,
        );
        gpu.glow_bind_group = gpu.render.create_glow_bind_group(
            &gpu.context.device,
            gpu.buffers.next_pos_type(),
            &gpu.buffers,
        );

        // Swap so next frame's compute reads from what we just rendered (the computed output)
        gpu.buffers.swap_buffers();
    }

    /// Run GPU compute using brute force O(n²) algorithm on a shared encoder.
    /// Reads from current_particles, writes to next_particles.
    fn run_gpu_compute_brute_force_on_encoder(
        encoder: &mut wgpu::CommandEncoder,
        gpu: &mut GpuState,
        workgroup_count: u32,
    ) {
        // Read from current (input), write to next (output)
        let pos_in = gpu.buffers.current_pos_type();
        let vel_in = gpu.buffers.current_velocities();
        let pos_out = gpu.buffers.next_pos_type();
        let vel_out = gpu.buffers.next_velocities();

        // Create bind groups for compute passes
        let force_bind_group = gpu.compute.create_force_bind_group(
            &gpu.context.device,
            pos_in,  // Read positions
            vel_in,  // Read velocities (accumulate forces)
            vel_out, // Write new velocities
            &gpu.buffers,
        );

        let advance_bind_group = gpu.compute.create_advance_bind_group(
            &gpu.context.device,
            pos_out, // Write new positions
            vel_out, // Read/Write velocities
            &gpu.buffers.params,
            &gpu.brush_pipelines.brush_buffer,
            &gpu.buffers.obstacles,
            &gpu.buffers.velocity_scratch,
        );

        encoder.copy_buffer_to_buffer(
            vel_in,
            0,
            &gpu.buffers.velocity_scratch,
            0,
            gpu.buffers.velocity_buffer_size(),
        );

        // Force computation pass
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Force Compute Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&gpu.compute.force_pipeline);
            compute_pass.set_bind_group(0, &force_bind_group, &[]);
            compute_pass.dispatch_workgroups(workgroup_count, 1, 1);
        }

        // Advance pass (integrate velocities, apply boundaries)
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Advance Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&gpu.compute.advance_pipeline);
            compute_pass.set_bind_group(0, &advance_bind_group, &[]);
            compute_pass.dispatch_workgroups(workgroup_count, 1, 1);
        }
        // No submit - encoder will be submitted by caller
    }

    /// Run GPU compute using spatial hashing - optimized single-encoder version.
    /// All passes are submitted in a single encoder for maximum GPU throughput.
    /// wgpu automatically handles memory barriers between compute passes.
    fn run_gpu_compute_spatial_with_barriers(
        gpu: &mut GpuState,
        sim_config: &SimulationConfig,
        particle_workgroups: u32,
        max_radius: f32,
    ) {
        // Update spatial params
        gpu.spatial_buffers
            .update_params(&gpu.context.queue, sim_config, max_radius);

        // Build or reuse bind groups for the current grid and buffers
        gpu.spatial_bind_groups.ensure(
            &gpu.context.device,
            &gpu.buffers,
            &mut gpu.spatial_buffers,
            &gpu.spatial_pipelines,
        );

        let total_bins = gpu.spatial_buffers.total_bins_with_end();
        let bin_workgroups = total_bins.div_ceil(256);
        let num_passes = gpu.spatial_bind_groups.pass_count;
        let offsets_in_a = gpu.spatial_bind_groups.offsets_in_a;

        log::debug!(
            "Spatial hash: {} bins, {} workgroups, {} prefix sum passes",
            total_bins,
            bin_workgroups,
            num_passes
        );

        // Create single encoder for all passes
        let mut encoder = gpu.context.create_encoder("Spatial Hash Compute");

        // ============ PHASE 1: Clear + Count ============
        // Clear bin counts (buffer A)
        let clear_bind_group = gpu.spatial_bind_groups.clear(true);
        let mut timestamp_labels: Vec<String> = Vec::new();
        let mut query_index: u32 = 0;

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Bin Clear Pass"),
                timestamp_writes: None,
            });
            if gpu.timestamps_supported
                && let Some(qs) = gpu.timestamp_query_set.as_ref()
            {
                pass.write_timestamp(qs, query_index);
                query_index += 1;
            }
            pass.set_pipeline(&gpu.spatial_pipelines.clear_pipeline);
            pass.set_bind_group(0, clear_bind_group, &[]);
            pass.dispatch_workgroups(bin_workgroups, 1, 1);
            if gpu.timestamps_supported
                && let Some(qs) = gpu.timestamp_query_set.as_ref()
            {
                pass.write_timestamp(qs, query_index);
                query_index += 1;
            }
        }
        timestamp_labels.push("clear".to_string());

        // Count particles per bin
        let count_bind_group = gpu.spatial_bind_groups.count_for_current(&gpu.buffers);

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Bin Count Pass"),
                timestamp_writes: None,
            });
            if let Some(qs) = gpu.timestamp_query_set.as_ref() {
                pass.write_timestamp(qs, query_index);
                query_index += 1;
            }
            pass.set_pipeline(&gpu.spatial_pipelines.count_pipeline);
            pass.set_bind_group(0, count_bind_group, &[]);
            pass.dispatch_workgroups(particle_workgroups, 1, 1);
            if let Some(qs) = gpu.timestamp_query_set.as_ref() {
                pass.write_timestamp(qs, query_index);
                query_index += 1;
            }
        }
        timestamp_labels.push("count".to_string());

        // ============ PHASE 2: Prefix Sum ============
        for (idx, bind_group) in gpu.spatial_bind_groups.prefix_groups().iter().enumerate() {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Prefix Sum Pass"),
                timestamp_writes: None,
            });
            if let Some(qs) = gpu.timestamp_query_set.as_ref() {
                pass.write_timestamp(qs, query_index);
                query_index += 1;
            }
            pass.set_pipeline(&gpu.spatial_pipelines.prefix_sum_pipeline);
            pass.set_bind_group(0, bind_group, &[]);
            pass.dispatch_workgroups(bin_workgroups, 1, 1);
            if let Some(qs) = gpu.timestamp_query_set.as_ref() {
                pass.write_timestamp(qs, query_index);
                query_index += 1;
            }
            timestamp_labels.push(format!("prefix {}", idx));
        }

        // Track which buffer has the final prefix sum result
        gpu.spatial_buffers.current_offset_buffer = if offsets_in_a { 0 } else { 1 };

        // ============ PHASE 3: Clear for Sort + Sort ============
        // Clear the OTHER buffer for sort atomic counters
        let clear_for_sort_bind_group = gpu.spatial_bind_groups.clear(!offsets_in_a);

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Pre-Sort Clear Pass"),
                timestamp_writes: None,
            });
            if let Some(qs) = gpu.timestamp_query_set.as_ref() {
                pass.write_timestamp(qs, query_index);
                query_index += 1;
            }
            pass.set_pipeline(&gpu.spatial_pipelines.clear_pipeline);
            pass.set_bind_group(0, clear_for_sort_bind_group, &[]);
            pass.dispatch_workgroups(bin_workgroups, 1, 1);
            if let Some(qs) = gpu.timestamp_query_set.as_ref() {
                pass.write_timestamp(qs, query_index);
                query_index += 1;
            }
        }
        timestamp_labels.push("clear_sort".to_string());

        // Sort particles by bin
        let sort_bind_group = gpu.spatial_bind_groups.sort_for_current(&gpu.buffers);

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Bin Sort Pass"),
                timestamp_writes: None,
            });
            if let Some(qs) = gpu.timestamp_query_set.as_ref() {
                pass.write_timestamp(qs, query_index);
                query_index += 1;
            }
            pass.set_pipeline(&gpu.spatial_pipelines.sort_pipeline);
            pass.set_bind_group(0, sort_bind_group, &[]);
            pass.dispatch_workgroups(particle_workgroups, 1, 1);
            if let Some(qs) = gpu.timestamp_query_set.as_ref() {
                pass.write_timestamp(qs, query_index);
                query_index += 1;
            }
        }
        timestamp_labels.push("sort".to_string());

        // ============ PHASE 4: Forces + Advance ============
        let forces_bind_group = gpu.spatial_bind_groups.forces_for_current(&gpu.buffers);

        let pos_out = gpu.buffers.next_pos_type();
        let vel_out = gpu.buffers.next_velocities();

        let advance_bind_group = gpu.compute.create_advance_bind_group(
            &gpu.context.device,
            pos_out, // In-place update
            vel_out, // In-place update (after force pass wrote to it)
            &gpu.buffers.params,
            &gpu.brush_pipelines.brush_buffer,
            &gpu.buffers.obstacles,
            &gpu.buffers.velocity_scratch,
        );

        encoder.copy_buffer_to_buffer(
            vel_out,
            0,
            &gpu.buffers.velocity_scratch,
            0,
            gpu.buffers.velocity_buffer_size(),
        );

        // Binned force computation
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Binned Forces Pass"),
                timestamp_writes: None,
            });
            if let Some(qs) = gpu.timestamp_query_set.as_ref() {
                pass.write_timestamp(qs, query_index);
                query_index += 1;
            }
            pass.set_pipeline(&gpu.spatial_pipelines.forces_pipeline);
            pass.set_bind_group(0, forces_bind_group, &[]);
            pass.dispatch_workgroups(particle_workgroups, 1, 1);
            if let Some(qs) = gpu.timestamp_query_set.as_ref() {
                pass.write_timestamp(qs, query_index);
                query_index += 1;
            }
        }
        timestamp_labels.push("forces".to_string());

        // Advance pass
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Advance Pass"),
                timestamp_writes: None,
            });
            if gpu.timestamps_supported
                && let Some(qs) = gpu.timestamp_query_set.as_ref()
            {
                pass.write_timestamp(qs, query_index);
                query_index += 1;
            }
            pass.set_pipeline(&gpu.compute.advance_pipeline);
            pass.set_bind_group(0, &advance_bind_group, &[]);
            pass.dispatch_workgroups(particle_workgroups, 1, 1);
            if gpu.timestamps_supported
                && let Some(qs) = gpu.timestamp_query_set.as_ref()
            {
                pass.write_timestamp(qs, query_index);
                query_index += 1;
            }
        }
        timestamp_labels.push("advance".to_string());

        if gpu.timestamps_supported {
            if let (Some(qs), Some(resolve)) = (
                gpu.timestamp_query_set.as_ref(),
                gpu.timestamp_resolve_buffer.as_ref(),
            ) {
                gpu.timestamp_last_count = query_index;
                gpu.timestamp_labels = timestamp_labels;
                if query_index > 0 {
                    encoder.resolve_query_set(qs, 0..query_index, resolve, 0);
                }
            }
        } else {
            gpu.timestamp_last_count = 0;
            gpu.timestamp_labels.clear();
        }

        // Submit all passes in one batch - no blocking wait
        gpu.context.submit(encoder.finish());

        // Read back GPU timings (best-effort; no-op if timestamps unsupported).
        // Gated behind PAR_PROFILE_GPU: the readback calls device.poll(wait),
        // which serializes CPU and GPU every frame. Off by default.
        if gpu.timestamps_supported
            && gpu.timestamp_last_count > 0
            && crate::app::gpu_state::gpu_profiling_enabled()
        {
            gpu.fetch_gpu_timings();
        }
    }
}

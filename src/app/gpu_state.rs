//! GPU state management for rendering and compute.

use bytemuck::cast_slice;

use crate::renderer::gpu::{
    BrushPipelines, ComputePipelines, GpuContext, RenderPipelines, SimulationBuffers,
    SpatialHashBuffers, SpatialHashPipelines,
};

// Maximum prefix-sum passes the spatial hash can issue (matches buffer allocation).
pub(crate) const MAX_PREFIX_PASSES: u32 = 32;
// Clear + count + prefix passes + clear-sort + sort + forces + advance (each with start/end).
pub(crate) const MAX_TIMESTAMP_QUERIES: u32 = (MAX_PREFIX_PASSES + 6) * 2;

/// Cached bind groups for the spatial hash compute passes.
///
/// These groups are rebuilt when buffer handles change or the number of
/// prefix-sum passes changes (driven by the current spatial grid size).
pub(crate) struct SpatialBindGroupCache {
    /// Number of prefix-sum passes the cache was built for.
    pub(crate) pass_count: u32,
    /// True when the final prefix-sum result lives in buffer A.
    pub(crate) offsets_in_a: bool,
    clear_a: Option<wgpu::BindGroup>,
    clear_b: Option<wgpu::BindGroup>,
    prefix: Vec<wgpu::BindGroup>,
    count_from_a: Option<wgpu::BindGroup>,
    count_from_b: Option<wgpu::BindGroup>,
    sort_from_a: Option<wgpu::BindGroup>,
    sort_from_b: Option<wgpu::BindGroup>,
    forces_into_a: Option<wgpu::BindGroup>,
    forces_into_b: Option<wgpu::BindGroup>,
}

impl SpatialBindGroupCache {
    pub(crate) fn new() -> Self {
        Self {
            pass_count: 0,
            offsets_in_a: true,
            clear_a: None,
            clear_b: None,
            prefix: Vec::new(),
            count_from_a: None,
            count_from_b: None,
            sort_from_a: None,
            sort_from_b: None,
            forces_into_a: None,
            forces_into_b: None,
        }
    }

    /// Drop cached bind groups so they will be rebuilt on next use.
    pub(crate) fn invalidate(&mut self) {
        self.pass_count = 0;
        self.prefix.clear();
        self.clear_a = None;
        self.clear_b = None;
        self.count_from_a = None;
        self.count_from_b = None;
        self.sort_from_a = None;
        self.sort_from_b = None;
        self.forces_into_a = None;
        self.forces_into_b = None;
    }

    /// Ensure cached bind groups match the current buffers and grid size.
    pub(crate) fn ensure(
        &mut self,
        device: &wgpu::Device,
        sim_buffers: &SimulationBuffers,
        spatial_buffers: &mut SpatialHashBuffers,
        spatial_pipelines: &SpatialHashPipelines,
    ) {
        let pass_count = spatial_buffers.prefix_sum_passes();

        let needs_rebuild = self.clear_a.is_none()
            || self.clear_b.is_none()
            || self.count_from_a.is_none()
            || self.count_from_b.is_none()
            || self.sort_from_a.is_none()
            || self.sort_from_b.is_none()
            || self.forces_into_a.is_none()
            || self.forces_into_b.is_none()
            || self.pass_count != pass_count
            || self.prefix.len() as u32 != pass_count;

        if !needs_rebuild {
            return;
        }

        self.pass_count = pass_count;
        self.offsets_in_a = pass_count.is_multiple_of(2);
        spatial_buffers.current_offset_buffer = if self.offsets_in_a { 0 } else { 1 };

        self.clear_a =
            Some(spatial_pipelines.create_clear_bind_group(device, spatial_buffers, true));
        self.clear_b =
            Some(spatial_pipelines.create_clear_bind_group(device, spatial_buffers, false));

        // Count uses bin_counts_a; we only need to vary the particle input buffer.
        self.count_from_a = Some(spatial_pipelines.create_count_bind_group(
            device,
            &sim_buffers.pos_type[0],
            spatial_buffers,
        ));
        self.count_from_b = Some(spatial_pipelines.create_count_bind_group(
            device,
            &sim_buffers.pos_type[1],
            spatial_buffers,
        ));

        // Prefix-sum bind groups alternate between A and B each pass.
        self.prefix.clear();
        let mut src_is_a = true;
        for pass_idx in 0..pass_count {
            let (source, dest) = if src_is_a {
                (&spatial_buffers.bin_counts_a, &spatial_buffers.bin_counts_b)
            } else {
                (&spatial_buffers.bin_counts_b, &spatial_buffers.bin_counts_a)
            };

            let step_size = &spatial_buffers.step_size_uniforms[pass_idx as usize];
            self.prefix.push(
                spatial_pipelines.create_prefix_sum_bind_group(device, source, dest, step_size),
            );

            src_is_a = !src_is_a;
        }

        // Offset buffer is where the prefix sum finished; count buffer is the other one.
        let offset_in_a = self.offsets_in_a;
        let count_in_a = !offset_in_a;

        // Sort: Current (Source) -> Next (Dest)
        // sort_from_a: 0 -> 1
        self.sort_from_a = Some(spatial_pipelines.create_sort_bind_group(
            device,
            &sim_buffers.pos_type[0],   // Pos In
            &sim_buffers.pos_type[1],   // Pos Out
            &sim_buffers.velocities[0], // Vel In
            &sim_buffers.velocities[1], // Vel Out
            spatial_buffers,
            offset_in_a,
            count_in_a,
        ));
        // sort_from_b: 1 -> 0
        self.sort_from_b = Some(spatial_pipelines.create_sort_bind_group(
            device,
            &sim_buffers.pos_type[1],   // Pos In
            &sim_buffers.pos_type[0],   // Pos Out
            &sim_buffers.velocities[1], // Vel In
            &sim_buffers.velocities[0], // Vel Out
            spatial_buffers,
            offset_in_a,
            count_in_a,
        ));

        // Forces: Operate on Next (Sorted)
        // forces_into_b (corresponds to sort dest 1)
        self.forces_into_b = Some(spatial_pipelines.create_forces_bind_group(
            device,
            &sim_buffers.velocities[1], // Vel (RW)
            &sim_buffers.pos_type[1],   // Sorted Pos (R)
            sim_buffers,
            spatial_buffers,
        ));
        // forces_into_a (corresponds to sort dest 0)
        self.forces_into_a = Some(spatial_pipelines.create_forces_bind_group(
            device,
            &sim_buffers.velocities[0], // Vel (RW)
            &sim_buffers.pos_type[0],   // Sorted Pos (R)
            sim_buffers,
            spatial_buffers,
        ));
    }

    pub(crate) fn clear(&self, use_buffer_a: bool) -> &wgpu::BindGroup {
        if use_buffer_a {
            self.clear_a.as_ref().expect("clear_a not built")
        } else {
            self.clear_b.as_ref().expect("clear_b not built")
        }
    }

    pub(crate) fn count_for_current(&self, sim_buffers: &SimulationBuffers) -> &wgpu::BindGroup {
        if sim_buffers.current_buffer == 0 {
            self.count_from_a.as_ref().expect("count_from_a not built")
        } else {
            self.count_from_b.as_ref().expect("count_from_b not built")
        }
    }

    pub(crate) fn sort_for_current(&self, sim_buffers: &SimulationBuffers) -> &wgpu::BindGroup {
        if sim_buffers.current_buffer == 0 {
            self.sort_from_a.as_ref().expect("sort_from_a not built")
        } else {
            self.sort_from_b.as_ref().expect("sort_from_b not built")
        }
    }

    pub(crate) fn forces_for_current(&self, sim_buffers: &SimulationBuffers) -> &wgpu::BindGroup {
        if sim_buffers.current_buffer == 0 {
            // Reading buffer 0, writing buffer 1
            self.forces_into_b
                .as_ref()
                .expect("forces_into_b not built")
        } else {
            // Reading buffer 1, writing buffer 0
            self.forces_into_a
                .as_ref()
                .expect("forces_into_a not built")
        }
    }

    pub(crate) fn prefix_groups(&self) -> &[wgpu::BindGroup] {
        &self.prefix
    }
}

/// GPU rendering state including egui.
pub(crate) struct GpuState {
    /// GPU context.
    pub(crate) context: GpuContext,
    /// Simulation buffers.
    pub(crate) buffers: SimulationBuffers,
    /// Compute pipelines.
    pub(crate) compute: ComputePipelines,
    /// Render pipelines.
    pub(crate) render: RenderPipelines,
    /// Spatial hash buffers.
    pub(crate) spatial_buffers: SpatialHashBuffers,
    /// Spatial hash compute pipelines.
    pub(crate) spatial_pipelines: SpatialHashPipelines,
    /// Cached bind groups for spatial hash passes.
    pub(crate) spatial_bind_groups: SpatialBindGroupCache,
    /// Timestamp query set for GPU pass timings (if supported).
    pub(crate) timestamp_query_set: Option<wgpu::QuerySet>,
    /// Buffer to resolve timestamp query results into.
    pub(crate) timestamp_resolve_buffer: Option<wgpu::Buffer>,
    /// Most recent GPU pass durations in milliseconds with labels.
    pub(crate) gpu_pass_ms: Vec<(String, f32)>,
    /// Total GPU frame time from the last measurement.
    pub(crate) gpu_total_ms: f32,
    /// Timestamp period reported by the queue (nanoseconds per tick).
    pub(crate) timestamp_period: f32,
    /// Number of timestamp slots written in the last frame.
    pub(crate) timestamp_last_count: u32,
    /// Labels matching each pass timestamp pair.
    pub(crate) timestamp_labels: Vec<String>,
    /// Whether timestamp queries inside passes are supported and enabled.
    pub(crate) timestamps_supported: bool,
    /// Brush pipelines.
    pub(crate) brush_pipelines: BrushPipelines,
    /// Brush force bind group (for future brush circle rendering).
    pub(crate) _brush_bind_group: wgpu::BindGroup,
    /// Render bind group.
    pub(crate) render_bind_group: wgpu::BindGroup,
    /// Glow render bind group.
    pub(crate) glow_bind_group: wgpu::BindGroup,
    /// Mirror wrap render bind group.
    pub(crate) mirror_bind_group: wgpu::BindGroup,
    /// Infinite wrap render bind group.
    pub(crate) infinite_bind_group: wgpu::BindGroup,
    /// egui context.
    pub(crate) egui_ctx: egui::Context,
    /// egui winit state.
    pub(crate) egui_state: egui_winit::State,
    /// egui wgpu renderer.
    pub(crate) egui_renderer: egui_wgpu::Renderer,
}

impl GpuState {
    /// Read back resolved timestamp queries and compute per-pass durations.
    pub(crate) fn fetch_gpu_timings(&mut self) {
        if self.timestamp_last_count < 2 {
            self.gpu_pass_ms.clear();
            self.gpu_total_ms = 0.0;
            return;
        }

        let Some(buffer) = self.timestamp_resolve_buffer.as_ref() else {
            return;
        };

        let size = (self.timestamp_last_count as u64) * 8;
        let slice = buffer.slice(..size);

        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |res| {
            let _ = tx.send(res);
        });

        // Block just long enough for the query results; this is only for profiling/HUD.
        let _ = self
            .context
            .device
            .poll(wgpu::PollType::wait_indefinitely());
        if rx.recv().is_err() {
            return;
        }

        let data = slice.get_mapped_range();
        let timestamps: &[u64] = cast_slice(&data);

        let mut pass_ms = Vec::with_capacity(self.timestamp_labels.len());
        let mut total_ms = 0.0f32;
        for (i, label) in self.timestamp_labels.iter().enumerate() {
            let start = timestamps[i * 2];
            let end = timestamps[i * 2 + 1];
            let delta_ms = (end.saturating_sub(start) as f32 * self.timestamp_period) / 1_000_000.0;
            total_ms += delta_ms;
            pass_ms.push((label.clone(), delta_ms));
        }

        self.gpu_pass_ms = pass_ms;
        self.gpu_total_ms = total_ms;

        drop(data);
        buffer.unmap();
    }
}

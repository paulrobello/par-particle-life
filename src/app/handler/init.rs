//! GPU initialization for the application handler.

use std::sync::Arc;
use winit::window::{Icon, Window};

use super::AppHandler;
use crate::app::gpu_state::{GpuState, MAX_TIMESTAMP_QUERIES, SpatialBindGroupCache};
use crate::renderer::gpu::{
    BrushPipelines, ComputePipelines, GpuContext, RenderPipelines, SimulationBuffers,
    SpatialHashBuffers, SpatialHashPipelines,
};

impl AppHandler {
    pub(crate) fn init_gpu(&mut self, window: Arc<Window>) {
        // Initialize GPU context using vsync preference from config
        let context = pollster::block_on(GpuContext::new(window.clone(), self.app.config.vsync))
            .expect("Failed to create GPU context");

        // Create simulation buffers
        let colors_rgba = self.app.colors_as_rgba();
        let buffers = SimulationBuffers::new(
            &context.device,
            &self.app.particles,
            &self.app.interaction_matrix,
            &self.app.radius_matrix,
            &colors_rgba,
            &self.app.sim_config,
        );

        // Create pipelines
        let compute = ComputePipelines::new(&context.device);
        let render = RenderPipelines::new(&context.device, context.surface_format());
        let spatial_pipelines = SpatialHashPipelines::new(&context.device);

        // Create spatial hash buffers (cell size clamped to max interaction radius)
        let max_radius = self.app.radius_matrix.max_interaction_radius();
        let spatial_buffers =
            SpatialHashBuffers::new(&context.device, &self.app.sim_config, max_radius);

        // Create brush pipelines
        let brush_pipelines = BrushPipelines::new(&context.device, context.surface_format());
        let brush_bind_group = brush_pipelines.create_force_bind_group(
            &context.device,
            buffers.current_pos_type(),
            buffers.current_velocities(),
        );

        // Create initial render bind groups (will be recreated each frame for GPU compute)
        let render_bind_group =
            render.create_render_bind_group(&context.device, buffers.current_pos_type(), &buffers);
        let glow_bind_group =
            render.create_glow_bind_group(&context.device, buffers.current_pos_type(), &buffers);
        let mirror_bind_group =
            render.create_mirror_bind_group(&context.device, buffers.current_pos_type(), &buffers);
        let infinite_bind_group = render.create_infinite_bind_group(
            &context.device,
            buffers.current_pos_type(),
            &buffers,
        );

        // Update camera and glow for initial world size
        render.update_camera(
            &context.queue,
            self.app.sim_config.world_size.x,
            self.app.sim_config.world_size.y,
            context.surface_config.width as f32,
            context.surface_config.height as f32,
        );

        // Initialize egui
        let egui_ctx = egui::Context::default();

        // Configure egui visuals for dark theme
        let mut visuals = egui::Visuals::dark();
        visuals.window_shadow = egui::epaint::Shadow::NONE;
        visuals.popup_shadow = egui::epaint::Shadow::NONE;
        egui_ctx.set_visuals(visuals);

        let egui_state = egui_winit::State::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
            Some(2 * 1024 * 1024), // 2MB max texture size
        );

        let egui_renderer = egui_wgpu::Renderer::new(
            &context.device,
            context.surface_format(),
            egui_wgpu::RendererOptions::default(),
        );

        // Timestamp queries for GPU profiling (when supported by the device).
        let ts_features =
            wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::TIMESTAMP_QUERY_INSIDE_PASSES;

        let timestamps_supported = context.device.features().contains(ts_features);

        let (timestamp_query_set, timestamp_resolve_buffer) = if timestamps_supported {
            let query_set = context.device.create_query_set(&wgpu::QuerySetDescriptor {
                label: Some("Timestamp Query Set"),
                ty: wgpu::QueryType::Timestamp,
                count: MAX_TIMESTAMP_QUERIES,
            });

            let resolve_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Timestamp Resolve Buffer"),
                size: (MAX_TIMESTAMP_QUERIES as u64) * 8,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });

            (Some(query_set), Some(resolve_buffer))
        } else {
            (None, None)
        };

        self.gpu = Some(GpuState {
            context,
            buffers,
            compute,
            render,
            spatial_buffers,
            spatial_pipelines,
            spatial_bind_groups: SpatialBindGroupCache::new(),
            timestamp_query_set,
            timestamp_resolve_buffer,
            gpu_pass_ms: Vec::new(),
            gpu_total_ms: 0.0,
            timestamp_period: 0.0,
            timestamp_last_count: 0,
            timestamp_labels: Vec::new(),
            timestamps_supported,
            brush_pipelines,
            _brush_bind_group: brush_bind_group,
            render_bind_group,
            glow_bind_group,
            mirror_bind_group,
            infinite_bind_group,
            egui_ctx,
            egui_state,
            egui_renderer,
        });

        if let Some(gpu) = &mut self.gpu {
            gpu.timestamp_period = gpu.context.queue.get_timestamp_period();
            gpu.spatial_bind_groups.ensure(
                &gpu.context.device,
                &gpu.buffers,
                &mut gpu.spatial_buffers,
                &gpu.spatial_pipelines,
            );
        }

        log::info!(
            "Initialized with {} particles, {} types",
            self.app.particles.len(),
            self.app.sim_config.num_types
        );
    }

    pub(super) fn load_window_icon() -> Option<Icon> {
        // Try to load the icon from embedded bytes
        let icon_bytes = include_bytes!("../../../assets/icon.png");

        match image::load_from_memory(icon_bytes) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                let (width, height) = rgba.dimensions();
                let pixels = rgba.into_raw();

                match Icon::from_rgba(pixels, width, height) {
                    Ok(icon) => Some(icon),
                    Err(e) => {
                        log::warn!("Failed to create icon from pixels: {}", e);
                        None
                    }
                }
            }
            Err(e) => {
                log::warn!("Failed to create window icon: {}", e);
                None
            }
        }
    }
}

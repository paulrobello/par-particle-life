//! Compute and render pipelines for brush interaction.
//!
//! This module contains pipelines for brush-based particle interaction,
//! including force application and visual circle indicator rendering.

use wgpu::util::DeviceExt;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, BlendState, Buffer, BufferBindingType, ColorTargetState,
    ColorWrites, ComputePipeline, ComputePipelineDescriptor, Device, FragmentState, FrontFace,
    MultisampleState, PipelineCompilationOptions, PipelineLayoutDescriptor, PolygonMode,
    PrimitiveState, PrimitiveTopology, Queue, RenderPipeline, RenderPipelineDescriptor,
    ShaderModuleDescriptor, ShaderSource, ShaderStages, TextureFormat, VertexState,
};

use super::load_shader;
use crate::renderer::gpu::{
    BrushParamsUniform, BrushRenderUniform, SimulationBuffers, SpawnParamsUniform,
};

/// Compute and render pipelines for brush interaction.
pub struct BrushPipelines {
    /// Compute pipeline for applying brush forces.
    pub force_pipeline: ComputePipeline,
    /// Bind group layout for brush force computation.
    pub force_bind_group_layout: BindGroupLayout,
    /// Brush parameters uniform buffer.
    pub brush_buffer: Buffer,
    /// Render pipeline for brush circle indicator.
    pub circle_pipeline: RenderPipeline,
    /// Bind group layout for brush circle rendering.
    pub circle_bind_group_layout: BindGroupLayout,
    /// Brush render parameters uniform buffer.
    pub render_buffer: Buffer,
    /// Bind group for brush circle rendering.
    pub circle_bind_group: BindGroup,
    /// Compute pipeline for GPU-driven particle spawning.
    pub spawn_pipeline: ComputePipeline,
    /// Bind group layout for GPU-driven particle spawning.
    pub spawn_bind_group_layout: BindGroupLayout,
    /// Spawn parameters uniform buffer.
    pub spawn_buffer: Buffer,
}

impl BrushPipelines {
    /// Create brush pipelines.
    pub fn new(device: &Device, surface_format: TextureFormat) -> Self {
        // Load brush force shader
        let force_shader = load_shader(
            device,
            "Brush Force Shader",
            include_str!("../../../../shaders/brush_force.wgsl"),
        );

        // Create bind group layout for brush force computation
        let force_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Brush Force Bind Group Layout"),
            entries: &[
                // pos_type (storage, read-only)
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // velocities (storage, read-write)
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // brush params (uniform)
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        // Create pipeline layout
        let force_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Brush Force Pipeline Layout"),
            bind_group_layouts: &[Some(&force_bind_group_layout)],
            immediate_size: 0,
        });

        // Create force compute pipeline
        let force_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Brush Force Pipeline"),
            layout: Some(&force_pipeline_layout),
            module: &force_shader,
            entry_point: Some("main"),
            compilation_options: PipelineCompilationOptions::default(),
            cache: None,
        });

        // Create brush buffer with default values
        let default_params = BrushParamsUniform {
            pos_x: 0.0,
            pos_y: 0.0,
            vel_x: 0.0,
            vel_y: 0.0,
            radius: 100.0,
            force: 0.0,
            directional_force: 0.0,
            is_active: 0,
            num_particles: 0,
            target_type: -1,
            _padding: [0; 2],
        };
        let brush_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Brush Params Buffer"),
            contents: bytemuck::bytes_of(&default_params),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // ===== Brush Circle Render Pipeline =====

        // Load brush circle shader
        let circle_shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Brush Circle Shader"),
            source: ShaderSource::Wgsl(
                include_str!("../../../../shaders/brush_circle.wgsl").into(),
            ),
        });

        // Create bind group layout for brush circle rendering
        let circle_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Brush Circle Bind Group Layout"),
                entries: &[
                    // brush render params (uniform)
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        // Create pipeline layout for circle
        let circle_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Brush Circle Pipeline Layout"),
            bind_group_layouts: &[Some(&circle_bind_group_layout)],
            immediate_size: 0,
        });

        // Create circle render pipeline
        let circle_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Brush Circle Pipeline"),
            layout: Some(&circle_pipeline_layout),
            vertex: VertexState {
                module: &circle_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(FragmentState {
                module: &circle_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format: surface_format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        // Create brush render buffer with default values
        let default_render = BrushRenderUniform {
            pos_x: 0.0,
            pos_y: 0.0,
            radius: 100.0,
            color_r: 0.5,
            color_g: 0.5,
            color_b: 0.5,
            color_a: 0.8,
            is_visible: 0,
            world_width: 1000.0,
            world_height: 1000.0,
            camera_zoom: 1.0,
            camera_offset_x: 0.0,
            camera_offset_y: 0.0,
            _padding1: [0.0; 3],
            _padding2: [0.0; 4],
        };
        let render_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Brush Render Buffer"),
            contents: bytemuck::bytes_of(&default_render),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create bind group for circle rendering
        let circle_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Brush Circle Bind Group"),
            layout: &circle_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: render_buffer.as_entire_binding(),
            }],
        });

        // ===== Particle Spawn Pipeline =====

        let spawn_shader = load_shader(
            device,
            "Particle Spawn Shader",
            include_str!("../../../../shaders/particle_spawn.wgsl"),
        );

        let spawn_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Particle Spawn Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let spawn_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Particle Spawn Pipeline Layout"),
            bind_group_layouts: &[Some(&spawn_bind_group_layout)],
            immediate_size: 0,
        });

        let spawn_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Particle Spawn Pipeline"),
            layout: Some(&spawn_pipeline_layout),
            module: &spawn_shader,
            entry_point: Some("main"),
            compilation_options: PipelineCompilationOptions::default(),
            cache: None,
        });

        let default_spawn = SpawnParamsUniform {
            start_index: 0,
            spawn_count: 0,
            capacity_particles: 0,
            num_types: 1,
            pos_x: 0.0,
            pos_y: 0.0,
            radius: 100.0,
            world_width: 1000.0,
            world_height: 1000.0,
            draw_type: -1,
            frame_counter: 0,
            _padding: 0,
        };
        let spawn_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Particle Spawn Params Buffer"),
            contents: bytemuck::bytes_of(&default_spawn),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            force_pipeline,
            force_bind_group_layout,
            brush_buffer,
            circle_pipeline,
            circle_bind_group_layout,
            render_buffer,
            circle_bind_group,
            spawn_pipeline,
            spawn_bind_group_layout,
            spawn_buffer,
        }
    }

    /// Create brush force bind group.
    pub fn create_force_bind_group(
        &self,
        device: &Device,
        pos_type: &Buffer,
        velocities: &Buffer,
    ) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: Some("Brush Force Bind Group"),
            layout: &self.force_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: pos_type.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: velocities.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: self.brush_buffer.as_entire_binding(),
                },
            ],
        })
    }

    /// Update brush parameters for compute.
    pub fn update_brush(&self, queue: &Queue, brush: &crate::app::BrushState, num_particles: u32) {
        let params = BrushParamsUniform::from_brush_state(brush, num_particles);
        queue.write_buffer(&self.brush_buffer, 0, bytemuck::bytes_of(&params));
    }

    /// Update brush render parameters for circle display.
    #[allow(clippy::too_many_arguments)]
    pub fn update_render(
        &self,
        queue: &Queue,
        brush: &crate::app::BrushState,
        world_width: f32,
        world_height: f32,
        camera_zoom: f32,
        camera_offset_x: f32,
        camera_offset_y: f32,
    ) {
        let params = BrushRenderUniform::from_brush_state(
            brush,
            world_width,
            world_height,
            camera_zoom,
            camera_offset_x,
            camera_offset_y,
        );
        queue.write_buffer(&self.render_buffer, 0, bytemuck::bytes_of(&params));
    }

    /// Create bind group for GPU-driven particle spawning into both ping-pong buffers.
    pub fn create_spawn_bind_group(
        &self,
        device: &Device,
        buffers: &SimulationBuffers,
    ) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: Some("Particle Spawn Bind Group"),
            layout: &self.spawn_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: buffers.current_pos_type().as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: buffers.current_velocities().as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: buffers.next_pos_type().as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: buffers.next_velocities().as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: self.spawn_buffer.as_entire_binding(),
                },
            ],
        })
    }

    /// Update GPU spawn parameters.
    pub fn update_spawn(&self, queue: &Queue, params: SpawnParamsUniform) {
        queue.write_buffer(&self.spawn_buffer, 0, bytemuck::bytes_of(&params));
    }
}

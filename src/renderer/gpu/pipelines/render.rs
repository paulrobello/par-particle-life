//! Render pipelines for particle visualization.
//!
//! This module contains the render pipelines for drawing particles with
//! various effects including glow, mirror wrap, and infinite wrap.

use wgpu::util::DeviceExt;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, BlendState, Buffer, BufferBindingType, ColorTargetState,
    ColorWrites, Device, FragmentState, FrontFace, MultisampleState, PipelineCompilationOptions,
    PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology, Queue,
    RenderPipeline, RenderPipelineDescriptor, ShaderStages, TextureFormat, VertexState,
};

use super::{CameraUniform, load_shader};
use crate::renderer::gpu::{
    GlowParamsUniform, InfiniteParamsUniform, MirrorParamsUniform, SimulationBuffers,
};

/// Render pipelines for particle visualization.
pub struct RenderPipelines {
    /// Pipeline for rendering particles as point sprites.
    pub particle_pipeline: RenderPipeline,
    /// Pipeline for rendering particle glow effect.
    pub glow_pipeline: RenderPipeline,
    /// Pipeline for rendering particles with mirror wrap effect.
    pub mirror_pipeline: RenderPipeline,
    /// Pipeline for rendering particles with infinite wrap tiling.
    pub infinite_pipeline: RenderPipeline,
    /// Bind group layout for particle rendering.
    pub render_bind_group_layout: BindGroupLayout,
    /// Bind group layout for glow rendering.
    pub glow_bind_group_layout: BindGroupLayout,
    /// Bind group layout for mirror wrap rendering.
    pub mirror_bind_group_layout: BindGroupLayout,
    /// Bind group layout for infinite wrap rendering.
    pub infinite_bind_group_layout: BindGroupLayout,
    /// Camera uniform buffer.
    pub camera_buffer: Buffer,
    /// Glow parameters uniform buffer.
    pub glow_buffer: Buffer,
    /// Mirror wrap parameters uniform buffer.
    pub mirror_buffer: Buffer,
    /// Infinite wrap parameters uniform buffer.
    pub infinite_buffer: Buffer,
}

impl RenderPipelines {
    /// Create render pipelines for particle visualization.
    pub fn new(device: &Device, surface_format: TextureFormat) -> Self {
        // Load render shaders with FP16 support
        let render_shader = load_shader(
            device,
            "Particle Render Shader",
            include_str!("../../../../shaders/particle_render.wgsl"),
        );

        let glow_shader = load_shader(
            device,
            "Particle Glow Shader",
            include_str!("../../../../shaders/particle_render_glow.wgsl"),
        );

        let mirror_shader = load_shader(
            device,
            "Mirror Wrap Render Shader",
            include_str!("../../../../shaders/particle_render_mirror.wgsl"),
        );

        let infinite_shader = load_shader(
            device,
            "Infinite Wrap Render Shader",
            include_str!("../../../../shaders/particle_render_infinite.wgsl"),
        );

        // Create bind group layouts
        let render_bind_group_layout = Self::create_render_bind_group_layout(device);
        let glow_bind_group_layout = Self::create_glow_bind_group_layout(device);
        let mirror_bind_group_layout = Self::create_mirror_bind_group_layout(device);
        let infinite_bind_group_layout = Self::create_infinite_bind_group_layout(device);

        // Create pipeline layouts
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&render_bind_group_layout],
            push_constant_ranges: &[],
        });

        let glow_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Glow Pipeline Layout"),
            bind_group_layouts: &[&glow_bind_group_layout],
            push_constant_ranges: &[],
        });

        let mirror_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Mirror Pipeline Layout"),
            bind_group_layouts: &[&mirror_bind_group_layout],
            push_constant_ranges: &[],
        });

        let infinite_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Infinite Pipeline Layout"),
            bind_group_layouts: &[&infinite_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create particle render pipeline
        let particle_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Particle Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &render_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(FragmentState {
                module: &render_shader,
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
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // Create glow render pipeline with additive blending
        let glow_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Glow Render Pipeline"),
            layout: Some(&glow_pipeline_layout),
            vertex: VertexState {
                module: &glow_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(FragmentState {
                module: &glow_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format: surface_format,
                    // Additive blending for glow effect
                    blend: Some(BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // Create mirror wrap render pipeline
        let mirror_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Mirror Wrap Render Pipeline"),
            layout: Some(&mirror_pipeline_layout),
            vertex: VertexState {
                module: &mirror_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(FragmentState {
                module: &mirror_shader,
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
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // Create infinite wrap render pipeline
        let infinite_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Infinite Wrap Render Pipeline"),
            layout: Some(&infinite_pipeline_layout),
            vertex: VertexState {
                module: &infinite_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(FragmentState {
                module: &infinite_shader,
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
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // Create camera buffer with default values
        let camera = CameraUniform::new(1920.0, 1080.0, 1920.0, 1080.0);
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::bytes_of(&camera),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create glow buffer with default values
        let glow_params = GlowParamsUniform {
            glow_size: 4.0,
            glow_intensity: 0.5,
            glow_steepness: 2.0,
            _padding: 0.0,
        };
        let glow_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Glow Buffer"),
            contents: bytemuck::bytes_of(&glow_params),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create mirror wrap buffer with default values
        let mirror_params = MirrorParamsUniform {
            num_copies: 5,
            _padding: [0; 3],
        };
        let mirror_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mirror Buffer"),
            contents: bytemuck::bytes_of(&mirror_params),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create infinite wrap buffer with default values
        let infinite_params = InfiniteParamsUniform {
            start_x: -1,
            start_y: -1,
            num_copies_x: 3,
            num_copies_y: 3,
        };
        let infinite_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Infinite Buffer"),
            contents: bytemuck::bytes_of(&infinite_params),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            particle_pipeline,
            glow_pipeline,
            mirror_pipeline,
            infinite_pipeline,
            render_bind_group_layout,
            glow_bind_group_layout,
            mirror_bind_group_layout,
            infinite_bind_group_layout,
            camera_buffer,
            glow_buffer,
            mirror_buffer,
            infinite_buffer,
        }
    }

    /// Create bind group layout for particle rendering.
    fn create_render_bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Render Bind Group Layout"),
            entries: &[
                // pos_type (storage, read-only)
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // colors (storage, read-only)
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // params (uniform)
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // camera (uniform)
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        })
    }

    /// Create render bind group.
    ///
    /// Takes a reference to the current particle buffer for rendering.
    pub fn create_render_bind_group(
        &self,
        device: &Device,
        pos_type: &Buffer,
        buffers: &SimulationBuffers,
    ) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: Some("Render Bind Group"),
            layout: &self.render_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: pos_type.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: buffers.colors.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: buffers.params.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: self.camera_buffer.as_entire_binding(),
                },
            ],
        })
    }

    /// Update camera uniform buffer.
    pub fn update_camera(
        &self,
        queue: &Queue,
        world_width: f32,
        world_height: f32,
        _viewport_width: f32,
        _viewport_height: f32,
    ) {
        self.update_camera_with_zoom(queue, world_width, world_height, 1.0, 0.0, 0.0);
    }

    /// Update camera uniform buffer with zoom and pan.
    pub fn update_camera_with_zoom(
        &self,
        queue: &Queue,
        world_width: f32,
        world_height: f32,
        zoom: f32,
        offset_x: f32,
        offset_y: f32,
    ) {
        let camera = CameraUniform::with_zoom_and_offset(
            world_width,
            world_height,
            zoom,
            offset_x,
            offset_y,
        );
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::bytes_of(&camera));
    }

    /// Update glow parameters uniform buffer.
    pub fn update_glow(&self, queue: &Queue, config: &crate::simulation::SimulationConfig) {
        let glow_params = GlowParamsUniform::from_config(config);
        queue.write_buffer(&self.glow_buffer, 0, bytemuck::bytes_of(&glow_params));
    }

    /// Create bind group layout for glow rendering.
    fn create_glow_bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Glow Bind Group Layout"),
            entries: &[
                // pos_type (storage, read-only)
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // colors (storage, read-only)
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // params (uniform)
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // camera (uniform)
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // glow params (uniform)
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        })
    }

    /// Create glow render bind group.
    pub fn create_glow_bind_group(
        &self,
        device: &Device,
        pos_type: &Buffer,
        buffers: &SimulationBuffers,
    ) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: Some("Glow Bind Group"),
            layout: &self.glow_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: pos_type.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: buffers.colors.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: buffers.params.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: self.camera_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: self.glow_buffer.as_entire_binding(),
                },
            ],
        })
    }

    /// Create bind group layout for mirror wrap rendering.
    fn create_mirror_bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Mirror Bind Group Layout"),
            entries: &[
                // pos_type (storage, read-only)
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // colors (storage, read-only)
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // params (uniform)
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // camera (uniform)
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // mirror params (uniform)
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        })
    }

    /// Create mirror wrap render bind group.
    pub fn create_mirror_bind_group(
        &self,
        device: &Device,
        pos_type: &Buffer,
        buffers: &SimulationBuffers,
    ) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: Some("Mirror Bind Group"),
            layout: &self.mirror_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: pos_type.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: buffers.colors.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: buffers.params.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: self.camera_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: self.mirror_buffer.as_entire_binding(),
                },
            ],
        })
    }

    /// Update mirror wrap parameters.
    pub fn update_mirror(&self, queue: &Queue, config: &crate::simulation::SimulationConfig) {
        let mirror_params = MirrorParamsUniform::from_config(config);
        queue.write_buffer(&self.mirror_buffer, 0, bytemuck::bytes_of(&mirror_params));
    }

    /// Create bind group layout for infinite wrap rendering.
    fn create_infinite_bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Infinite Bind Group Layout"),
            entries: &[
                // pos_type (storage, read-only)
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // colors (storage, read-only)
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // params (uniform)
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // camera (uniform)
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // infinite params (uniform)
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        })
    }

    /// Create infinite wrap render bind group.
    pub fn create_infinite_bind_group(
        &self,
        device: &Device,
        pos_type: &Buffer,
        buffers: &SimulationBuffers,
    ) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: Some("Infinite Bind Group"),
            layout: &self.infinite_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: pos_type.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: buffers.colors.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: buffers.params.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: self.camera_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: self.infinite_buffer.as_entire_binding(),
                },
            ],
        })
    }

    /// Update infinite wrap parameters based on camera state.
    pub fn update_infinite(
        &self,
        queue: &Queue,
        world_width: f32,
        world_height: f32,
        camera_center_x: f32,
        camera_center_y: f32,
        zoom: f32,
    ) {
        let infinite_params = InfiniteParamsUniform::from_camera(
            world_width,
            world_height,
            camera_center_x,
            camera_center_y,
            zoom,
        );
        queue.write_buffer(
            &self.infinite_buffer,
            0,
            bytemuck::bytes_of(&infinite_params),
        );
    }

    /// Get the current infinite params for calculating instance count.
    pub fn get_infinite_params(
        world_width: f32,
        world_height: f32,
        camera_center_x: f32,
        camera_center_y: f32,
        zoom: f32,
    ) -> InfiniteParamsUniform {
        InfiniteParamsUniform::from_camera(
            world_width,
            world_height,
            camera_center_x,
            camera_center_y,
            zoom,
        )
    }
}

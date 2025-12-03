//! Compute pipelines for particle simulation.
//!
//! This module contains the force and advance compute pipelines that run
//! the particle physics simulation on the GPU.

use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, ComputePipeline,
    ComputePipelineDescriptor, Device, PipelineCompilationOptions, PipelineLayoutDescriptor,
    ShaderStages,
};

use super::load_shader;
use crate::renderer::gpu::SimulationBuffers;

/// Compute pipelines for particle simulation.
pub struct ComputePipelines {
    /// Pipeline for computing forces between particles.
    pub force_pipeline: ComputePipeline,
    /// Pipeline for advancing particle positions.
    pub advance_pipeline: ComputePipeline,
    /// Bind group layout for force computation.
    pub force_bind_group_layout: BindGroupLayout,
    /// Bind group layout for position advancement.
    pub advance_bind_group_layout: BindGroupLayout,
}

impl ComputePipelines {
    /// Create compute pipelines for particle simulation.
    pub fn new(device: &Device) -> Self {
        // Load shaders with FP16 support
        let force_shader = load_shader(
            device,
            "Force Compute Shader",
            include_str!("../../../../shaders/particle_forces.wgsl"),
        );

        let advance_shader = load_shader(
            device,
            "Advance Compute Shader",
            include_str!("../../../../shaders/particle_advance.wgsl"),
        );

        // Create bind group layouts
        let force_bind_group_layout = Self::create_force_bind_group_layout(device);
        let advance_bind_group_layout = Self::create_advance_bind_group_layout(device);

        // Create pipeline layouts
        let force_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Force Pipeline Layout"),
            bind_group_layouts: &[&force_bind_group_layout],
            push_constant_ranges: &[],
        });

        let advance_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Advance Pipeline Layout"),
            bind_group_layouts: &[&advance_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create compute pipelines
        let force_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Force Compute Pipeline"),
            layout: Some(&force_pipeline_layout),
            module: &force_shader,
            entry_point: Some("main"),
            compilation_options: PipelineCompilationOptions::default(),
            cache: None,
        });

        let advance_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Advance Compute Pipeline"),
            layout: Some(&advance_pipeline_layout),
            module: &advance_shader,
            entry_point: Some("main"),
            compilation_options: PipelineCompilationOptions::default(),
            cache: None,
        });

        Self {
            force_pipeline,
            advance_pipeline,
            force_bind_group_layout,
            advance_bind_group_layout,
        }
    }

    /// Create bind group layout for force computation.
    fn create_force_bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Force Bind Group Layout"),
            entries: &[
                // pos_type (current, read-only)
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
                // vel_in (current, read-only)
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // vel_out (next, read-write)
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
                // params (uniform)
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // interaction_matrix
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // min_radius
                BindGroupLayoutEntry {
                    binding: 5,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // max_radius
                BindGroupLayoutEntry {
                    binding: 6,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        })
    }

    /// Create bind group layout for position advancement.
    fn create_advance_bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Advance Bind Group Layout"),
            entries: &[
                // pos (read-write)
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
                // vel (read-write)
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
                // params (uniform)
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
                // brush params (uniform)
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::COMPUTE,
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

    /// Create force compute bind group.
    pub fn create_force_bind_group(
        &self,
        device: &Device,
        pos_type: &Buffer,
        vel_in: &Buffer,
        vel_out: &Buffer,
        buffers: &SimulationBuffers,
    ) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: Some("Force Bind Group"),
            layout: &self.force_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: pos_type.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: vel_in.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: vel_out.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: buffers.params.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: buffers.interaction_matrix.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 5,
                    resource: buffers.min_radius.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 6,
                    resource: buffers.max_radius.as_entire_binding(),
                },
            ],
        })
    }

    /// Create advance compute bind group.
    pub fn create_advance_bind_group(
        &self,
        device: &Device,
        pos: &Buffer,
        vel: &Buffer,
        params: &Buffer,
        brush_params: &Buffer,
    ) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: Some("Advance Bind Group"),
            layout: &self.advance_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: pos.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: vel.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: params.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: brush_params.as_entire_binding(),
                },
            ],
        })
    }
}

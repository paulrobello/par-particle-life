//! Compute pipelines for spatial hashing optimization.
//!
//! Spatial hashing divides the simulation space into a grid of bins.
//! Particles are sorted by bin, allowing force calculation to only check
//! neighboring bins instead of all particles (O(n*k) vs O(n²)).

use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, ComputePipeline,
    ComputePipelineDescriptor, Device, PipelineCompilationOptions, PipelineLayoutDescriptor,
    ShaderStages,
};

use super::load_shader;
use crate::renderer::gpu::{SimulationBuffers, SpatialHashBuffers};

/// Compute pipelines for spatial hashing optimization.
pub struct SpatialHashPipelines {
    /// Pipeline for clearing bin counters.
    pub clear_pipeline: ComputePipeline,
    /// Pipeline for counting particles per bin.
    pub count_pipeline: ComputePipeline,
    /// Pipeline for prefix sum (computing bin offsets).
    pub prefix_sum_pipeline: ComputePipeline,
    /// Pipeline for sorting particles by bin.
    pub sort_pipeline: ComputePipeline,
    /// Pipeline for binned force calculation.
    pub forces_pipeline: ComputePipeline,
    /// Bind group layout for bin clear.
    pub clear_bind_group_layout: BindGroupLayout,
    /// Bind group layout for bin count.
    pub count_bind_group_layout: BindGroupLayout,
    /// Bind group layout for prefix sum.
    pub prefix_sum_bind_group_layout: BindGroupLayout,
    /// Bind group layout for particle sort.
    pub sort_bind_group_layout: BindGroupLayout,
    /// Bind group layout for binned force calculation.
    pub forces_bind_group_layout: BindGroupLayout,
}

impl SpatialHashPipelines {
    /// Create spatial hash pipelines.
    pub fn new(device: &Device) -> Self {
        // Load shaders with FP16 support
        let clear_shader = load_shader(
            device,
            "Bin Clear Shader",
            include_str!("../../../../shaders/bin_clear.wgsl"),
        );

        let count_shader = load_shader(
            device,
            "Bin Count Shader",
            include_str!("../../../../shaders/bin_count.wgsl"),
        );

        let prefix_sum_shader = load_shader(
            device,
            "Bin Prefix Sum Shader",
            include_str!("../../../../shaders/bin_prefix_sum.wgsl"),
        );

        let sort_shader = load_shader(
            device,
            "Bin Sort Shader",
            include_str!("../../../../shaders/bin_sort.wgsl"),
        );

        let forces_shader = load_shader(
            device,
            "Binned Forces Shader",
            include_str!("../../../../shaders/particle_forces_binned.wgsl"),
        );

        // Create bind group layouts
        let clear_bind_group_layout = Self::create_clear_bind_group_layout(device);
        let count_bind_group_layout = Self::create_count_bind_group_layout(device);
        let prefix_sum_bind_group_layout = Self::create_prefix_sum_bind_group_layout(device);
        let sort_bind_group_layout = Self::create_sort_bind_group_layout(device);
        let forces_bind_group_layout = Self::create_forces_bind_group_layout(device);

        // Create pipeline layouts
        let clear_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Bin Clear Pipeline Layout"),
            bind_group_layouts: &[&clear_bind_group_layout],
            push_constant_ranges: &[],
        });

        let count_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Bin Count Pipeline Layout"),
            bind_group_layouts: &[&count_bind_group_layout],
            push_constant_ranges: &[],
        });

        let prefix_sum_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Prefix Sum Pipeline Layout"),
            bind_group_layouts: &[&prefix_sum_bind_group_layout],
            push_constant_ranges: &[],
        });

        let sort_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Bin Sort Pipeline Layout"),
            bind_group_layouts: &[&sort_bind_group_layout],
            push_constant_ranges: &[],
        });

        let forces_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Binned Forces Pipeline Layout"),
            bind_group_layouts: &[&forces_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create pipelines
        let clear_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Bin Clear Pipeline"),
            layout: Some(&clear_pipeline_layout),
            module: &clear_shader,
            entry_point: Some("main"),
            compilation_options: PipelineCompilationOptions::default(),
            cache: None,
        });

        let count_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Bin Count Pipeline"),
            layout: Some(&count_pipeline_layout),
            module: &count_shader,
            entry_point: Some("main"),
            compilation_options: PipelineCompilationOptions::default(),
            cache: None,
        });

        let prefix_sum_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Prefix Sum Pipeline"),
            layout: Some(&prefix_sum_pipeline_layout),
            module: &prefix_sum_shader,
            entry_point: Some("main"),
            compilation_options: PipelineCompilationOptions::default(),
            cache: None,
        });

        let sort_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Bin Sort Pipeline"),
            layout: Some(&sort_pipeline_layout),
            module: &sort_shader,
            entry_point: Some("main"),
            compilation_options: PipelineCompilationOptions::default(),
            cache: None,
        });

        let forces_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Binned Forces Pipeline"),
            layout: Some(&forces_pipeline_layout),
            module: &forces_shader,
            entry_point: Some("main"),
            compilation_options: PipelineCompilationOptions::default(),
            cache: None,
        });

        Self {
            clear_pipeline,
            count_pipeline,
            prefix_sum_pipeline,
            sort_pipeline,
            forces_pipeline,
            clear_bind_group_layout,
            count_bind_group_layout,
            prefix_sum_bind_group_layout,
            sort_bind_group_layout,
            forces_bind_group_layout,
        }
    }

    /// Create bind group layout for bin clear.
    fn create_clear_bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Bin Clear Bind Group Layout"),
            entries: &[
                // bin_counts (storage, read-write)
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
                // total_bins (uniform)
                BindGroupLayoutEntry {
                    binding: 1,
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

    /// Create bind group layout for bin count.
    fn create_count_bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Bin Count Bind Group Layout"),
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
                // bin_counts (storage, read-write)
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
            ],
        })
    }

    /// Create bind group layout for prefix sum.
    fn create_prefix_sum_bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Prefix Sum Bind Group Layout"),
            entries: &[
                // source (storage, read-only)
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
                // destination (storage, read-write)
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
                // step_size (uniform)
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
        })
    }

    /// Create bind group layout for particle sort.
    fn create_sort_bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Bin Sort Bind Group Layout"),
            entries: &[
                // pos_type_in (storage, read-only)
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
                // pos_type_out (storage, read-write)
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
                // vel_in (storage, read-only)
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // vel_out (storage, read-write)
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
                // bin_offsets (storage, read-only)
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
                // bin_counts (storage, read-write for atomic)
                BindGroupLayoutEntry {
                    binding: 5,
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
                    binding: 6,
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

    /// Create bind group layout for binned force calculation.
    fn create_forces_bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Binned Forces Bind Group Layout"),
            entries: &[
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
                // interaction_matrix (storage, read-only)
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // min_radius (storage, read-only)
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
                // max_radius (storage, read-only)
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
                // bin_offsets (storage, read-only)
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
                // spatial params (uniform)
                BindGroupLayoutEntry {
                    binding: 7,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // sorted_pos_type (storage, read-only)
                BindGroupLayoutEntry {
                    binding: 8,
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

    /// Create bin clear bind group.
    pub fn create_clear_bind_group(
        &self,
        device: &Device,
        spatial: &SpatialHashBuffers,
        use_buffer_a: bool,
    ) -> BindGroup {
        let bin_buffer = if use_buffer_a {
            &spatial.bin_counts_a
        } else {
            &spatial.bin_counts_b
        };

        device.create_bind_group(&BindGroupDescriptor {
            label: Some("Bin Clear Bind Group"),
            layout: &self.clear_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: bin_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: spatial.total_bins_uniform.as_entire_binding(),
                },
            ],
        })
    }

    /// Create bin count bind group.
    pub fn create_count_bind_group(
        &self,
        device: &Device,
        pos_type: &Buffer,
        spatial: &SpatialHashBuffers,
    ) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: Some("Bin Count Bind Group"),
            layout: &self.count_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: pos_type.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: spatial.bin_counts_a.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: spatial.params.as_entire_binding(),
                },
            ],
        })
    }

    /// Create prefix sum bind group for one pass.
    pub fn create_prefix_sum_bind_group(
        &self,
        device: &Device,
        source: &Buffer,
        destination: &Buffer,
        step_size_uniform: &Buffer,
    ) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: Some("Prefix Sum Bind Group"),
            layout: &self.prefix_sum_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: source.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: destination.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: step_size_uniform.as_entire_binding(),
                },
            ],
        })
    }

    /// Create bin sort bind group.
    #[allow(clippy::too_many_arguments)]
    pub fn create_sort_bind_group(
        &self,
        device: &Device,
        pos_type_in: &Buffer,
        pos_type_out: &Buffer,
        vel_in: &Buffer,
        vel_out: &Buffer,
        spatial: &SpatialHashBuffers,
        use_offset_buffer_a: bool,
        use_count_buffer_a: bool,
    ) -> BindGroup {
        let offset_buffer = if use_offset_buffer_a {
            &spatial.bin_counts_a
        } else {
            &spatial.bin_counts_b
        };
        let count_buffer = if use_count_buffer_a {
            &spatial.bin_counts_a
        } else {
            &spatial.bin_counts_b
        };

        device.create_bind_group(&BindGroupDescriptor {
            label: Some("Bin Sort Bind Group"),
            layout: &self.sort_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: pos_type_in.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: pos_type_out.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: vel_in.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: vel_out.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: offset_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 5,
                    resource: count_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 6,
                    resource: spatial.params.as_entire_binding(),
                },
            ],
        })
    }

    /// Create binned forces bind group.
    pub fn create_forces_bind_group(
        &self,
        device: &Device,
        velocities: &Buffer,
        sorted_pos_type: &Buffer,
        sim_buffers: &SimulationBuffers,
        spatial: &SpatialHashBuffers,
    ) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: Some("Binned Forces Bind Group"),
            layout: &self.forces_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 1,
                    resource: velocities.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: sim_buffers.params.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: sim_buffers.interaction_matrix.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: sim_buffers.min_radius.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 5,
                    resource: sim_buffers.max_radius.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 6,
                    resource: spatial.current_offsets().as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 7,
                    resource: spatial.params.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 8,
                    resource: sorted_pos_type.as_entire_binding(),
                },
            ],
        })
    }
}

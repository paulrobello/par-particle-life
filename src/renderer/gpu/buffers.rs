//! GPU buffer management for particle simulation.
//!
//! This module handles the creation and management of GPU buffers
//! for particles, forces, interaction matrices, and simulation parameters.

use bytemuck::{Pod, Zeroable};
use wgpu::{Buffer, BufferUsages, Device, Queue, util::DeviceExt};

use crate::simulation::{
    InteractionMatrix, Particle, ParticlePosType, ParticleVel, ParticleVelHalf, RadiusMatrix,
    SimulationConfig,
};

/// Parameters for spatial hashing uniform buffer.
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct SpatialParamsUniform {
    /// Number of particles.
    pub num_particles: u32,
    /// Cell size for spatial hash grid.
    pub cell_size: f32,
    /// Number of grid cells in X direction.
    pub grid_width: u32,
    /// Number of grid cells in Y direction.
    pub grid_height: u32,
}

impl SpatialParamsUniform {
    /// Create spatial parameters from simulation config.
    ///
    /// Cell size is clamped to the maximum interaction radius so that
    /// a 3x3 bin neighborhood fully covers the force range.
    pub fn from_config(config: &SimulationConfig, max_radius: f32) -> Self {
        let cell_size = config.spatial_hash_cell_size.max(max_radius);
        let grid_width = (config.world_size.x / cell_size).ceil() as u32;
        let grid_height = (config.world_size.y / cell_size).ceil() as u32;

        Self {
            num_particles: config.num_particles,
            cell_size,
            grid_width,
            grid_height,
        }
    }

    /// Get total number of bins.
    pub fn total_bins(&self) -> u32 {
        self.grid_width * self.grid_height
    }
}

/// Parameters for glow effect uniform buffer.
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct GlowParamsUniform {
    /// Multiplier for glow quad size (typically 2.0-8.0).
    pub glow_size: f32,
    /// Intensity of the glow (0.0-2.0).
    pub glow_intensity: f32,
    /// Steepness of falloff (higher = sharper edge, 1.0-4.0).
    pub glow_steepness: f32,
    /// Padding for alignment.
    pub _padding: f32,
}

/// Parameters for mirror wrap rendering.
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct MirrorParamsUniform {
    /// Number of copies to render (5 or 9).
    pub num_copies: u32,
    /// Padding for alignment (vec3<u32>).
    pub _padding: [u32; 3],
}

impl MirrorParamsUniform {
    /// Create mirror parameters from simulation config.
    pub fn from_config(config: &SimulationConfig) -> Self {
        Self {
            num_copies: config.mirror_wrap_count,
            _padding: [0; 3],
        }
    }
}

/// Parameters for infinite wrap rendering.
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct InfiniteParamsUniform {
    /// Start offset for tile grid X (can be negative).
    pub start_x: i32,
    /// Start offset for tile grid Y (can be negative).
    pub start_y: i32,
    /// Number of tile copies in X direction.
    pub num_copies_x: u32,
    /// Number of tile copies in Y direction.
    pub num_copies_y: u32,
}

impl InfiniteParamsUniform {
    /// Create infinite params for rendering based on camera position and zoom.
    ///
    /// Calculates how many tiles are needed to fill the visible area.
    pub fn from_camera(
        world_width: f32,
        world_height: f32,
        camera_center_x: f32,
        camera_center_y: f32,
        zoom: f32,
    ) -> Self {
        // Calculate visible area in world units
        let visible_width = world_width / zoom;
        let visible_height = world_height / zoom;

        // Calculate how many tiles we need in each direction
        // Add 2 extra tiles to ensure coverage during panning
        let tiles_x = (visible_width / world_width).ceil() as u32 + 2;
        let tiles_y = (visible_height / world_height).ceil() as u32 + 2;

        // Calculate start offset (which tile the camera center is in)
        let camera_tile_x = (camera_center_x / world_width).floor() as i32;
        let camera_tile_y = (camera_center_y / world_height).floor() as i32;

        // Center the tile grid on the camera
        let half_tiles_x = (tiles_x / 2) as i32;
        let half_tiles_y = (tiles_y / 2) as i32;

        Self {
            start_x: camera_tile_x - half_tiles_x,
            start_y: camera_tile_y - half_tiles_y,
            num_copies_x: tiles_x,
            num_copies_y: tiles_y,
        }
    }

    /// Get total number of copies.
    pub fn total_copies(&self) -> u32 {
        self.num_copies_x * self.num_copies_y
    }
}

impl GlowParamsUniform {
    /// Create glow parameters from simulation config.
    pub fn from_config(config: &SimulationConfig) -> Self {
        Self {
            glow_size: config.glow_size,
            glow_intensity: config.glow_intensity,
            glow_steepness: config.glow_steepness,
            _padding: 0.0,
        }
    }
}

/// Uniform buffer for brush interaction parameters.
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct BrushParamsUniform {
    /// Brush position X in world coordinates.
    pub pos_x: f32,
    /// Brush position Y in world coordinates.
    pub pos_y: f32,
    /// Brush velocity X for directional force.
    pub vel_x: f32,
    /// Brush velocity Y for directional force.
    pub vel_y: f32,
    /// Brush radius in world coordinates.
    pub radius: f32,
    /// Force strength (positive = attract, negative = repel).
    pub force: f32,
    /// Directional force multiplier.
    pub directional_force: f32,
    /// Is brush active (0 = inactive, 1 = active).
    pub is_active: u32,
    /// Number of particles.
    pub num_particles: u32,
    /// Target particle type (-1 for all).
    pub target_type: i32,
    /// Padding for 16-byte alignment.
    pub _padding: [u32; 2],
}

impl BrushParamsUniform {
    /// Create brush parameters from brush state.
    pub fn from_brush_state(brush: &crate::app::BrushState, num_particles: u32) -> Self {
        Self {
            pos_x: brush.position.x,
            pos_y: brush.position.y,
            vel_x: brush.velocity.x,
            vel_y: brush.velocity.y,
            radius: brush.radius,
            force: brush.get_force(),
            directional_force: brush.directional_force,
            is_active: if brush.is_active { 1 } else { 0 },
            num_particles,
            target_type: brush.target_type,
            _padding: [0; 2],
        }
    }
}

/// Uniform buffer for brush circle rendering parameters.
///
/// WGSL memory layout: vec3<f32> has 16-byte alignment, so the struct
/// needs explicit padding to match. Total size must be 80 bytes.
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct BrushRenderUniform {
    /// Brush position X in world coordinates.
    pub pos_x: f32,
    /// Brush position Y in world coordinates.
    pub pos_y: f32,
    /// Brush radius.
    pub radius: f32,
    /// Circle color R (0-1).
    pub color_r: f32,
    /// Circle color G (0-1).
    pub color_g: f32,
    /// Circle color B (0-1).
    pub color_b: f32,
    /// Circle color A (0-1).
    pub color_a: f32,
    /// Is brush visible (0 = hidden, 1 = visible).
    pub is_visible: u32,
    /// World width for camera transform.
    pub world_width: f32,
    /// World height for camera transform.
    pub world_height: f32,
    /// Camera zoom.
    pub camera_zoom: f32,
    /// Camera offset X.
    pub camera_offset_x: f32,
    /// Camera offset Y.
    pub camera_offset_y: f32,
    /// Padding to align vec3 to 16-byte boundary (52 bytes -> 64 bytes).
    pub _padding1: [f32; 3],
    /// Padding matching WGSL vec3<f32> (16-byte aligned, takes 16 bytes).
    pub _padding2: [f32; 4],
}

impl BrushRenderUniform {
    /// Create render parameters from brush state and camera.
    pub fn from_brush_state(
        brush: &crate::app::BrushState,
        world_width: f32,
        world_height: f32,
        camera_zoom: f32,
        camera_offset_x: f32,
        camera_offset_y: f32,
    ) -> Self {
        // Color based on tool type
        let (r, g, b) = match brush.tool {
            crate::app::BrushTool::None => (0.5, 0.5, 0.5),
            crate::app::BrushTool::Draw => (0.2, 0.8, 0.2),
            crate::app::BrushTool::Erase => (0.8, 0.2, 0.2),
            crate::app::BrushTool::Attract => (0.2, 0.6, 0.9),
            crate::app::BrushTool::Repel => (0.9, 0.6, 0.2),
        };

        Self {
            pos_x: brush.position.x,
            pos_y: brush.position.y,
            radius: brush.radius,
            color_r: r,
            color_g: g,
            color_b: b,
            color_a: 0.8,
            is_visible: if brush.show_circle && brush.tool != crate::app::BrushTool::None {
                1
            } else {
                0
            },
            world_width,
            world_height,
            camera_zoom,
            camera_offset_x,
            camera_offset_y,
            _padding1: [0.0; 3],
            _padding2: [0.0; 4],
        }
    }
}

/// Uniform buffer containing simulation parameters for shaders.
///
/// This struct is tightly packed and aligned for GPU uniform buffer layout.
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct SimParamsUniform {
    /// Number of particles.
    pub num_particles: u32,
    /// Number of particle types.
    pub num_types: u32,
    /// Force scaling factor.
    pub force_factor: f32,
    /// Friction coefficient.
    pub friction: f32,
    /// Repulsion strength at close range.
    pub repel_strength: f32,
    /// Maximum velocity.
    pub max_velocity: f32,
    /// World width.
    pub world_width: f32,
    /// World height.
    pub world_height: f32,
    /// Boundary mode (0=Repel, 1=Wrap, 2=MirrorWrap, 3=InfiniteWrap).
    pub boundary_mode: u32,
    /// Wall repulsion strength for Repel mode (0-100).
    pub wall_repel_strength: f32,
    /// Particle render size.
    pub particle_size: f32,
    /// Time delta for this frame.
    pub dt: f32,
    /// Maximum particles in a bin before forces are scaled.
    pub max_bin_density: f32,
    /// Maximum neighbors to check per particle (0 = unlimited).
    pub neighbor_budget: u32,
    /// Padding to match WGSL struct alignment (vec3<u32> requires 16-byte alignment + struct rounds to 16 bytes).
    _padding: [u32; 6],
}

impl SimParamsUniform {
    /// Create uniform parameters from simulation config.
    pub fn from_config(config: &SimulationConfig, dt: f32) -> Self {
        use crate::simulation::BoundaryMode;

        Self {
            num_particles: config.num_particles,
            num_types: config.num_types,
            force_factor: config.force_factor,
            friction: config.friction,
            repel_strength: config.repel_strength,
            max_velocity: config.max_velocity,
            world_width: config.world_size.x,
            world_height: config.world_size.y,
            boundary_mode: match config.boundary_mode {
                BoundaryMode::Repel => 0,
                BoundaryMode::Wrap => 1,
                BoundaryMode::MirrorWrap => 2,
                BoundaryMode::InfiniteWrap => 3,
            },
            wall_repel_strength: config.wall_repel_strength,
            particle_size: config.particle_size,
            dt,
            max_bin_density: config.max_bin_density,
            neighbor_budget: config.neighbor_budget,
            _padding: [0; 6],
        }
    }
}

/// Manages GPU buffers for the particle simulation.
///
/// Uses double-buffering (ping-pong) for particles to enable
/// GPU compute without race conditions.
/// Now uses SoA (Structure of Arrays) layout: Position+Type in one buffer, Velocity in another.
pub struct SimulationBuffers {
    /// Position and Type buffers (double-buffered).
    pub pos_type: [Buffer; 2],
    /// Velocity buffers (double-buffered).
    pub velocities: [Buffer; 2],
    /// Current buffer index (0 or 1) - the "read" buffer for rendering.
    pub current_buffer: usize,
    /// Interaction matrix buffer.
    pub interaction_matrix: Buffer,
    /// Minimum radius matrix buffer.
    pub min_radius: Buffer,
    /// Maximum radius matrix buffer.
    pub max_radius: Buffer,
    /// Simulation parameters uniform buffer.
    pub params: Buffer,
    /// Color palette buffer for particle types.
    pub colors: Buffer,
    /// Current number of particles.
    pub num_particles: u32,
    /// Current number of particle types.
    pub num_types: u32,
    /// Whether to use half-precision (f16) for particle storage.
    pub use_f16: bool,
}

impl SimulationBuffers {
    /// Create new simulation buffers.
    ///
    /// # Arguments
    /// * `device` - The wgpu device
    /// * `particles` - Initial particle data
    /// * `interaction_matrix` - Interaction matrix between types
    /// * `radius_matrix` - Min/max radius matrices
    /// * `colors` - RGBA colors for each particle type
    /// * `config` - Simulation configuration
    pub fn new(
        device: &Device,
        particles: &[Particle],
        interaction_matrix: &InteractionMatrix,
        radius_matrix: &RadiusMatrix,
        colors: &[[f32; 4]],
        config: &SimulationConfig,
    ) -> Self {
        let num_particles = particles.len() as u32;
        let num_types = config.num_types;

        // Check if we should use F16 (based on device features)
        // We can't access device features directly from here easily without passing them or checking device.
        // Assuming the caller will recreate buffers if they want to switch mode is safer, but here we check device.
        let use_f16 = device.features().contains(wgpu::Features::SHADER_F16);

        // Create double-buffered particle buffers
        // Note: Positions are always F32 to ensure precision for large world coordinates.
        // Velocities can be F16 to save bandwidth.
        let pos_type_data: Vec<ParticlePosType> =
            particles.iter().map(ParticlePosType::from).collect();

        let pt0 = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Particle Pos/Type Buffer 0"),
            contents: bytemuck::cast_slice(&pos_type_data),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
        });
        let pt1 = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Particle Pos/Type Buffer 1"),
            contents: bytemuck::cast_slice(&pos_type_data),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
        });

        let (vel_buffer_0, vel_buffer_1) = if use_f16 {
            let vel_data: Vec<ParticleVelHalf> =
                particles.iter().map(ParticleVelHalf::from).collect();

            let v0 = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Particle Velocity Buffer 0 (F16)"),
                contents: bytemuck::cast_slice(&vel_data),
                usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
            });
            let v1 = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Particle Velocity Buffer 1 (F16)"),
                contents: bytemuck::cast_slice(&vel_data),
                usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
            });
            (v0, v1)
        } else {
            let vel_data: Vec<ParticleVel> = particles.iter().map(ParticleVel::from).collect();

            let v0 = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Particle Velocity Buffer 0"),
                contents: bytemuck::cast_slice(&vel_data),
                usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
            });
            let v1 = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Particle Velocity Buffer 1"),
                contents: bytemuck::cast_slice(&vel_data),
                usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
            });
            (v0, v1)
        };

        // Create interaction matrix buffer
        let interaction_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Interaction Matrix Buffer"),
            contents: bytemuck::cast_slice(&interaction_matrix.data),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        });

        // Create min radius buffer
        let min_radius_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Min Radius Buffer"),
            contents: bytemuck::cast_slice(&radius_matrix.min_radius),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        });

        // Create max radius buffer
        let max_radius_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Max Radius Buffer"),
            contents: bytemuck::cast_slice(&radius_matrix.max_radius),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        });

        // Create simulation params uniform buffer
        let params = SimParamsUniform::from_config(config, 1.0 / 60.0);
        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Simulation Params Buffer"),
            contents: bytemuck::bytes_of(&params),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        // Create color palette buffer
        let colors_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Colors Buffer"),
            contents: bytemuck::cast_slice(colors),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        });

        Self {
            pos_type: [pt0, pt1],
            velocities: [vel_buffer_0, vel_buffer_1],
            current_buffer: 0,
            interaction_matrix: interaction_buffer,
            min_radius: min_radius_buffer,
            max_radius: max_radius_buffer,
            params: params_buffer,
            colors: colors_buffer,
            num_particles,
            num_types,
            use_f16,
        }
    }

    /// Get the current pos/type buffer (for reading/rendering).
    pub fn current_pos_type(&self) -> &Buffer {
        &self.pos_type[self.current_buffer]
    }

    /// Get the next pos/type buffer (for writing in compute).
    pub fn next_pos_type(&self) -> &Buffer {
        &self.pos_type[1 - self.current_buffer]
    }

    /// Get the current velocity buffer.
    pub fn current_velocities(&self) -> &Buffer {
        &self.velocities[self.current_buffer]
    }

    /// Get the next velocity buffer.
    pub fn next_velocities(&self) -> &Buffer {
        &self.velocities[1 - self.current_buffer]
    }

    /// Swap the particle buffers after compute pass.
    pub fn swap_buffers(&mut self) {
        self.current_buffer = 1 - self.current_buffer;
    }

    /// Update both particle buffers with new data.
    pub fn update_particles(&self, queue: &Queue, particles: &[Particle]) {
        let pos_type_data: Vec<ParticlePosType> =
            particles.iter().map(ParticlePosType::from).collect();
        let pos_type_bytes = bytemuck::cast_slice(&pos_type_data);

        queue.write_buffer(&self.pos_type[0], 0, pos_type_bytes);
        queue.write_buffer(&self.pos_type[1], 0, pos_type_bytes);

        if self.use_f16 {
            let vel_data: Vec<ParticleVelHalf> =
                particles.iter().map(ParticleVelHalf::from).collect();
            let vel_bytes = bytemuck::cast_slice(&vel_data);
            queue.write_buffer(&self.velocities[0], 0, vel_bytes);
            queue.write_buffer(&self.velocities[1], 0, vel_bytes);
        } else {
            let vel_data: Vec<ParticleVel> = particles.iter().map(ParticleVel::from).collect();
            let vel_bytes = bytemuck::cast_slice(&vel_data);
            queue.write_buffer(&self.velocities[0], 0, vel_bytes);
            queue.write_buffer(&self.velocities[1], 0, vel_bytes);
        }
    }

    /// Update interaction matrix buffer.
    pub fn update_interaction_matrix(&self, queue: &Queue, matrix: &InteractionMatrix) {
        queue.write_buffer(
            &self.interaction_matrix,
            0,
            bytemuck::cast_slice(&matrix.data),
        );
    }

    /// Update radius matrices.
    pub fn update_radius_matrix(&self, queue: &Queue, matrix: &RadiusMatrix) {
        queue.write_buffer(
            &self.min_radius,
            0,
            bytemuck::cast_slice(&matrix.min_radius),
        );
        queue.write_buffer(
            &self.max_radius,
            0,
            bytemuck::cast_slice(&matrix.max_radius),
        );
    }

    /// Update simulation parameters uniform.
    pub fn update_params(&self, queue: &Queue, config: &SimulationConfig, dt: f32) {
        let params = SimParamsUniform::from_config(config, dt);
        queue.write_buffer(&self.params, 0, bytemuck::bytes_of(&params));
    }

    /// Update color palette buffer.
    pub fn update_colors(&self, queue: &Queue, colors: &[[f32; 4]]) {
        queue.write_buffer(&self.colors, 0, bytemuck::cast_slice(colors));
    }

    /// Read particles back from GPU (for debugging or saving).
    ///
    /// Note: This blocks until the GPU is done.
    pub fn read_particles(&self, device: &Device, queue: &Queue) -> Vec<Particle> {
        let num = self.num_particles as usize;

        let pos_type_size = num * std::mem::size_of::<ParticlePosType>();

        let vel_size = if self.use_f16 {
            num * std::mem::size_of::<ParticleVelHalf>()
        } else {
            num * std::mem::size_of::<ParticleVel>()
        };

        // Create staging buffers for readback
        let staging_pos_type = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Particle PosType Staging Buffer"),
            size: pos_type_size as u64,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let staging_vel = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Particle Vel Staging Buffer"),
            size: vel_size as u64,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Copy from particle buffers to staging
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Particle Readback Encoder"),
        });
        encoder.copy_buffer_to_buffer(
            self.current_pos_type(),
            0,
            &staging_pos_type,
            0,
            pos_type_size as u64,
        );
        encoder.copy_buffer_to_buffer(
            self.current_velocities(),
            0,
            &staging_vel,
            0,
            vel_size as u64,
        );
        queue.submit(std::iter::once(encoder.finish()));

        // Map the staging buffers
        let slice_pos = staging_pos_type.slice(..);
        let slice_vel = staging_vel.slice(..);

        let (tx, rx) = std::sync::mpsc::channel();
        let tx2 = tx.clone();

        slice_pos.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });
        slice_vel.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx2.send(result);
        });

        device.poll(wgpu::PollType::wait_indefinitely()).unwrap();

        // Wait for both mappings
        rx.recv().unwrap().unwrap();
        rx.recv().unwrap().unwrap();

        // Read the data
        let data_pos = slice_pos.get_mapped_range();
        let data_vel = slice_vel.get_mapped_range();

        let pos_types: &[ParticlePosType] = bytemuck::cast_slice(&data_pos);
        let mut particles = Vec::with_capacity(num);

        if self.use_f16 {
            let vels: &[ParticleVelHalf] = bytemuck::cast_slice(&data_vel);

            for i in 0..num {
                particles.push(Particle {
                    x: pos_types[i].x,
                    y: pos_types[i].y,
                    vx: vels[i].vx.to_f32(),
                    vy: vels[i].vy.to_f32(),
                    particle_type: pos_types[i].particle_type,
                    _padding1: [0; 3],
                    _padding2: [0; 4],
                });
            }
        } else {
            let vels: &[ParticleVel] = bytemuck::cast_slice(&data_vel);

            for i in 0..num {
                particles.push(Particle {
                    x: pos_types[i].x,
                    y: pos_types[i].y,
                    vx: vels[i].vx,
                    vy: vels[i].vy,
                    particle_type: pos_types[i].particle_type,
                    _padding1: [0; 3],
                    _padding2: [0; 4],
                });
            }
        }

        drop(data_pos);
        drop(data_vel);
        staging_pos_type.unmap();
        staging_vel.unmap();

        particles
    }
}

/// Manages render-specific GPU buffers.
pub struct RenderBuffers {
    /// Vertex buffer for fullscreen quad (for post-processing).
    pub fullscreen_quad: Buffer,
}

impl RenderBuffers {
    /// Create render buffers.
    pub fn new(device: &Device) -> Self {
        // Fullscreen quad vertices (two triangles)
        // Format: [x, y, u, v]
        #[rustfmt::skip]
        let vertices: [[f32; 4]; 6] = [
            // Triangle 1
            [-1.0, -1.0, 0.0, 1.0],
            [ 1.0, -1.0, 1.0, 1.0],
            [-1.0,  1.0, 0.0, 0.0],
            // Triangle 2
            [-1.0,  1.0, 0.0, 0.0],
            [ 1.0, -1.0, 1.0, 1.0],
            [ 1.0,  1.0, 1.0, 0.0],
        ];

        let fullscreen_quad = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Fullscreen Quad Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::VERTEX,
        });

        Self { fullscreen_quad }
    }
}

/// Manages GPU buffers for spatial hashing optimization.
///
/// The spatial hash divides the world into a grid of cells (bins).
/// Particles are counted and sorted by bin for efficient neighbor lookup.
pub struct SpatialHashBuffers {
    /// Count/offset buffer A for ping-pong prefix sum.
    /// Size: total_bins + 1 (extra element for end offset).
    pub bin_counts_a: Buffer,
    /// Count/offset buffer B for ping-pong prefix sum.
    pub bin_counts_b: Buffer,
    /// Spatial parameters uniform buffer.
    pub params: Buffer,
    /// Total bins uniform (for clear shader).
    pub total_bins_uniform: Buffer,
    /// Prefix sum step size uniform (legacy, kept for compatibility).
    pub step_size_uniform: Buffer,
    /// Pre-allocated step size uniforms for each prefix sum pass.
    /// This allows all passes to be added to a single encoder without
    /// having to submit between passes for uniform updates.
    pub step_size_uniforms: Vec<Buffer>,
    /// Current spatial parameters.
    pub spatial_params: SpatialParamsUniform,
    /// Which buffer has the current prefix sum result (0 = A, 1 = B).
    pub current_offset_buffer: usize,
}

impl SpatialHashBuffers {
    /// Create spatial hash buffers.
    pub fn new(device: &Device, config: &SimulationConfig, max_radius: f32) -> Self {
        let spatial_params = SpatialParamsUniform::from_config(config, max_radius);
        let total_bins = spatial_params.total_bins();

        // +1 for the extra end offset element
        let bin_buffer_size = ((total_bins + 1) as usize) * std::mem::size_of::<u32>();

        // Create bin count/offset buffers for ping-pong
        let bin_counts_a = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Bin Counts Buffer A"),
            size: bin_buffer_size as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let bin_counts_b = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Bin Counts Buffer B"),
            size: bin_buffer_size as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // Spatial params uniform
        let params = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Spatial Params Buffer"),
            contents: bytemuck::bytes_of(&spatial_params),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        // Total bins uniform for clear shader
        let total_bins_uniform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Total Bins Uniform"),
            contents: bytemuck::bytes_of(&(total_bins + 1)), // +1 for end offset
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        // Step size uniform for prefix sum (legacy)
        let step_size_uniform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Step Size Uniform"),
            contents: bytemuck::bytes_of(&1u32),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        // Pre-allocate step size uniforms for all possible prefix sum passes.
        // We allocate for the maximum (32 passes) to handle any cell size changes
        // without needing to reallocate buffers. Each buffer is just 4 bytes.
        const MAX_PREFIX_PASSES: u32 = 32;
        let step_size_uniforms: Vec<Buffer> = (0..MAX_PREFIX_PASSES)
            .map(|pass_idx| {
                let step_size = 1u32 << pass_idx;
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("Step Size Uniform Pass {}", pass_idx)),
                    contents: bytemuck::bytes_of(&step_size),
                    usage: BufferUsages::UNIFORM,
                })
            })
            .collect();

        Self {
            bin_counts_a,
            bin_counts_b,
            params,
            total_bins_uniform,
            step_size_uniform,
            step_size_uniforms,
            spatial_params,
            current_offset_buffer: 0,
        }
    }

    /// Get the current bin offset buffer (result of prefix sum).
    pub fn current_offsets(&self) -> &Buffer {
        if self.current_offset_buffer == 0 {
            &self.bin_counts_a
        } else {
            &self.bin_counts_b
        }
    }

    /// Update spatial parameters.
    pub fn update_params(&mut self, queue: &Queue, config: &SimulationConfig, max_radius: f32) {
        self.spatial_params = SpatialParamsUniform::from_config(config, max_radius);
        queue.write_buffer(&self.params, 0, bytemuck::bytes_of(&self.spatial_params));

        let total_bins = self.spatial_params.total_bins() + 1;
        queue.write_buffer(&self.total_bins_uniform, 0, bytemuck::bytes_of(&total_bins));
    }

    /// Update step size for prefix sum pass.
    pub fn update_step_size(&self, queue: &Queue, step_size: u32) {
        queue.write_buffer(&self.step_size_uniform, 0, bytemuck::bytes_of(&step_size));
    }

    /// Get total number of bins (including end offset element).
    pub fn total_bins_with_end(&self) -> u32 {
        self.spatial_params.total_bins() + 1
    }

    /// Calculate number of prefix sum passes needed.
    pub fn prefix_sum_passes(&self) -> u32 {
        let total = self.total_bins_with_end();
        // ceil(log2(total))
        32 - total.leading_zeros()
    }

    /// Read bin counts/offsets buffer back from GPU for debugging.
    pub fn read_bin_counts(&self, device: &Device, queue: &Queue, use_buffer_a: bool) -> Vec<u32> {
        let buffer = if use_buffer_a {
            &self.bin_counts_a
        } else {
            &self.bin_counts_b
        };
        let size = self.total_bins_with_end() as usize * std::mem::size_of::<u32>();

        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Bin Counts Staging Buffer"),
            size: size as u64,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Bin Counts Readback Encoder"),
        });
        encoder.copy_buffer_to_buffer(buffer, 0, &staging, 0, size as u64);
        queue.submit(std::iter::once(encoder.finish()));

        let buffer_slice = staging.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });
        device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        rx.recv().unwrap().unwrap();

        let data = buffer_slice.get_mapped_range();
        let counts: Vec<u32> = bytemuck::cast_slice(&data).to_vec();
        drop(data);
        staging.unmap();

        counts
    }
}

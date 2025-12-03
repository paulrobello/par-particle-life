//! Simulation module containing core physics and data structures.

mod boundary;
mod game_of_life;
mod particle;
mod physics;
mod spatial_hash;

pub use boundary::BoundaryMode;
pub use game_of_life::GameOfLife;
pub use particle::{
    InteractionMatrix, Particle, ParticlePosType, ParticlePosTypeHalf, ParticleVel,
    ParticleVelHalf, RadiusMatrix,
};
pub use physics::{PhysicsEngine, advance_particles, compute_forces_cpu};
pub use spatial_hash::SpatialHash;

use serde::{Deserialize, Serialize};

/// Configuration for the particle life simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    /// Number of particles in the simulation (16 - 1,048,576).
    pub num_particles: u32,

    /// Number of particle types/species (1 - 16).
    pub num_types: u32,

    /// Force scaling factor (0.1 - 10.0). Higher values reduce force magnitude.
    pub force_factor: f32,

    /// Friction coefficient (0.0 - 1.0). Applied each frame to slow particles.
    pub friction: f32,

    /// Repulsion strength at close range (0.01 - 4.0).
    pub repel_strength: f32,

    /// Maximum velocity magnitude. Particles are clamped to this speed.
    pub max_velocity: f32,

    /// Boundary handling mode.
    pub boundary_mode: BoundaryMode,

    /// Wall repulsion strength for Repel boundary mode (0.0 - 100.0).
    pub wall_repel_strength: f32,

    /// Number of mirror copies for MirrorWrap mode (5 or 9).
    pub mirror_wrap_count: u32,

    /// World size in pixels.
    pub world_size: glam::Vec2,

    /// Enable 3D simulation with depth.
    pub enable_3d: bool,

    /// Maximum depth for 3D mode.
    pub depth_limit: f32,

    /// Particle render size in pixels.
    pub particle_size: f32,

    /// Enable glow effect on particles.
    pub enable_glow: bool,

    /// Glow effect intensity (0.0 - 2.0).
    pub glow_intensity: f32,

    /// Glow effect size multiplier.
    pub glow_size: f32,

    /// Glow falloff steepness (1.0 - 4.0). Higher = sharper edge.
    pub glow_steepness: f32,

    /// Use spatial hashing for force calculation optimization.
    pub use_spatial_hash: bool,

    /// Spatial hash cell size. Should be >= max interaction radius.
    pub spatial_hash_cell_size: f32,

    /// Maximum number of particles in a single bin before force scaling occurs.
    #[serde(default = "default_max_bin_density")]
    pub max_bin_density: f32,

    /// Maximum neighbors to check per particle (0 = unlimited).
    /// Setting a budget prevents slowdown when particles cluster heavily.
    #[serde(default)]
    pub neighbor_budget: u32,

    /// Background color [r, g, b] in 0.0-1.0 range.
    pub background_color: [f32; 3],
}

/// Default value for max_bin_density (used by serde).
fn default_max_bin_density() -> f32 {
    5000.0
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            num_particles: 64_000,
            num_types: 7,
            force_factor: 1.0,
            friction: 0.3,
            repel_strength: 3.0, // Increased to discourage clustering
            max_velocity: 500.0,
            boundary_mode: BoundaryMode::Wrap,
            wall_repel_strength: 100.0,
            mirror_wrap_count: 5,
            world_size: glam::Vec2::new(1920.0, 1080.0),
            enable_3d: false,
            depth_limit: 420.0,
            particle_size: 0.5,
            enable_glow: true,
            glow_intensity: 0.35,
            glow_size: 4.0,
            glow_steepness: 2.0,
            // Spatial hash enabled for debugging
            use_spatial_hash: true,
            spatial_hash_cell_size: 64.0,
            background_color: [0.0, 0.0, 0.0], // Black
            max_bin_density: 5000.0,
            neighbor_budget: 0, // 0 = unlimited (default), set non-zero to cap iterations in dense clusters
        }
    }
}

impl SimulationConfig {
    /// Create a configuration suitable for GPU rendering.
    pub fn gpu_defaults() -> Self {
        Self::default()
    }

    /// Validate the configuration and return errors if invalid.
    pub fn validate(&self) -> Result<(), String> {
        if self.num_particles == 0 {
            return Err("num_particles must be greater than 0".to_string());
        }
        if self.num_types == 0 || self.num_types > 16 {
            return Err("num_types must be between 1 and 16".to_string());
        }
        if self.force_factor <= 0.0 {
            return Err("force_factor must be positive".to_string());
        }
        if !(0.0..=1.0).contains(&self.friction) {
            return Err("friction must be between 0.0 and 1.0".to_string());
        }
        if self.repel_strength < 0.0 {
            return Err("repel_strength must be non-negative".to_string());
        }
        if self.world_size.x <= 0.0 || self.world_size.y <= 0.0 {
            return Err("world_size must have positive dimensions".to_string());
        }
        Ok(())
    }
}

//! Main application state.

use anyhow::Result;
use winit::event_loop::{ControlFlow, EventLoop};

use super::{AppConfig, handler::AppHandler};
use crate::generators::{
    colors::{Color, PaletteType, generate_colors},
    positions::{PositionPattern, SpawnConfig, generate_positions},
    rules::{RuleType, generate_rules},
};
use crate::simulation::{
    InteractionMatrix, Particle, PhysicsEngine, RadiusMatrix, SimulationConfig,
};

/// Main application state.
pub struct App {
    /// Application configuration.
    pub config: AppConfig,
    /// Simulation configuration.
    pub sim_config: SimulationConfig,
    /// Particle data.
    pub particles: Vec<Particle>,
    /// Interaction matrix.
    pub interaction_matrix: InteractionMatrix,
    /// Radius matrices.
    pub radius_matrix: RadiusMatrix,
    /// Color palette for particle types.
    pub colors: Vec<Color>,
    /// Physics engine.
    pub physics: PhysicsEngine,
    /// Is simulation running?
    pub running: bool,
    /// Current rule type.
    pub current_rule: RuleType,
    /// Current palette type.
    pub current_palette: PaletteType,
    /// Current position pattern.
    pub current_pattern: PositionPattern,
    /// Auto-scale radii with density (persisted setting).
    pub auto_scale_radii: bool,
}

impl App {
    /// Create a new application with default settings.
    pub fn new(reset_config: bool) -> Self {
        let config = if reset_config {
            AppConfig::default()
        } else {
            AppConfig::load()
        };
        let auto_scale_radii = config.auto_scale_radii;

        let mut sim_config = SimulationConfig {
            num_particles: config.sim_num_particles,
            num_types: config.sim_num_types,
            force_factor: config.phys_force_factor,
            friction: config.phys_friction,
            repel_strength: config.phys_repel_strength,
            max_velocity: config.phys_max_velocity,
            boundary_mode: config.phys_boundary_mode,
            wall_repel_strength: config.phys_wall_repel_strength,
            mirror_wrap_count: config.phys_mirror_wrap_count,
            particle_size: config.render_particle_size,
            background_color: config.render_background_color,
            enable_glow: config.render_glow_enabled,
            glow_intensity: config.render_glow_intensity,
            glow_size: config.render_glow_size,
            glow_steepness: config.render_glow_steepness,
            spatial_hash_cell_size: config.render_spatial_hash_cell_size,
            use_spatial_hash: true, // always on
            ..SimulationConfig::default()
        };
        // Enforce current max particle size limit
        sim_config.particle_size = sim_config.particle_size.min(2.0);

        let num_types = sim_config.num_types as usize;

        let current_rule = config.gen_rule;
        let current_palette = config.gen_palette;
        let current_pattern = config.gen_pattern;

        let interaction_matrix = generate_rules(current_rule, num_types);
        let mut radius_matrix = RadiusMatrix::default_for_size(num_types);
        let colors = generate_colors(current_palette, num_types);

        let spawn_config = SpawnConfig {
            num_particles: sim_config.num_particles as usize,
            num_types,
            width: sim_config.world_size.x,
            height: sim_config.world_size.y,
        };
        // Scale radii to keep neighbor counts reasonable as particle density changes.
        if auto_scale_radii {
            Self::rebalance_radii_for_density_static(
                &mut radius_matrix,
                sim_config.num_particles,
                sim_config.world_size,
            );
            let max_r = radius_matrix.max_interaction_radius();
            sim_config.spatial_hash_cell_size = sim_config.spatial_hash_cell_size.max(max_r);
        }

        let particles = generate_positions(current_pattern, &spawn_config);

        let physics = PhysicsEngine::new(particles.len());

        Self {
            config,
            sim_config,
            particles,
            interaction_matrix,
            radius_matrix,
            colors,
            physics,
            running: true,
            current_rule,
            current_palette,
            current_pattern,
            auto_scale_radii,
        }
    }

    /// Run the main application loop.
    pub fn run(reset_config: bool) -> Result<()> {
        log::info!("Par Particle Life starting...");

        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(ControlFlow::Poll);

        let mut app_handler = AppHandler::new(reset_config);
        event_loop.run_app(&mut app_handler)?;

        Ok(())
    }

    /// Advance the simulation by one timestep.
    pub fn step(&mut self, dt: f32) {
        if !self.running {
            return;
        }

        self.physics.step(
            &mut self.particles,
            &self.interaction_matrix,
            &self.radius_matrix,
            &self.sim_config,
            dt,
        );
    }

    /// Regenerate particles with the current pattern.
    pub fn regenerate_particles(&mut self) {
        let spawn_config = SpawnConfig {
            num_particles: self.sim_config.num_particles as usize,
            num_types: self.sim_config.num_types as usize,
            width: self.sim_config.world_size.x,
            height: self.sim_config.world_size.y,
        };
        self.particles = generate_positions(self.current_pattern, &spawn_config);
        self.physics.resize(self.particles.len());
    }

    /// Regenerate the interaction matrix with the current rule type.
    pub fn regenerate_rules(&mut self) {
        self.interaction_matrix =
            generate_rules(self.current_rule, self.sim_config.num_types as usize);
    }

    /// Regenerate the color palette.
    pub fn regenerate_colors(&mut self) {
        self.colors = generate_colors(self.current_palette, self.sim_config.num_types as usize);
    }

    /// Toggle simulation running state.
    pub fn toggle_running(&mut self) {
        self.running = !self.running;
    }

    /// Get colors as RGBA f32 arrays for GPU.
    pub fn colors_as_rgba(&self) -> Vec<[f32; 4]> {
        // Color is already [f32; 4], just clone
        self.colors.clone()
    }

    /// Scale min/max interaction radii so neighbor counts stay roughly constant.
    /// We target a fixed expected neighbor count per particle by adjusting radii
    /// based on density (density * pi * r^2).
    pub(crate) fn rebalance_radii_for_density(&mut self) {
        if !self.auto_scale_radii {
            return;
        }

        Self::rebalance_radii_for_density_static(
            &mut self.radius_matrix,
            self.sim_config.num_particles,
            self.sim_config.world_size,
        );

        // Keep spatial hash cell size in sync with new max radius
        let max_r = self.radius_matrix.max_interaction_radius();
        self.sim_config.spatial_hash_cell_size = self.sim_config.spatial_hash_cell_size.max(max_r);
        self.config.render_spatial_hash_cell_size = self.sim_config.spatial_hash_cell_size;
    }

    /// Static helper so we can reuse during construction before self exists.
    fn rebalance_radii_for_density_static(
        radius_matrix: &mut RadiusMatrix,
        num_particles: u32,
        world_size: glam::Vec2,
    ) {
        const TARGET_NEIGHBORS: f32 = 350.0;
        const MIN_SCALE: f32 = 0.25;
        const MAX_SCALE: f32 = 1.5;

        let area = world_size.x * world_size.y;
        if area <= 0.0 || num_particles == 0 {
            return;
        }

        let density = num_particles as f32 / area;
        let r_ref = radius_matrix.max_interaction_radius();
        let current_neighbors = density * std::f32::consts::PI * r_ref * r_ref;
        if current_neighbors <= 0.0 {
            return;
        }

        let mut scale = (TARGET_NEIGHBORS / current_neighbors).sqrt();
        scale = scale.clamp(MIN_SCALE, MAX_SCALE);

        let clamp_min = 2.0;
        let clamp_max = 512.0;

        for (min_r, max_r) in radius_matrix
            .min_radius
            .iter_mut()
            .zip(radius_matrix.max_radius.iter_mut())
        {
            *min_r = (*min_r * scale).clamp(clamp_min, clamp_max);
            *max_r = (*max_r * scale).clamp(*min_r + 0.5, clamp_max * 2.0);
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new(false) // Default implies not resetting config
    }
}

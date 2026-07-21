//! Main application state.

use super::AppConfig;
use crate::generators::{
    colors::{Color, CustomPalette, PaletteType, generate_colors},
    custom::CustomGenerator,
    positions::{PositionPattern, SpawnConfig, generate_positions},
    rules::{RuleType, generate_rules},
};
use crate::simulation::{
    InteractionMatrix, MatrixVariationConfig, Obstacle, Particle, RadiusMatrix, SimulationConfig,
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
    /// Preserved base matrix used by time-varying matrix transforms.
    pub matrix_variation_base: InteractionMatrix,
    /// Time-varying interaction matrix settings.
    pub matrix_variation: MatrixVariationConfig,
    /// Elapsed variation time in seconds.
    pub matrix_variation_time: f32,
    /// Radius matrices.
    pub radius_matrix: RadiusMatrix,
    /// Color palette for particle types.
    pub colors: Vec<Color>,
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
    /// Per-type mass (higher = slower response to forces).
    pub type_masses: Vec<f32>,
    /// Per-type size multiplier on global particle_size.
    pub type_sizes: Vec<f32>,
    /// Obstacle zones that deflect particles.
    pub obstacles: Vec<Obstacle>,
    /// User-defined custom rule generators.
    pub custom_generators: Vec<CustomGenerator>,
    /// User-defined custom color palettes.
    pub custom_palettes: Vec<CustomPalette>,
    /// Active custom palette, if one is selected or being edited.
    pub active_custom_palette: Option<CustomPalette>,
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
            temperature: config.phys_temperature,
            time_scale: config.phys_time_scale,
            velocity_coupling: config.phys_velocity_coupling,
            integration_method: config.phys_integration_method,
            ..SimulationConfig::default()
        };
        // Enforce current max particle size limit
        sim_config.particle_size = sim_config.particle_size.min(2.0);

        let num_types = sim_config.num_types as usize;

        let current_rule = config.gen_rule;
        let current_palette = config.gen_palette;
        let current_pattern = config.gen_pattern;

        let interaction_matrix = generate_rules(current_rule, num_types);
        let matrix_variation_base = interaction_matrix.clone();
        let matrix_variation = MatrixVariationConfig {
            enabled: config.gen_matrix_variation_enabled,
            mode: config.gen_matrix_variation_mode,
            amplitude: config.gen_matrix_variation_amplitude,
            speed: config.gen_matrix_variation_speed,
        };
        let matrix_variation_time = 0.0;
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

        let type_masses = vec![1.0; num_types];
        let type_sizes = vec![1.0; num_types];
        let obstacles = Vec::new();
        let custom_generators = CustomGenerator::list().unwrap_or_else(|e| {
            log::warn!("Failed to load custom generators: {e}");
            Vec::new()
        });
        let custom_palettes = CustomPalette::list().unwrap_or_else(|e| {
            log::warn!("Failed to load custom palettes: {e}");
            Vec::new()
        });
        let active_custom_palette = None;

        Self {
            config,
            sim_config,
            particles,
            interaction_matrix,
            matrix_variation_base,
            matrix_variation,
            matrix_variation_time,
            radius_matrix,
            colors,
            running: true,
            current_rule,
            current_palette,
            current_pattern,
            auto_scale_radii,
            type_masses,
            type_sizes,
            obstacles,
            custom_generators,
            custom_palettes,
            active_custom_palette,
        }
    }

    /// Build a persisted-config snapshot from the current runtime state.
    ///
    /// Captures every field that should survive a close/restart: UI/window
    /// preferences are preserved from `self.config` (they have no runtime
    /// mirror), while physics/render/generator fields are mirrored from their
    /// authoritative runtime location (`sim_config`, `current_rule`,
    /// `matrix_variation`, etc.).
    ///
    /// This replaces the per-call site hand-mirror blocks (close handler,
    /// preset-apply) that previously forgot to add new fields — the
    /// `temperature` and `velocity_coupling` persistence bug (ARC-009) was
    /// caused by exactly that drift. Adding a new tunable now means adding
    /// one line here instead of editing 3–5 mirror blocks.
    pub fn snapshot_config(&self) -> AppConfig {
        let mut c = self.config.clone();
        c.sim_num_particles = self.sim_config.num_particles;
        c.sim_num_types = self.sim_config.num_types;
        c.phys_force_factor = self.sim_config.force_factor;
        c.phys_friction = self.sim_config.friction;
        c.phys_repel_strength = self.sim_config.repel_strength;
        c.phys_max_velocity = self.sim_config.max_velocity;
        c.phys_boundary_mode = self.sim_config.boundary_mode;
        c.phys_wall_repel_strength = self.sim_config.wall_repel_strength;
        c.phys_mirror_wrap_count = self.sim_config.mirror_wrap_count;
        c.phys_integration_method = self.sim_config.integration_method;
        c.phys_temperature = self.sim_config.temperature;
        c.phys_time_scale = self.sim_config.time_scale;
        c.phys_velocity_coupling = self.sim_config.velocity_coupling;
        c.gen_rule = self.current_rule;
        c.gen_palette = self.current_palette;
        c.gen_pattern = self.current_pattern;
        c.gen_matrix_variation_enabled = self.matrix_variation.enabled;
        c.gen_matrix_variation_mode = self.matrix_variation.mode;
        c.gen_matrix_variation_amplitude = self.matrix_variation.amplitude;
        c.gen_matrix_variation_speed = self.matrix_variation.speed;
        c.render_particle_size = self.sim_config.particle_size;
        c.render_background_color = self.sim_config.background_color;
        c.render_glow_enabled = self.sim_config.enable_glow;
        c.render_glow_intensity = self.sim_config.glow_intensity;
        c.render_glow_size = self.sim_config.glow_size;
        c.render_glow_steepness = self.sim_config.glow_steepness;
        c.render_spatial_hash_cell_size = self.sim_config.spatial_hash_cell_size;
        c
    }

    /// Apply a persisted-config snapshot back into runtime state.
    ///
    /// Inverse of [`snapshot_config`]: writes every field owned by `AppConfig`
    /// into its runtime mirror. Used after loading a config snapshot from a
    /// preset (the preset embeds `sim_config` already; this is for the
    /// surviving `AppConfig`-level preferences on next save).
    ///
    /// [`snapshot_config`]: App::snapshot_config
    pub fn apply_config(&mut self, c: AppConfig) {
        self.config = c.clone();
        self.sim_config.num_particles = c.sim_num_particles;
        self.sim_config.num_types = c.sim_num_types;
        self.sim_config.force_factor = c.phys_force_factor;
        self.sim_config.friction = c.phys_friction;
        self.sim_config.repel_strength = c.phys_repel_strength;
        self.sim_config.max_velocity = c.phys_max_velocity;
        self.sim_config.boundary_mode = c.phys_boundary_mode;
        self.sim_config.wall_repel_strength = c.phys_wall_repel_strength;
        self.sim_config.mirror_wrap_count = c.phys_mirror_wrap_count;
        self.sim_config.integration_method = c.phys_integration_method;
        self.sim_config.temperature = c.phys_temperature;
        self.sim_config.time_scale = c.phys_time_scale;
        self.sim_config.velocity_coupling = c.phys_velocity_coupling;
        self.current_rule = c.gen_rule;
        self.current_palette = c.gen_palette;
        self.current_pattern = c.gen_pattern;
        self.matrix_variation.enabled = c.gen_matrix_variation_enabled;
        self.matrix_variation.mode = c.gen_matrix_variation_mode;
        self.matrix_variation.amplitude = c.gen_matrix_variation_amplitude;
        self.matrix_variation.speed = c.gen_matrix_variation_speed;
        self.sim_config.particle_size = c.render_particle_size;
        self.sim_config.background_color = c.render_background_color;
        self.sim_config.enable_glow = c.render_glow_enabled;
        self.sim_config.glow_intensity = c.render_glow_intensity;
        self.sim_config.glow_size = c.render_glow_size;
        self.sim_config.glow_steepness = c.render_glow_steepness;
        self.sim_config.spatial_hash_cell_size = c.render_spatial_hash_cell_size;
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
    }

    /// Resize the particle list while preserving existing particle state.
    /// New particles are spawned at random zero-velocity positions.
    pub fn set_particle_count_preserving_existing(&mut self, target_count: u32) {
        let target_len = target_count as usize;
        let current_len = self.particles.len();

        if target_len < current_len {
            self.particles.truncate(target_len);
        } else if target_len > current_len {
            let mut rng = rand::rng();
            let additional = target_len - current_len;
            self.particles.reserve(additional);
            for _ in 0..additional {
                self.particles.push(Particle::random_in_world(
                    &mut rng,
                    self.sim_config.world_size.x,
                    self.sim_config.world_size.y,
                    self.sim_config.num_types,
                ));
            }
        }

        self.sim_config.num_particles = target_count;
    }

    /// Regenerate the interaction matrix with the current rule type.
    pub fn regenerate_rules(&mut self) {
        self.matrix_variation_base =
            generate_rules(self.current_rule, self.sim_config.num_types as usize);
        self.interaction_matrix = self
            .matrix_variation
            .apply(&self.matrix_variation_base, self.matrix_variation_time);
    }

    /// Capture the current matrix as the base for future time variation.
    pub fn capture_matrix_variation_base(&mut self) {
        self.matrix_variation_base = self.interaction_matrix.clone();
        self.matrix_variation_time = 0.0;
    }

    /// Update time-varying matrix state. Returns true when the matrix changed.
    pub fn update_matrix_variation(&mut self, dt: f32) -> bool {
        if !self.matrix_variation.enabled {
            return false;
        }
        self.matrix_variation_time += dt;
        self.interaction_matrix = self
            .matrix_variation
            .apply(&self.matrix_variation_base, self.matrix_variation_time);
        true
    }

    /// Regenerate the color palette.
    pub fn regenerate_colors(&mut self) {
        self.colors = if let Some(palette) = &self.active_custom_palette {
            palette.colors_for_types(self.sim_config.num_types as usize)
        } else {
            generate_colors(self.current_palette, self.sim_config.num_types as usize)
        };
    }

    /// Apply a user-defined color palette.
    pub fn apply_custom_palette(&mut self, palette: CustomPalette) {
        self.colors = palette.colors_for_types(self.sim_config.num_types as usize);
        self.active_custom_palette = Some(palette);
    }

    /// Switch back to a built-in color palette.
    pub fn clear_custom_palette(&mut self) {
        self.active_custom_palette = None;
        self.regenerate_colors();
    }

    /// Generate rules from a custom generator, with error handling.
    pub fn generate_custom_rules(&mut self, index: usize) -> Result<InteractionMatrix, String> {
        let num_types = self.sim_config.num_types as usize;
        self.custom_generators
            .get_mut(index)
            .ok_or_else(|| format!("Custom generator index {index} out of range"))?
            .generate(num_types)
            .map_err(|e| e.to_string())
    }

    /// Resize per-type arrays (masses, sizes) to match current num_types.
    pub fn resize_per_type_arrays(&mut self) {
        let n = self.sim_config.num_types as usize;
        self.type_masses.resize(n, 1.0);
        self.type_sizes.resize(n, 1.0);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::IntegrationMethod;

    /// Regression test for the ARC-009 persistence bug: `temperature` and
    /// `velocity_coupling` were omitted from the CloseRequested hand-mirror
    /// block, so both reset to their defaults on every normal quit. With
    /// `snapshot_config`/`apply_config` as the single source of truth, both
    /// fields must survive a close→reopen (snapshot→apply) cycle even when
    /// set to non-default values.
    #[test]
    fn snapshot_apply_round_trip_preserves_temperature_and_velocity_coupling() {
        let mut app = App::new(true);

        // Pick distinctive non-default values that would be caught by an
        // equality check if they silently reverted to the defaults (0.0).
        app.sim_config.temperature = 7.7;
        app.sim_config.velocity_coupling = 0.42;
        // Also exercise a non-float field so the test isn't overly narrow.
        app.sim_config.integration_method = IntegrationMethod::VelocityVerlet;

        // Snapshot the runtime state into the persisted config shape.
        let snapshot = app.snapshot_config();
        assert_eq!(snapshot.phys_temperature, 7.7);
        assert_eq!(snapshot.phys_velocity_coupling, 0.42);
        assert_eq!(
            snapshot.phys_integration_method,
            IntegrationMethod::VelocityVerlet
        );

        // Simulate close→reopen: a fresh App loads the snapshot from disk.
        // Using apply_config pushes the snapshot back into runtime state,
        // mirroring what App::new does inline on startup.
        let mut reopened = App::new(true);
        reopened.apply_config(snapshot);

        assert_eq!(
            reopened.sim_config.temperature, 7.7,
            "temperature must survive snapshot→apply round-trip"
        );
        assert_eq!(
            reopened.sim_config.velocity_coupling, 0.42,
            "velocity_coupling must survive snapshot→apply round-trip"
        );
        assert_eq!(
            reopened.sim_config.integration_method,
            IntegrationMethod::VelocityVerlet,
            "integration_method must survive snapshot→apply round-trip"
        );
    }

    /// `snapshot_config` must be the inverse of `apply_config` for every
    /// persisted field — otherwise the next save silently reverts runtime
    /// edits. Round-trip a fully-populated snapshot through apply→snapshot.
    #[test]
    fn snapshot_apply_round_trip_is_inverse_for_all_persisted_fields() {
        let mut app = App::new(true);
        app.sim_config.temperature = 3.3;
        app.sim_config.velocity_coupling = 0.27;
        app.sim_config.force_factor = 2.5;
        app.sim_config.friction = 0.7;
        app.sim_config.max_velocity = 333.0;

        let first = app.snapshot_config();
        app.apply_config(first.clone());
        let second = app.snapshot_config();

        assert_eq!(first.phys_temperature, second.phys_temperature);
        assert_eq!(first.phys_velocity_coupling, second.phys_velocity_coupling);
        assert_eq!(first.phys_force_factor, second.phys_force_factor);
        assert_eq!(first.phys_friction, second.phys_friction);
        assert_eq!(first.phys_max_velocity, second.phys_max_velocity);
    }
}

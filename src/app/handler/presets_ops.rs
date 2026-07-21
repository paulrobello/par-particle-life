//! Preset save/load operations.

use std::path::Path;

use super::AppHandler;
use crate::app::Preset;
use crate::generators::colors::CustomPalette;

impl AppHandler {
    pub(crate) fn refresh_presets(&mut self) {
        self.preset_list = Preset::list_presets().unwrap_or_default();
    }

    pub(crate) fn refresh_custom_palettes(&mut self) {
        self.app.custom_palettes = CustomPalette::list().unwrap_or_default();
    }

    pub(crate) fn save_custom_palette(&mut self, name: &str) {
        match CustomPalette::new(name, self.app.colors.clone()) {
            Ok(palette) => match CustomPalette::ensure_custom_palettes_dir()
                .and_then(|dir| palette.save_to_dir(dir))
            {
                Ok(()) => {
                    self.app.active_custom_palette = Some(palette.clone());
                    self.preset_status = format!("Saved palette: {}", palette.name);
                    self.refresh_custom_palettes();
                }
                Err(e) => {
                    self.preset_status = format!("Palette save error: {e}");
                    log::error!("Failed to save custom palette: {e}");
                }
            },
            Err(e) => {
                self.preset_status = format!("Palette error: {e}");
            }
        }
    }

    pub(crate) fn load_custom_palette(&mut self, name: &str) {
        let dir = CustomPalette::custom_palettes_dir();
        match CustomPalette::load_from_dir(&dir, name) {
            Ok(palette) => {
                let palette_name = palette.name.clone();
                self.app.apply_custom_palette(palette);
                self.sync_colors();
                self.preset_status = format!("Loaded palette: {palette_name}");
            }
            Err(e) => {
                self.preset_status = format!("Palette load error: {e}");
                log::error!("Failed to load custom palette: {e}");
            }
        }
    }

    pub(crate) fn save_preset(&mut self, name: &str) {
        let preset = Preset::new(
            name,
            &self.app.sim_config,
            &self.app.interaction_matrix,
            &self.app.radius_matrix,
            &self.app.matrix_variation_base,
            self.app.matrix_variation,
            self.app.matrix_variation_time,
            self.app.current_rule,
            self.app.current_palette,
            self.app.active_custom_palette.clone(),
            self.app.current_pattern,
            &self.app.type_masses,
            &self.app.type_sizes,
            &self.app.obstacles,
        );

        match Preset::ensure_presets_dir() {
            Ok(dir) => {
                let path = dir.join(format!("{}.json", name));
                match preset.save_to_file(&path) {
                    Ok(()) => {
                        self.preset_status = format!("Saved: {}", name);
                        self.refresh_presets();
                        log::info!("Saved preset to {}", path.display());
                    }
                    Err(e) => {
                        self.preset_status = format!("Error: {}", e);
                        log::error!("Failed to save preset: {}", e);
                    }
                }
            }
            Err(e) => {
                self.preset_status = format!("Error: {}", e);
                log::error!("Failed to create presets directory: {}", e);
            }
        }
    }

    pub(crate) fn load_preset(&mut self, name: &str) {
        let dir = Preset::presets_dir();
        let path = dir.join(format!("{}.json", name));
        self.load_preset_from_file(&path);
    }

    pub(crate) fn load_dropped_preset(&mut self, path: &Path) {
        if !Preset::is_preset_file(path) {
            self.preset_status = format!("Ignored dropped file: {}", path.display());
            log::info!("Ignored non-preset dropped file: {}", path.display());
            return;
        }

        self.load_preset_from_file(path);
    }

    pub(crate) fn load_preset_from_file(&mut self, path: &Path) {
        match Preset::load_from_file(path) {
            Ok(preset) => {
                // SEC-001 / ARC-016: gate application on validate() + matrix-shape
                // checks before mutating any live state. A crafted or corrupt
                // preset (num_particles=4294967295, mismatched matrix size, NaN
                // values) must surface a clean user-facing error rather than
                // OOM-killing the app or panicking on a bounds check.
                if let Err(e) = preset.validate() {
                    self.preset_status = format!("Validation error: {e}");
                    log::error!("Rejected preset from {}: {e}", path.display());
                    return;
                }
                let display_name = path
                    .file_stem()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_else(|| preset.name.clone());
                self.apply_preset(preset);
                self.preset_status = format!("Loaded: {display_name}");
                log::info!("Loaded preset from {}", path.display());
            }
            Err(e) => {
                self.preset_status = format!("Error: {e}");
                log::error!("Failed to load preset from {}: {e}", path.display());
            }
        }
    }

    fn apply_preset(&mut self, preset: Preset) {
        self.app.sim_config = preset.sim_config;
        self.app.interaction_matrix = preset.interaction_matrix;
        self.app.matrix_variation_base = preset
            .matrix_variation_base
            .unwrap_or_else(|| self.app.interaction_matrix.clone());
        self.app.matrix_variation = preset.matrix_variation;
        self.app.matrix_variation_time = preset.matrix_variation_time;
        self.app.radius_matrix = preset.radius_matrix;
        self.app.current_rule = preset.rule_type;
        self.app.current_palette = preset.palette_type;
        self.app.active_custom_palette = preset.custom_palette;
        self.app.current_pattern = preset.position_pattern;
        self.app.type_masses = preset.type_masses;
        self.app.type_sizes = preset.type_sizes;
        self.app.obstacles = preset.obstacles;

        // Mirror into persisted config so settings survive restart
        self.app.config.sim_num_particles = self.app.sim_config.num_particles;
        self.app.config.sim_num_types = self.app.sim_config.num_types;
        self.app.config.phys_force_factor = self.app.sim_config.force_factor;
        self.app.config.phys_friction = self.app.sim_config.friction;
        self.app.config.phys_repel_strength = self.app.sim_config.repel_strength;
        self.app.config.phys_max_velocity = self.app.sim_config.max_velocity;
        self.app.config.phys_boundary_mode = self.app.sim_config.boundary_mode;
        self.app.config.phys_wall_repel_strength = self.app.sim_config.wall_repel_strength;
        self.app.config.phys_mirror_wrap_count = self.app.sim_config.mirror_wrap_count;
        self.app.config.gen_rule = self.app.current_rule;
        self.app.config.gen_palette = self.app.current_palette;
        self.app.config.gen_pattern = self.app.current_pattern;
        self.app.config.gen_matrix_variation_enabled = self.app.matrix_variation.enabled;
        self.app.config.gen_matrix_variation_mode = self.app.matrix_variation.mode;
        self.app.config.gen_matrix_variation_amplitude = self.app.matrix_variation.amplitude;
        self.app.config.gen_matrix_variation_speed = self.app.matrix_variation.speed;
        self.app.config.render_particle_size = self.app.sim_config.particle_size;
        self.app.config.render_background_color = self.app.sim_config.background_color;
        self.app.config.render_glow_enabled = self.app.sim_config.enable_glow;
        self.app.config.render_glow_intensity = self.app.sim_config.glow_intensity;
        self.app.config.render_glow_size = self.app.sim_config.glow_size;
        self.app.config.render_glow_steepness = self.app.sim_config.glow_steepness;
        self.app.config.render_spatial_hash_cell_size = self.app.sim_config.spatial_hash_cell_size;
        self.app.config.phys_temperature = self.app.sim_config.temperature;
        self.app.config.phys_time_scale = self.app.sim_config.time_scale;
        self.app.config.phys_velocity_coupling = self.app.sim_config.velocity_coupling;
        self.app.config.phys_integration_method = self.app.sim_config.integration_method;

        // Regenerate colors from palette
        self.app.regenerate_colors();

        // Regenerate particles from pattern
        let spawn_config = crate::generators::positions::SpawnConfig {
            num_particles: self.app.sim_config.num_particles as usize,
            num_types: self.app.sim_config.num_types as usize,
            width: self.app.sim_config.world_size.x,
            height: self.app.sim_config.world_size.y,
        };
        self.app.particles = crate::generators::positions::generate_positions(
            self.app.current_pattern,
            &spawn_config,
        );

        // Resize physics engine
        self.app.physics.resize(self.app.particles.len());

        // Sync GPU buffers
        self.sync_buffers();
        self.sync_interaction_matrix();
        self.sync_colors();
        self.sync_obstacles();
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::AppHandler;
    use crate::app::Preset;
    use crate::generators::{colors::PaletteType, positions::PositionPattern, rules::RuleType};
    use crate::simulation::{InteractionMatrix, RadiusMatrix, SimulationConfig};

    #[test]
    fn load_preset_from_file_applies_dragged_preset_and_reports_source_name() {
        let dir = std::env::temp_dir().join(format!(
            "par-particle-life-dropped-preset-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create preset test dir");
        let path = dir.join("dropped-preset.json");

        let mut config = SimulationConfig {
            num_types: 3,
            num_particles: 1_000,
            ..SimulationConfig::default()
        };
        config.force_factor = 2.5;
        let matrix = InteractionMatrix::filled(3, 0.25);
        let radii = RadiusMatrix::default_for_size(3);
        let preset = Preset::new(
            "Dragged Preset Name",
            &config,
            &matrix,
            &radii,
            &matrix,
            Default::default(),
            0.0,
            RuleType::Random,
            PaletteType::Rainbow,
            None,
            PositionPattern::Disk,
            &[1.0, 1.0, 1.0],
            &[1.0, 1.0, 1.0],
            &[],
        );
        preset.save_to_file(&path).expect("save preset");

        let mut handler = AppHandler::new(true);
        handler.load_preset_from_file(&path);

        assert_eq!(handler.app.sim_config.num_types, 3);
        assert_eq!(handler.app.sim_config.num_particles, 1_000);
        assert_eq!(handler.app.sim_config.force_factor, 2.5);
        assert_eq!(handler.preset_status, "Loaded: dropped-preset");

        fs::remove_dir_all(&dir).expect("cleanup preset test dir");
    }

    #[test]
    fn load_preset_from_file_rejects_hostile_num_particles_with_validation_error() {
        // SEC-001: a crafted preset with num_particles = u32::MAX must not
        // OOM the app. The validation gate must surface a clean user-facing
        // error and leave the live simulation state untouched.
        let dir = std::env::temp_dir().join(format!(
            "par-particle-life-hostile-preset-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create preset test dir");
        let path = dir.join("hostile-preset.json");

        let mut config = SimulationConfig::default();
        config.num_particles = u32::MAX;
        config.num_types = 3;
        // Matrix shape matches the (invalid) config so the validate() failure
        // is specifically about num_particles, not shape.
        let matrix = InteractionMatrix::filled(3, 0.25);
        let radii = RadiusMatrix::default_for_size(3);
        let preset = Preset::new(
            "Hostile",
            &config,
            &matrix,
            &radii,
            &matrix,
            Default::default(),
            0.0,
            RuleType::Random,
            PaletteType::Rainbow,
            None,
            PositionPattern::Disk,
            &[1.0, 1.0, 1.0],
            &[1.0, 1.0, 1.0],
            &[],
        );
        preset.save_to_file(&path).expect("save hostile preset");

        let mut handler = AppHandler::new(true);
        let original_num_particles = handler.app.sim_config.num_particles;
        handler.load_preset_from_file(&path);

        assert!(
            handler.preset_status.starts_with("Validation error"),
            "expected validation error, got: {}",
            handler.preset_status
        );
        // Live state must be untouched.
        assert_eq!(
            handler.app.sim_config.num_particles, original_num_particles,
            "hostile preset must not mutate live state"
        );

        fs::remove_dir_all(&dir).expect("cleanup preset test dir");
    }

    #[test]
    fn load_preset_from_file_rejects_matrix_shape_mismatch() {
        // A preset whose interaction_matrix.data.len() != size*size currently
        // panics on access; the validation gate must catch it first.
        let dir = std::env::temp_dir().join(format!(
            "par-particle-life-shape-mismatch-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create preset test dir");
        let path = dir.join("shape-mismatch.json");

        let config = SimulationConfig {
            num_types: 3,
            num_particles: 1_000,
            ..SimulationConfig::default()
        };
        // 4x4 matrix attached to a 3-type config.
        let bad_matrix = InteractionMatrix::filled(4, 0.25);
        let radii = RadiusMatrix::default_for_size(3);
        let preset = Preset::new(
            "ShapeMismatch",
            &config,
            &bad_matrix,
            &radii,
            &bad_matrix,
            Default::default(),
            0.0,
            RuleType::Random,
            PaletteType::Rainbow,
            None,
            PositionPattern::Disk,
            &[1.0, 1.0, 1.0],
            &[1.0, 1.0, 1.0],
            &[],
        );
        preset.save_to_file(&path).expect("save mismatched preset");

        let mut handler = AppHandler::new(true);
        handler.load_preset_from_file(&path);
        assert!(
            handler.preset_status.starts_with("Validation error"),
            "expected shape-mismatch validation error, got: {}",
            handler.preset_status
        );

        fs::remove_dir_all(&dir).expect("cleanup preset test dir");
    }
}

//! Preset save/load operations.

use super::AppHandler;
use crate::app::Preset;

impl AppHandler {
    pub(crate) fn refresh_presets(&mut self) {
        self.preset_list = Preset::list_presets().unwrap_or_default();
    }

    pub(crate) fn save_preset(&mut self, name: &str) {
        let preset = Preset::new(
            name,
            &self.app.sim_config,
            &self.app.interaction_matrix,
            &self.app.radius_matrix,
            self.app.current_rule,
            self.app.current_palette,
            self.app.current_pattern,
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

        match Preset::load_from_file(&path) {
            Ok(preset) => {
                // Apply the preset
                self.app.sim_config = preset.sim_config;
                self.app.interaction_matrix = preset.interaction_matrix;
                self.app.radius_matrix = preset.radius_matrix;
                self.app.current_rule = preset.rule_type;
                self.app.current_palette = preset.palette_type;
                self.app.current_pattern = preset.position_pattern;

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
                self.app.config.render_particle_size = self.app.sim_config.particle_size;
                self.app.config.render_background_color = self.app.sim_config.background_color;
                self.app.config.render_glow_enabled = self.app.sim_config.enable_glow;
                self.app.config.render_glow_intensity = self.app.sim_config.glow_intensity;
                self.app.config.render_glow_size = self.app.sim_config.glow_size;
                self.app.config.render_glow_steepness = self.app.sim_config.glow_steepness;
                self.app.config.render_spatial_hash_cell_size =
                    self.app.sim_config.spatial_hash_cell_size;

                // Regenerate colors from palette
                self.app.colors = crate::generators::colors::generate_colors(
                    self.app.current_palette,
                    self.app.sim_config.num_types as usize,
                );

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

                self.preset_status = format!("Loaded: {}", name);
                log::info!("Loaded preset: {}", name);
            }
            Err(e) => {
                self.preset_status = format!("Error: {}", e);
                log::error!("Failed to load preset: {}", e);
            }
        }
    }
}

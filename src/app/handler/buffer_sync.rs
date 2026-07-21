//! Buffer synchronization operations between CPU and GPU.

use super::AppHandler;
use crate::app::AppConfig;
use crate::generators::{
    colors::{PaletteType, generate_colors},
    positions::{PositionPattern, SpawnConfig, generate_positions},
    rules::{RuleType, generate_rules},
};
use crate::renderer::gpu::{SimulationBuffers, SpatialHashBuffers};
use crate::simulation::{BoundaryMode, RadiusMatrix};

impl AppHandler {
    /// Update camera uniform buffer with current zoom and pan.
    pub(crate) fn update_camera(&self) {
        if let Some(gpu) = &self.gpu {
            gpu.render.update_camera_with_zoom(
                &gpu.context.queue,
                self.app.sim_config.world_size.x,
                self.app.sim_config.world_size.y,
                self.camera.zoom,
                self.camera.offset.x,
                self.camera.offset.y,
            );
        }
    }

    pub(crate) fn sync_buffers(&mut self) {
        if let Some(gpu) = &self.gpu {
            // Recreate buffers with new particle count
            let colors_rgba = self.app.colors_as_rgba();
            let new_buffers = SimulationBuffers::new(
                &gpu.context.device,
                &self.app.particles,
                &self.app.interaction_matrix,
                &self.app.radius_matrix,
                &colors_rgba,
                &self.app.type_masses,
                &self.app.type_sizes,
                &self.app.sim_config,
            );

            // Recreate spatial hash buffers
            let max_radius = self.app.radius_matrix.max_interaction_radius();
            let new_spatial_buffers =
                SpatialHashBuffers::new(&gpu.context.device, &self.app.sim_config, max_radius);

            // Update render bind groups
            let new_bind_group = gpu.render.create_render_bind_group(
                &gpu.context.device,
                new_buffers.current_pos_type(),
                &new_buffers,
            );
            let new_glow_bind_group = gpu.render.create_glow_bind_group(
                &gpu.context.device,
                new_buffers.current_pos_type(),
                &new_buffers,
            );
            let new_mirror_bind_group = gpu.render.create_mirror_bind_group(
                &gpu.context.device,
                new_buffers.current_pos_type(),
                &new_buffers,
            );
            let new_infinite_bind_group = gpu.render.create_infinite_bind_group(
                &gpu.context.device,
                new_buffers.current_pos_type(),
                &new_buffers,
            );

            // Replace old buffers (need mutable access)
            if let Some(gpu) = &mut self.gpu {
                gpu.buffers = new_buffers;
                gpu.render_bind_group = new_bind_group;
                gpu.glow_bind_group = new_glow_bind_group;
                gpu.mirror_bind_group = new_mirror_bind_group;
                gpu.infinite_bind_group = new_infinite_bind_group;

                // Sync obstacles to new buffers
                let obs_data: Vec<crate::simulation::ObstacleData> =
                    self.app.obstacles.iter().map(|o| o.into()).collect();
                gpu.buffers.num_obstacles = self.app.obstacles.len() as u32;
                if !obs_data.is_empty() {
                    gpu.buffers.update_obstacles(
                        &gpu.context.queue,
                        &obs_data,
                        gpu.buffers.num_obstacles,
                    );
                }

                // Always invalidate spatial bind groups since they reference sim_buffers
                // which were just recreated above
                gpu.spatial_bind_groups.invalidate();

                // Update spatial hash buffers if cell size changed
                if self.needs_sync_spatial_buffers
                    || new_spatial_buffers.spatial_params.cell_size
                        != gpu.spatial_buffers.spatial_params.cell_size
                {
                    gpu.spatial_buffers = new_spatial_buffers;
                    self.needs_sync_spatial_buffers = false;
                }

                gpu.spatial_bind_groups.ensure(
                    &gpu.context.device,
                    &gpu.buffers,
                    &mut gpu.spatial_buffers,
                    &gpu.spatial_pipelines,
                );
            }
        }
    }

    /// Change active particle count without resetting existing GPU particle state when possible.
    pub(crate) fn hot_swap_particle_count(&mut self, target_count: u32) {
        if target_count == self.app.sim_config.num_particles {
            return;
        }

        let previous_count = self.app.sim_config.num_particles;
        self.app.config.sim_num_particles = target_count;
        self.app.rebalance_radii_for_density();

        let gpu_has_capacity = self
            .gpu
            .as_ref()
            .map(|gpu| gpu.buffers.has_capacity_for(target_count))
            .unwrap_or(false);

        if gpu_has_capacity {
            self.app
                .set_particle_count_preserving_existing(target_count);

            if target_count > previous_count {
                let start = previous_count as usize;
                let appended = &self.app.particles[start..];
                if let Some(gpu) = &self.gpu {
                    gpu.buffers
                        .write_particle_range(&gpu.context.queue, previous_count, appended);
                }
            }

            if let Some(gpu) = &mut self.gpu {
                gpu.buffers.set_num_particles(target_count);
                gpu.buffers
                    .update_params(&gpu.context.queue, &self.app.sim_config, 1.0 / 60.0);
                gpu.spatial_buffers.update_params(
                    &gpu.context.queue,
                    &self.app.sim_config,
                    self.app.radius_matrix.max_interaction_radius(),
                );
            }
        } else {
            self.sync_particles_from_gpu();
            self.app
                .set_particle_count_preserving_existing(target_count);
            self.sync_buffers();
        }
    }

    /// Sync only the spatial hash buffers (when cell size changes).
    /// This is separate from sync_buffers to avoid unnecessary particle buffer recreation.
    pub(crate) fn sync_spatial_buffers(&mut self) {
        if let Some(gpu) = &mut self.gpu {
            let max_radius = self.app.radius_matrix.max_interaction_radius();
            let new_spatial_buffers =
                SpatialHashBuffers::new(&gpu.context.device, &self.app.sim_config, max_radius);

            gpu.spatial_buffers = new_spatial_buffers;
            gpu.spatial_bind_groups.invalidate();

            gpu.spatial_bind_groups.ensure(
                &gpu.context.device,
                &gpu.buffers,
                &mut gpu.spatial_buffers,
                &gpu.spatial_pipelines,
            );

            log::info!(
                "Spatial hash: {} bins, {} prefix sum passes",
                gpu.spatial_buffers.total_bins_with_end(),
                gpu.spatial_buffers.prefix_sum_passes()
            );
        }
    }

    /// Read particles back from GPU to CPU to ensure we have the latest state
    /// before modifying them (e.g. for brush tools).
    pub(crate) fn sync_particles_from_gpu(&mut self) {
        if let Some(gpu) = &self.gpu {
            self.app.particles = gpu
                .buffers
                .read_particles(&gpu.context.device, &gpu.context.queue);
        }
    }

    /// Normalize particle positions based on current boundary mode.
    /// Wraps or clamps particles to be within world bounds.
    pub(crate) fn normalize_particle_positions(&mut self) {
        let width = self.app.sim_config.world_size.x;
        let height = self.app.sim_config.world_size.y;
        let margin = self.app.sim_config.particle_size;

        for particle in &mut self.app.particles {
            match self.app.sim_config.boundary_mode {
                BoundaryMode::Repel => {
                    // Clamp to valid bounds with margin
                    particle.x = particle.x.clamp(margin, width - margin);
                    particle.y = particle.y.clamp(margin, height - margin);
                }
                BoundaryMode::Wrap | BoundaryMode::MirrorWrap | BoundaryMode::InfiniteWrap => {
                    // Wrap to [0, width) and [0, height)
                    particle.x = particle.x.rem_euclid(width);
                    particle.y = particle.y.rem_euclid(height);
                }
            }
        }
    }

    pub(crate) fn sync_interaction_matrix(&mut self) {
        if let Some(gpu) = &self.gpu {
            gpu.buffers
                .update_interaction_matrix(&gpu.context.queue, &self.app.interaction_matrix);
        }
    }

    pub(crate) fn sync_colors(&mut self) {
        if let Some(gpu) = &self.gpu {
            let colors_rgba = self.app.colors_as_rgba();
            gpu.buffers.update_colors(&gpu.context.queue, &colors_rgba);
        }
    }

    pub(crate) fn sync_type_masses(&mut self) {
        if let Some(gpu) = &self.gpu {
            gpu.buffers
                .update_type_masses(&gpu.context.queue, &self.app.type_masses);
        }
    }

    pub(crate) fn sync_type_sizes(&mut self) {
        if let Some(gpu) = &self.gpu {
            gpu.buffers
                .update_type_sizes(&gpu.context.queue, &self.app.type_sizes);
        }
    }

    pub(crate) fn sync_obstacles(&mut self) {
        if let Some(gpu) = &self.gpu {
            let obs_data: Vec<crate::simulation::ObstacleData> =
                self.app.obstacles.iter().map(|o| o.into()).collect();
            let count = obs_data.len() as u32;
            gpu.buffers
                .update_obstacles(&gpu.context.queue, &obs_data, count);
        }
        if let Some(gpu) = &mut self.gpu {
            gpu.buffers.num_obstacles = self.app.obstacles.len() as u32;
        }
    }

    /// Resets all application settings and simulation state to their default values.
    pub(crate) fn reset_to_defaults(&mut self) {
        // Reset AppConfig to default
        self.app.config = AppConfig::default();
        // Save the default config to overwrite the old one
        if let Err(e) = self.app.config.save() {
            log::error!("Failed to save default app config: {}", e);
        }

        // Reset SimulationConfig to default
        self.app.sim_config = crate::simulation::SimulationConfig::default();

        // Update UI open states to match default config
        self.ui_simulation_open = self.app.config.ui_simulation_open;
        self.ui_physics_open = self.app.config.ui_physics_open;
        self.ui_generators_open = self.app.config.ui_generators_open;
        self.ui_interaction_matrix_open = self.app.config.ui_interaction_matrix_open;
        self.ui_brush_tools_open = self.app.config.ui_brush_tools_open;
        self.ui_rendering_open = self.app.config.ui_rendering_open;
        self.ui_presets_open = self.app.config.ui_presets_open;
        self.ui_keyboard_shortcuts_open = self.app.config.ui_keyboard_shortcuts_open;

        // Reset simulation parameters
        let num_types = self.app.sim_config.num_types as usize;
        self.app.interaction_matrix = generate_rules(RuleType::Random, num_types);
        self.app.capture_matrix_variation_base();
        self.app.matrix_variation = Default::default();
        self.app.radius_matrix = RadiusMatrix::default_for_size(num_types);
        self.app.current_rule = RuleType::Random;
        self.app.current_palette = PaletteType::Rainbow;
        self.app.active_custom_palette = None;
        self.app.colors = generate_colors(PaletteType::Rainbow, num_types);
        self.app.current_pattern = PositionPattern::Disk;
        self.app.type_masses = vec![1.0; num_types];
        self.app.type_sizes = vec![1.0; num_types];

        // Regenerate particles with default settings
        let spawn_config = SpawnConfig {
            num_particles: self.app.sim_config.num_particles as usize,
            num_types,
            width: self.app.sim_config.world_size.x,
            height: self.app.sim_config.world_size.y,
        };
        self.app.particles = generate_positions(self.app.current_pattern, &spawn_config);

        // Reset camera and brush state
        self.camera = crate::app::CameraState::default();
        self.brush = crate::app::BrushState::default();

        // Ensure GPU buffers are updated with new state
        self.app.obstacles.clear();
        self.sync_buffers();
        self.update_camera();
        self.sync_interaction_matrix();
        self.sync_colors();

        self.preset_status = "All settings reset to defaults.".to_string();
        log::info!("All settings reset to defaults.");
    }
}

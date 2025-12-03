//! Brush tool operations for particle manipulation.

use rand::Rng;

use super::AppHandler;
use crate::app::BrushTool;
use crate::simulation::{BoundaryMode, Particle};

impl AppHandler {
    /// Draw particles at the brush position.
    /// Adds new particles within the brush radius with random offset.
    pub(crate) fn draw_particles(&mut self) {
        // Sync with GPU first to get current positions
        self.sync_particles_from_gpu();

        let mut rng = rand::rng();
        let num_types = self.app.sim_config.num_types;
        let world_width = self.app.sim_config.world_size.x;
        let world_height = self.app.sim_config.world_size.y;

        // Determine how many particles to spawn this frame
        let spawn_count = self.brush.draw_intensity as usize;

        for _ in 0..spawn_count {
            // Random position within brush radius
            let angle = rng.random::<f32>() * std::f32::consts::TAU;
            let radius = rng.random::<f32>().sqrt() * self.brush.radius;
            let x = self.brush.position.x + angle.cos() * radius;
            let y = self.brush.position.y + angle.sin() * radius;

            // Determine particle type
            let particle_type = if self.brush.draw_type < 0 {
                // Random type
                rng.random_range(0..num_types)
            } else {
                (self.brush.draw_type as u32).min(num_types - 1)
            };

            // Create new particle
            let particle = Particle::new(x, y, particle_type);

            // Add to particles list (will grow buffer on sync)
            self.app.particles.push(particle);
        }

        // Update particle count in sim config
        self.app.sim_config.num_particles = self.app.particles.len() as u32;

        // Mark that buffers need syncing
        self.needs_sync = true;

        // Apply boundary wrapping to newly added particles
        if self.app.sim_config.boundary_mode == BoundaryMode::Wrap
            || self.app.sim_config.boundary_mode == BoundaryMode::MirrorWrap
            || self.app.sim_config.boundary_mode == BoundaryMode::InfiniteWrap
        {
            // Calculate skip offset before borrowing
            let skip_offset = self.app.particles.len() - spawn_count;

            // Wrap particle positions
            for particle in self.app.particles.iter_mut().skip(skip_offset) {
                particle.x = particle.x.rem_euclid(world_width);
                particle.y = particle.y.rem_euclid(world_height);
            }
        }
    }

    /// Erase particles within the brush radius.
    /// Removes particles that fall within the brush area.
    pub(crate) fn erase_particles(&mut self) {
        // Sync with GPU first to get current positions
        self.sync_particles_from_gpu();

        let brush_pos = self.brush.position;
        let brush_radius_sq = self.brush.radius * self.brush.radius;
        let target_type = self.brush.target_type;
        let world_width = self.app.sim_config.world_size.x;
        let world_height = self.app.sim_config.world_size.y;
        let use_wrap = matches!(
            self.app.sim_config.boundary_mode,
            BoundaryMode::Wrap | BoundaryMode::MirrorWrap | BoundaryMode::InfiniteWrap
        );

        let initial_count = self.app.particles.len();

        // Remove particles within brush radius
        self.app.particles.retain(|particle| {
            // Check if particle type matches target (-1 means all types)
            if target_type >= 0 && particle.particle_type != target_type as u32 {
                return true; // Keep particle (doesn't match target type)
            }

            // Calculate distance to brush center
            let mut dx = particle.x - brush_pos.x;
            let mut dy = particle.y - brush_pos.y;

            // Handle wrapping distance
            if use_wrap {
                if dx > world_width * 0.5 {
                    dx -= world_width;
                } else if dx < -world_width * 0.5 {
                    dx += world_width;
                }
                if dy > world_height * 0.5 {
                    dy -= world_height;
                } else if dy < -world_height * 0.5 {
                    dy += world_height;
                }
            }

            let dist_sq = dx * dx + dy * dy;

            // Keep particle if outside brush radius
            dist_sq > brush_radius_sq
        });

        // Check if any particles were removed
        if self.app.particles.len() < initial_count {
            // Update particle count
            self.app.sim_config.num_particles = self.app.particles.len() as u32;
            self.needs_sync = true;
        }
    }

    /// Process brush tools during active use.
    /// Called each frame when brush is active.
    pub(crate) fn process_brush_tools(&mut self) {
        if !self.brush.is_active {
            return;
        }

        match self.brush.tool {
            BrushTool::Draw => self.draw_particles(),
            BrushTool::Erase => self.erase_particles(),
            BrushTool::Attract | BrushTool::Repel => {
                // These are handled by the GPU compute shader
            }
            BrushTool::None => {}
        }
    }
}

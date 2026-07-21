//! Brush tool operations for particle manipulation.

use super::AppHandler;
use crate::app::BrushTool;
use crate::simulation::{BoundaryMode, ObstacleShape};

impl AppHandler {
    /// Draw particles at the brush position using a GPU compute shader.
    pub(crate) fn draw_particles(&mut self) {
        let Some(gpu) = &mut self.gpu else { return };

        let current_count = gpu.buffers.num_particles;
        let capacity = gpu.buffers.capacity_particles;
        if current_count >= capacity {
            self.preset_status = format!("Particle capacity reached ({capacity})");
            return;
        }

        let requested = self.brush.draw_intensity;
        let spawn_count = requested.min(capacity - current_count);
        if spawn_count == 0 {
            return;
        }

        let params = crate::renderer::gpu::SpawnParamsUniform {
            start_index: current_count,
            spawn_count,
            capacity_particles: capacity,
            num_types: self.app.sim_config.num_types.max(1),
            pos_x: self.brush.position.x,
            pos_y: self.brush.position.y,
            radius: self.brush.radius,
            world_width: self.app.sim_config.world_size.x,
            world_height: self.app.sim_config.world_size.y,
            draw_type: self.brush.draw_type,
            frame_counter: self.app.sim_config.frame_counter,
            _padding: 0,
        };

        gpu.brush_pipelines.update_spawn(&gpu.context.queue, params);
        let spawn_bind_group = gpu
            .brush_pipelines
            .create_spawn_bind_group(&gpu.context.device, &gpu.buffers);

        let mut encoder = gpu.context.create_encoder("Particle Spawn Encoder");
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Particle Spawn Pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&gpu.brush_pipelines.spawn_pipeline);
            pass.set_bind_group(0, &spawn_bind_group, &[]);
            pass.dispatch_workgroups(spawn_count.div_ceil(256), 1, 1);
        }
        gpu.context.submit(encoder.finish());

        let new_count = current_count + spawn_count;
        gpu.buffers.set_num_particles(new_count);
        self.app.sim_config.num_particles = new_count;
        self.app.config.sim_num_particles = new_count;
        self.app
            .particles
            .resize(new_count as usize, crate::simulation::Particle::default());
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
            BrushTool::None | BrushTool::Obstacle => {}
        }
    }

    /// Find the obstacle index under the given world position, or -1 if none.
    /// Checks in reverse order so topmost (last-drawn) obstacle wins.
    pub(crate) fn hit_test_obstacle(&self, world_pos: glam::Vec2) -> i32 {
        for (i, obs) in self.app.obstacles.iter().enumerate().rev() {
            match obs.shape {
                ObstacleShape::Circle => {
                    let radius = obs.width / 2.0;
                    let dx = world_pos.x - obs.x;
                    let dy = world_pos.y - obs.y;
                    if dx * dx + dy * dy <= radius * radius {
                        return i as i32;
                    }
                }
                ObstacleShape::Rectangle => {
                    let hw = obs.width / 2.0;
                    let hh = obs.height / 2.0;
                    if (world_pos.x - obs.x).abs() <= hw && (world_pos.y - obs.y).abs() <= hh {
                        return i as i32;
                    }
                }
            }
        }
        -1
    }
}

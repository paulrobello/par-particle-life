//! Physics engine for particle force calculations and movement.

use glam::Vec2;
use rayon::prelude::*;

use super::{
    SimulationConfig,
    boundary::{BoundaryMode, apply_boundary, wrapped_delta},
    particle::{InteractionMatrix, Particle, RadiusMatrix},
    spatial_hash::SpatialHash,
};

/// Physics engine that computes forces and advances the simulation.
pub struct PhysicsEngine {
    /// Cached force vectors for each particle.
    forces: Vec<Vec2>,
    /// Spatial hash for optimized neighbor queries.
    spatial_hash: Option<SpatialHash>,
}

impl PhysicsEngine {
    /// Create a new physics engine sized for the given particle count.
    pub fn new(num_particles: usize) -> Self {
        Self {
            forces: vec![Vec2::ZERO; num_particles],
            spatial_hash: None,
        }
    }

    /// Resize the internal buffers for a new particle count.
    pub fn resize(&mut self, num_particles: usize) {
        self.forces.resize(num_particles, Vec2::ZERO);
    }

    /// Run one physics step: compute forces and advance particles.
    pub fn step(
        &mut self,
        particles: &mut [Particle],
        interaction_matrix: &InteractionMatrix,
        radius_matrix: &RadiusMatrix,
        config: &SimulationConfig,
        dt: f32,
    ) {
        // Build spatial hash if enabled
        if config.use_spatial_hash {
            let cell_size = radius_matrix
                .max_interaction_radius()
                .max(config.spatial_hash_cell_size);
            self.spatial_hash = Some(SpatialHash::build(particles, cell_size, config.world_size));
        } else {
            self.spatial_hash = None;
        }

        // Compute forces (parallel)
        if let Some(ref spatial_hash) = self.spatial_hash {
            compute_forces_spatial(
                particles,
                &mut self.forces,
                interaction_matrix,
                radius_matrix,
                config,
                spatial_hash,
            );
        } else {
            self.forces = compute_forces_cpu(particles, interaction_matrix, radius_matrix, config);
        }

        // Advance particles (parallel)
        advance_particles(particles, &self.forces, config, dt);
    }

    /// Get the computed forces for debugging/visualization.
    pub fn forces(&self) -> &[Vec2] {
        &self.forces
    }
}

/// Compute forces between all particles using brute force O(n²).
///
/// This is the CPU fallback when spatial hashing is disabled or unavailable.
/// Uses Rayon for parallel computation across particles.
pub fn compute_forces_cpu(
    particles: &[Particle],
    interaction_matrix: &InteractionMatrix,
    radius_matrix: &RadiusMatrix,
    config: &SimulationConfig,
) -> Vec<Vec2> {
    let use_wrap = matches!(
        config.boundary_mode,
        BoundaryMode::Wrap | BoundaryMode::MirrorWrap | BoundaryMode::InfiniteWrap
    );

    particles
        .par_iter()
        .enumerate()
        .map(|(i, p)| {
            let mut force = Vec2::ZERO;
            let p_pos = p.position();
            let p_type = p.particle_type as usize;

            for (j, q) in particles.iter().enumerate() {
                if i == j {
                    continue;
                }

                let q_pos = q.position();
                let q_type = q.particle_type as usize;

                // Get delta accounting for world wrapping
                let delta = wrapped_delta(p_pos, q_pos, config.world_size, use_wrap);
                let dist_sq = delta.length_squared();

                // Skip if too far (optimization)
                let max_r = radius_matrix.get_max(p_type, q_type);
                if dist_sq > max_r * max_r {
                    continue;
                }

                let dist = dist_sq.sqrt();
                if dist < 0.0001 {
                    continue; // Avoid division by zero
                }

                let min_r = radius_matrix.get_min(p_type, q_type);
                let direction = delta / dist;

                if dist < min_r {
                    // Close range repulsion
                    let repel_strength = config.repel_strength * (min_r - dist) / min_r;
                    force -= direction * repel_strength;
                } else {
                    // Attraction/repulsion based on interaction matrix
                    let strength = interaction_matrix.get(p_type, q_type);
                    // Linear falloff from min to max radius
                    let t = (dist - min_r) / (max_r - min_r);
                    force += direction * strength * (1.0 - t);
                }
            }

            force / config.force_factor
        })
        .collect()
}

/// Compute forces using spatial hashing for O(n) average case.
fn compute_forces_spatial(
    particles: &[Particle],
    forces: &mut [Vec2],
    interaction_matrix: &InteractionMatrix,
    radius_matrix: &RadiusMatrix,
    config: &SimulationConfig,
    spatial_hash: &SpatialHash,
) {
    let use_wrap = matches!(
        config.boundary_mode,
        BoundaryMode::Wrap | BoundaryMode::MirrorWrap | BoundaryMode::InfiniteWrap
    );

    forces.par_iter_mut().enumerate().for_each(|(i, force)| {
        *force = Vec2::ZERO;

        let p = &particles[i];
        let p_pos = p.position();
        let p_type = p.particle_type as usize;

        // Query nearby particles from spatial hash
        let max_radius = radius_matrix.max_interaction_radius();
        let neighbor_indices =
            spatial_hash.query_radius(p_pos, max_radius, config.world_size, use_wrap);

        for j in neighbor_indices {
            if i == j {
                continue;
            }

            let q = &particles[j];
            let q_pos = q.position();
            let q_type = q.particle_type as usize;

            let delta = wrapped_delta(p_pos, q_pos, config.world_size, use_wrap);
            let dist_sq = delta.length_squared();

            let max_r = radius_matrix.get_max(p_type, q_type);
            if dist_sq > max_r * max_r {
                continue;
            }

            let dist = dist_sq.sqrt();
            if dist < 0.0001 {
                continue;
            }

            let min_r = radius_matrix.get_min(p_type, q_type);
            let direction = delta / dist;

            if dist < min_r {
                let repel_strength = config.repel_strength * (min_r - dist) / min_r;
                *force -= direction * repel_strength;
            } else {
                let strength = interaction_matrix.get(p_type, q_type);
                let t = (dist - min_r) / (max_r - min_r);
                *force += direction * strength * (1.0 - t);
            }
        }

        *force /= config.force_factor;
    });
}

/// Advance all particles based on computed forces.
///
/// Applies:
/// 1. Friction damping to velocities
/// 2. Force integration
/// 3. Velocity clamping
/// 4. Position update
/// 5. Boundary handling
pub fn advance_particles(
    particles: &mut [Particle],
    forces: &[Vec2],
    config: &SimulationConfig,
    dt: f32,
) {
    particles
        .par_iter_mut()
        .zip(forces.par_iter())
        .for_each(|(p, &force)| {
            // Apply friction (damping)
            let friction_factor = 1.0 - config.friction;
            p.vx *= friction_factor;
            p.vy *= friction_factor;

            // Apply force (Euler integration)
            p.vx += force.x * dt;
            p.vy += force.y * dt;

            // Clamp velocity magnitude
            let speed = p.speed();
            if speed > config.max_velocity {
                let scale = config.max_velocity / speed;
                p.vx *= scale;
                p.vy *= scale;
            }

            // Update position
            p.x += p.vx * dt;
            p.y += p.vy * dt;

            // Handle boundaries
            apply_boundary(p, config);
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_particles() -> Vec<Particle> {
        vec![Particle::new(50.0, 50.0, 0), Particle::new(60.0, 50.0, 1)]
    }

    fn make_test_matrix() -> InteractionMatrix {
        let mut m = InteractionMatrix::new(2);
        m.set(0, 1, 1.0); // Type 0 attracted to type 1
        m.set(1, 0, -1.0); // Type 1 repelled by type 0
        m
    }

    fn make_test_radii() -> RadiusMatrix {
        RadiusMatrix::new(2, 5.0, 50.0)
    }

    #[test]
    fn test_force_calculation() {
        let particles = make_test_particles();
        let matrix = make_test_matrix();
        let radii = make_test_radii();
        let config = SimulationConfig {
            force_factor: 1.0,
            repel_strength: 1.0,
            world_size: glam::Vec2::new(100.0, 100.0),
            ..Default::default()
        };

        let forces = compute_forces_cpu(&particles, &matrix, &radii, &config);

        // Particle 0 should be pulled right (towards particle 1)
        assert!(forces[0].x > 0.0);

        // Particle 1 should be pushed right (away from particle 0 which is to the left)
        assert!(forces[1].x > 0.0);
    }

    #[test]
    fn test_particle_advancement() {
        let mut particles = vec![Particle::with_velocity(50.0, 50.0, 1.0, 0.0, 0)];
        let forces = vec![Vec2::ZERO];
        let config = SimulationConfig {
            friction: 0.0,
            max_velocity: 100.0,
            world_size: glam::Vec2::new(100.0, 100.0),
            ..Default::default()
        };

        advance_particles(&mut particles, &forces, &config, 1.0);

        assert!((particles[0].x - 51.0).abs() < 0.001);
    }

    #[test]
    fn test_friction_damping() {
        let mut particles = vec![Particle::with_velocity(50.0, 50.0, 10.0, 0.0, 0)];
        let forces = vec![Vec2::ZERO];
        let config = SimulationConfig {
            friction: 0.5,
            max_velocity: 100.0,
            world_size: glam::Vec2::new(100.0, 100.0),
            ..Default::default()
        };

        advance_particles(&mut particles, &forces, &config, 1.0);

        // Velocity should be halved due to 0.5 friction
        assert!((particles[0].vx - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_velocity_clamping() {
        let mut particles = vec![Particle::with_velocity(50.0, 50.0, 100.0, 0.0, 0)];
        let forces = vec![Vec2::ZERO];
        let config = SimulationConfig {
            friction: 0.0,
            max_velocity: 10.0,
            world_size: glam::Vec2::new(200.0, 200.0),
            ..Default::default()
        };

        advance_particles(&mut particles, &forces, &config, 1.0);

        assert!(particles[0].speed() <= 10.0 + 0.001);
    }
}

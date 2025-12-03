//! Boundary condition handling for the simulation.

use serde::{Deserialize, Serialize};

use super::{Particle, SimulationConfig};

/// Defines how particles interact with world boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum BoundaryMode {
    /// Particles are repelled from walls, bouncing back.
    #[default]
    Repel,

    /// Particles wrap around to the opposite side.
    Wrap,

    /// Particles wrap with mirrored rendering effect.
    MirrorWrap,

    /// Infinite tiling - particles rendered multiple times.
    InfiniteWrap,
}

impl BoundaryMode {
    /// Get all available boundary modes.
    pub fn all() -> &'static [BoundaryMode] {
        &[
            BoundaryMode::Repel,
            BoundaryMode::Wrap,
            BoundaryMode::MirrorWrap,
            BoundaryMode::InfiniteWrap,
        ]
    }

    /// Get the display name for this mode.
    pub fn display_name(&self) -> &'static str {
        match self {
            BoundaryMode::Repel => "Repel (Bounce)",
            BoundaryMode::Wrap => "Wrap Around",
            BoundaryMode::MirrorWrap => "Mirror Wrap",
            BoundaryMode::InfiniteWrap => "Infinite Tiling",
        }
    }
}

/// Apply boundary conditions to a single particle.
pub fn apply_boundary(particle: &mut Particle, config: &SimulationConfig) {
    match config.boundary_mode {
        BoundaryMode::Repel => apply_repel_boundary(particle, config),
        BoundaryMode::Wrap | BoundaryMode::MirrorWrap | BoundaryMode::InfiniteWrap => {
            apply_wrap_boundary(particle, config);
        }
    }
}

/// Apply repel (bounce) boundary conditions.
///
/// Particles that hit the edge are pushed back and their velocity is reversed.
fn apply_repel_boundary(particle: &mut Particle, config: &SimulationConfig) {
    let margin = config.particle_size * 2.0;
    let repel_force = 0.5; // Force applied when hitting boundary

    // Left boundary
    if particle.x < margin {
        particle.x = margin;
        particle.vx = particle.vx.abs() * repel_force;
    }

    // Right boundary
    if particle.x > config.world_size.x - margin {
        particle.x = config.world_size.x - margin;
        particle.vx = -particle.vx.abs() * repel_force;
    }

    // Top boundary
    if particle.y < margin {
        particle.y = margin;
        particle.vy = particle.vy.abs() * repel_force;
    }

    // Bottom boundary
    if particle.y > config.world_size.y - margin {
        particle.y = config.world_size.y - margin;
        particle.vy = -particle.vy.abs() * repel_force;
    }
}

/// Apply wrap-around boundary conditions.
///
/// Particles that exit one side appear on the opposite side.
fn apply_wrap_boundary(particle: &mut Particle, config: &SimulationConfig) {
    // Wrap X
    if particle.x < 0.0 {
        particle.x += config.world_size.x;
    } else if particle.x >= config.world_size.x {
        particle.x -= config.world_size.x;
    }

    // Wrap Y
    if particle.y < 0.0 {
        particle.y += config.world_size.y;
    } else if particle.y >= config.world_size.y {
        particle.y -= config.world_size.y;
    }
}

/// Calculate the shortest distance between two particles considering wrapping.
///
/// Returns the delta vector from `from` to `to` using the shortest path,
/// which may go through a boundary if wrapping is enabled.
pub fn wrapped_delta(
    from: glam::Vec2,
    to: glam::Vec2,
    world_size: glam::Vec2,
    wrap: bool,
) -> glam::Vec2 {
    if !wrap {
        return to - from;
    }

    let mut delta = to - from;

    // Check if wrapping gives a shorter path in X
    if delta.x > world_size.x * 0.5 {
        delta.x -= world_size.x;
    } else if delta.x < -world_size.x * 0.5 {
        delta.x += world_size.x;
    }

    // Check if wrapping gives a shorter path in Y
    if delta.y > world_size.y * 0.5 {
        delta.y -= world_size.y;
    } else if delta.y < -world_size.y * 0.5 {
        delta.y += world_size.y;
    }

    delta
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> SimulationConfig {
        SimulationConfig {
            world_size: glam::Vec2::new(100.0, 100.0),
            particle_size: 2.0,
            ..Default::default()
        }
    }

    #[test]
    fn test_repel_boundary() {
        let mut config = test_config();
        config.boundary_mode = BoundaryMode::Repel;

        let mut p = Particle::new(-10.0, 50.0, 0);
        p.vx = -5.0;
        apply_boundary(&mut p, &config);

        assert!(p.x >= 0.0);
        assert!(p.vx >= 0.0); // Velocity should be reversed
    }

    #[test]
    fn test_wrap_boundary() {
        let mut config = test_config();
        config.boundary_mode = BoundaryMode::Wrap;

        let mut p = Particle::new(110.0, 50.0, 0);
        apply_boundary(&mut p, &config);

        assert!(p.x >= 0.0 && p.x < 100.0);
    }

    #[test]
    fn test_wrapped_delta() {
        let world = glam::Vec2::new(100.0, 100.0);

        // Normal case - no wrapping needed
        let delta = wrapped_delta(
            glam::Vec2::new(10.0, 10.0),
            glam::Vec2::new(20.0, 20.0),
            world,
            true,
        );
        assert!((delta.x - 10.0).abs() < 0.001);
        assert!((delta.y - 10.0).abs() < 0.001);

        // Wrapped case - shorter to go through boundary
        let delta = wrapped_delta(
            glam::Vec2::new(90.0, 50.0),
            glam::Vec2::new(10.0, 50.0),
            world,
            true,
        );
        assert!((delta.x - 20.0).abs() < 0.001); // Should go right through boundary
    }
}

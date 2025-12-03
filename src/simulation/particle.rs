//! Particle data structures for the simulation.

use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

/// A single particle in the simulation.
///
/// The struct is aligned to 48 bytes to match WGSL storage buffer layout.
/// In WGSL, vec3<u32> has 16-byte alignment, which causes padding after
/// particle_type, making the total struct size 48 bytes.
/// Position and velocity are stored as 2D vectors, with the particle type
/// indicating which species/color the particle belongs to.
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C, align(16))]
pub struct Particle {
    /// X position in world coordinates.
    pub x: f32,
    /// Y position in world coordinates.
    pub y: f32,
    /// X velocity component.
    pub vx: f32,
    /// Y velocity component.
    pub vy: f32,
    /// Particle type/species index (0 to num_types-1).
    pub particle_type: u32,
    /// Padding after particle_type to align _padding2 to 16 bytes.
    pub _padding1: [u32; 3],
    /// Additional padding to match WGSL vec3<u32> at 16-byte alignment.
    /// In WGSL storage buffers, vec3<u32> requires 16-byte alignment,
    /// so it starts at offset 32, making total struct size 48 bytes.
    pub _padding2: [u32; 4],
}

impl Default for Particle {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            vx: 0.0,
            vy: 0.0,
            particle_type: 0,
            _padding1: [0; 3],
            _padding2: [0; 4],
        }
    }
}

impl Particle {
    /// Create a new particle at the given position with zero velocity.
    pub fn new(x: f32, y: f32, particle_type: u32) -> Self {
        Self {
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            particle_type,
            _padding1: [0; 3],
            _padding2: [0; 4],
        }
    }

    /// Create a new particle with position and velocity.
    pub fn with_velocity(x: f32, y: f32, vx: f32, vy: f32, particle_type: u32) -> Self {
        Self {
            x,
            y,
            vx,
            vy,
            particle_type,
            _padding1: [0; 3],
            _padding2: [0; 4],
        }
    }

    /// Get position as a glam Vec2.
    #[inline]
    pub fn position(&self) -> glam::Vec2 {
        glam::Vec2::new(self.x, self.y)
    }

    /// Set position from a glam Vec2.
    #[inline]
    pub fn set_position(&mut self, pos: glam::Vec2) {
        self.x = pos.x;
        self.y = pos.y;
    }

    /// Get velocity as a glam Vec2.
    #[inline]
    pub fn velocity(&self) -> glam::Vec2 {
        glam::Vec2::new(self.vx, self.vy)
    }

    /// Set velocity from a glam Vec2.
    #[inline]
    pub fn set_velocity(&mut self, vel: glam::Vec2) {
        self.vx = vel.x;
        self.vy = vel.y;
    }

    /// Get the speed (magnitude of velocity).
    #[inline]
    pub fn speed(&self) -> f32 {
        self.velocity().length()
    }
}

/// Interaction matrix defining attraction/repulsion between particle types.
///
/// The matrix is stored as a flattened 1D array where `data[i * size + j]`
/// gives the interaction strength from particle type `i` towards type `j`.
///
/// - Positive values indicate attraction
/// - Negative values indicate repulsion
/// - Values typically range from -1.0 to 1.0
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionMatrix {
    /// Flattened NxN matrix data.
    pub data: Vec<f32>,
    /// Number of particle types (matrix is size x size).
    pub size: usize,
}

impl InteractionMatrix {
    /// Create a new zero-initialized interaction matrix.
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![0.0; size * size],
            size,
        }
    }

    /// Create a matrix filled with a constant value.
    pub fn filled(size: usize, value: f32) -> Self {
        Self {
            data: vec![value; size * size],
            size,
        }
    }

    /// Create an identity-like matrix where same-type particles attract.
    pub fn identity(size: usize) -> Self {
        let mut matrix = Self::new(size);
        for i in 0..size {
            matrix.set(i, i, 1.0);
        }
        matrix
    }

    /// Get the interaction strength between two particle types.
    #[inline]
    pub fn get(&self, from_type: usize, to_type: usize) -> f32 {
        debug_assert!(from_type < self.size && to_type < self.size);
        self.data[from_type * self.size + to_type]
    }

    /// Set the interaction strength between two particle types.
    #[inline]
    pub fn set(&mut self, from_type: usize, to_type: usize, value: f32) {
        debug_assert!(from_type < self.size && to_type < self.size);
        self.data[from_type * self.size + to_type] = value;
    }

    /// Make the matrix symmetric (average of m[i][j] and m[j][i]).
    pub fn symmetrize(&mut self) {
        for i in 0..self.size {
            for j in i + 1..self.size {
                let avg = (self.get(i, j) + self.get(j, i)) / 2.0;
                self.set(i, j, avg);
                self.set(j, i, avg);
            }
        }
    }

    /// Make the matrix anti-symmetric (m[i][j] = -m[j][i]).
    pub fn anti_symmetrize(&mut self) {
        for i in 0..self.size {
            for j in i + 1..self.size {
                let val = (self.get(i, j) - self.get(j, i)) / 2.0;
                self.set(i, j, val);
                self.set(j, i, -val);
            }
            // Diagonal should be zero for anti-symmetric
            self.set(i, i, 0.0);
        }
    }

    /// Clamp all values to the given range.
    pub fn clamp(&mut self, min: f32, max: f32) {
        for val in &mut self.data {
            *val = val.clamp(min, max);
        }
    }

    /// Validate that all values are within expected bounds.
    pub fn validate(&self) -> Result<(), String> {
        for (i, &val) in self.data.iter().enumerate() {
            if val.is_nan() {
                return Err(format!("NaN value at index {i}"));
            }
            if val.is_infinite() {
                return Err(format!("Infinite value at index {i}"));
            }
            if !(-2.0..=2.0).contains(&val) {
                return Err(format!(
                    "Value {val} at index {i} is outside expected range [-2, 2]"
                ));
            }
        }
        Ok(())
    }
}

/// Radius matrices defining the minimum and maximum interaction distances.
///
/// For each pair of particle types (i, j), the interaction occurs when
/// the distance is between `min_radius[i*size + j]` and `max_radius[i*size + j]`.
///
/// - Below min_radius: Repulsion force is applied
/// - Between min and max: Attraction/repulsion from InteractionMatrix
/// - Above max_radius: No interaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadiusMatrix {
    /// Minimum interaction distances (flattened NxN).
    pub min_radius: Vec<f32>,
    /// Maximum interaction distances (flattened NxN).
    pub max_radius: Vec<f32>,
    /// Number of particle types.
    pub size: usize,
}

impl RadiusMatrix {
    /// Create new radius matrices with uniform values.
    pub fn new(size: usize, min_radius: f32, max_radius: f32) -> Self {
        Self {
            min_radius: vec![min_radius; size * size],
            max_radius: vec![max_radius; size * size],
            size,
        }
    }

    /// Create radius matrices with default values (30-80 pixels).
    pub fn default_for_size(size: usize) -> Self {
        Self::new(size, 30.0, 80.0)
    }

    /// Get the minimum radius for interaction between two types.
    #[inline]
    pub fn get_min(&self, from_type: usize, to_type: usize) -> f32 {
        self.min_radius[from_type * self.size + to_type]
    }

    /// Get the maximum radius for interaction between two types.
    #[inline]
    pub fn get_max(&self, from_type: usize, to_type: usize) -> f32 {
        self.max_radius[from_type * self.size + to_type]
    }

    /// Set the radius range for interaction between two types.
    pub fn set(&mut self, from_type: usize, to_type: usize, min: f32, max: f32) {
        let idx = from_type * self.size + to_type;
        self.min_radius[idx] = min;
        self.max_radius[idx] = max;
    }

    /// Set the same radius range for all type pairs.
    pub fn set_uniform(&mut self, min: f32, max: f32) {
        for val in &mut self.min_radius {
            *val = min;
        }
        for val in &mut self.max_radius {
            *val = max;
        }
    }

    /// Get the maximum radius value in the matrix (for spatial hash sizing).
    pub fn max_interaction_radius(&self) -> f32 {
        self.max_radius.iter().copied().fold(0.0, f32::max)
    }

    /// Validate that all radius values are sensible.
    pub fn validate(&self) -> Result<(), String> {
        if self.min_radius.len() != self.size * self.size {
            return Err("min_radius size mismatch".to_string());
        }
        if self.max_radius.len() != self.size * self.size {
            return Err("max_radius size mismatch".to_string());
        }

        for i in 0..self.size * self.size {
            let min = self.min_radius[i];
            let max = self.max_radius[i];

            if min < 0.0 {
                return Err(format!("Negative min_radius at index {i}"));
            }
            if max < min {
                return Err(format!(
                    "max_radius ({max}) < min_radius ({min}) at index {i}"
                ));
            }
            if max.is_nan() || min.is_nan() {
                return Err(format!("NaN radius at index {i}"));
            }
        }
        Ok(())
    }
}

/// Structure for Position and Type (SoA layout).
/// aligned to 16 bytes to match WGSL vec2<f32> + u32 + pad.
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C, align(16))]
pub struct ParticlePosType {
    pub x: f32,
    pub y: f32,
    pub particle_type: u32,
    pub _padding: u32,
}

impl From<&Particle> for ParticlePosType {
    fn from(p: &Particle) -> Self {
        Self {
            x: p.x,
            y: p.y,
            particle_type: p.particle_type,
            _padding: 0,
        }
    }
}

/// Structure for Velocity (SoA layout).
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)] // size 8, align 4. Matches vec2<f32> array stride 8.
pub struct ParticleVel {
    pub vx: f32,
    pub vy: f32,
}

impl From<&Particle> for ParticleVel {
    fn from(p: &Particle) -> Self {
        Self { vx: p.vx, vy: p.vy }
    }
}

/// Structure for Position and Type (SoA layout, FP16).
/// Size: 2 (x) + 2 (y) + 4 (type) = 8 bytes.
/// Alignment: 4 bytes (max align of u32 and vec2<f16>).
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C, align(4))]
pub struct ParticlePosTypeHalf {
    pub x: half::f16,
    pub y: half::f16,
    pub particle_type: u32,
}

impl From<&Particle> for ParticlePosTypeHalf {
    fn from(p: &Particle) -> Self {
        Self {
            x: half::f16::from_f32(p.x),
            y: half::f16::from_f32(p.y),
            particle_type: p.particle_type,
        }
    }
}

/// Structure for Velocity (SoA layout, FP16).
/// Size: 2 (vx) + 2 (vy) = 4 bytes.
/// Alignment: 4 bytes (matches vec2<f16> array stride).
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C, align(4))]
pub struct ParticleVelHalf {
    pub vx: half::f16,
    pub vy: half::f16,
}

impl From<&Particle> for ParticleVelHalf {
    fn from(p: &Particle) -> Self {
        Self {
            vx: half::f16::from_f32(p.vx),
            vy: half::f16::from_f32(p.vy),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_particle_creation() {
        let p = Particle::new(100.0, 200.0, 5);
        assert_eq!(p.x, 100.0);
        assert_eq!(p.y, 200.0);
        assert_eq!(p.particle_type, 5);
        assert_eq!(p.vx, 0.0);
        assert_eq!(p.vy, 0.0);
    }

    #[test]
    fn test_particle_velocity() {
        let p = Particle::with_velocity(0.0, 0.0, 3.0, 4.0, 0);
        assert!((p.speed() - 5.0).abs() < 0.0001);
    }

    #[test]
    fn test_interaction_matrix() {
        let mut m = InteractionMatrix::new(3);
        m.set(0, 1, 0.5);
        m.set(1, 0, -0.3);

        assert_eq!(m.get(0, 1), 0.5);
        assert_eq!(m.get(1, 0), -0.3);
        assert_eq!(m.get(2, 2), 0.0);
    }

    #[test]
    fn test_matrix_symmetrize() {
        let mut m = InteractionMatrix::new(2);
        m.set(0, 1, 1.0);
        m.set(1, 0, -1.0);
        m.symmetrize();

        assert_eq!(m.get(0, 1), 0.0);
        assert_eq!(m.get(1, 0), 0.0);
    }

    #[test]
    fn test_radius_matrix_validation() {
        let mut r = RadiusMatrix::new(2, 30.0, 80.0);
        assert!(r.validate().is_ok());

        r.set(0, 0, 100.0, 50.0); // max < min
        assert!(r.validate().is_err());
    }
}

//! Mathematical utility functions.

use std::f32::consts::PI;

/// Two times pi (tau).
pub const TAU: f32 = 2.0 * PI;

/// Linear interpolation between two values.
#[inline]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Clamp a value to a range.
#[inline]
pub fn clamp(x: f32, min: f32, max: f32) -> f32 {
    x.clamp(min, max)
}

/// Smoothstep interpolation.
#[inline]
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Map a value from one range to another.
#[inline]
pub fn map_range(value: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    out_min + (value - in_min) * (out_max - out_min) / (in_max - in_min)
}

/// Wrap a value to the range [0, max).
#[inline]
pub fn wrap(value: f32, max: f32) -> f32 {
    let result = value % max;
    if result < 0.0 { result + max } else { result }
}

/// Euclidean modulo (always positive result).
#[inline]
pub fn euclidean_mod(a: i32, b: i32) -> i32 {
    ((a % b) + b) % b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lerp() {
        assert!((lerp(0.0, 10.0, 0.5) - 5.0).abs() < 0.001);
        assert!((lerp(0.0, 10.0, 0.0) - 0.0).abs() < 0.001);
        assert!((lerp(0.0, 10.0, 1.0) - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_smoothstep() {
        assert!((smoothstep(0.0, 1.0, 0.5) - 0.5).abs() < 0.001);
        assert!((smoothstep(0.0, 1.0, 0.0) - 0.0).abs() < 0.001);
        assert!((smoothstep(0.0, 1.0, 1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_wrap() {
        assert!((wrap(5.5, 5.0) - 0.5).abs() < 0.001);
        assert!((wrap(-0.5, 5.0) - 4.5).abs() < 0.001);
    }

    #[test]
    fn test_euclidean_mod() {
        assert_eq!(euclidean_mod(7, 5), 2);
        assert_eq!(euclidean_mod(-3, 5), 2);
        assert_eq!(euclidean_mod(-8, 5), 2);
    }
}

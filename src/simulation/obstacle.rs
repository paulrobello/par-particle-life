//! Obstacle data types for particle collision.

use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

/// Shape of an obstacle zone.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ObstacleShape {
    Circle,
    Rectangle,
}

/// User-facing obstacle definition (serde-compatible).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Obstacle {
    pub x: f32,
    pub y: f32,
    pub shape: ObstacleShape,
    /// Circle: diameter. Rectangle: width.
    pub width: f32,
    /// Rectangle: height. Unused for circles.
    #[serde(default)]
    pub height: f32,
    /// Restitution coefficient (0.0 = no bounce, 1.0 = full bounce).
    #[serde(default = "default_bounce")]
    pub bounce: f32,
}

fn default_bounce() -> f32 {
    0.8
}

impl Default for Obstacle {
    fn default() -> Self {
        Self {
            x: 960.0,
            y: 540.0,
            shape: ObstacleShape::Circle,
            width: 100.0,
            height: 100.0,
            bounce: 0.8,
        }
    }
}

/// GPU-aligned obstacle data (24 bytes, bytemuck-compatible).
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct ObstacleData {
    pub pos_x: f32,
    pub pos_y: f32,
    /// Circle: radius. Rectangle: half_width.
    pub size_a: f32,
    /// Rectangle: half_height. Unused for circles.
    pub size_b: f32,
    /// 0 = circle, 1 = rectangle.
    pub shape: u32,
    pub bounce: f32,
}

impl From<&Obstacle> for ObstacleData {
    fn from(o: &Obstacle) -> Self {
        match o.shape {
            ObstacleShape::Circle => Self {
                pos_x: o.x,
                pos_y: o.y,
                size_a: o.width / 2.0,
                size_b: 0.0,
                shape: 0,
                bounce: o.bounce,
            },
            ObstacleShape::Rectangle => Self {
                pos_x: o.x,
                pos_y: o.y,
                size_a: o.width / 2.0,
                size_b: o.height / 2.0,
                shape: 1,
                bounce: o.bounce,
            },
        }
    }
}

/// Maximum number of obstacles supported by the GPU buffer.
pub const MAX_OBSTACLES: usize = 16;

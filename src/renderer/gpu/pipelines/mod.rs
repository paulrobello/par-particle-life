//! GPU compute and render pipelines for particle simulation.
//!
//! This module creates and manages the wgpu compute pipelines for simulation
//! and render pipelines for visualization.
//!
//! # Submodules
//!
//! - [`compute`]: Force and advance compute pipelines
//! - [`render`]: Particle visualization render pipelines
//! - [`spatial`]: Spatial hashing optimization pipelines
//! - [`brush`]: Brush interaction pipelines

mod brush;
mod compute;
mod render;
mod spatial;

pub use brush::BrushPipelines;
pub use compute::ComputePipelines;
pub use render::RenderPipelines;
pub use spatial::SpatialHashPipelines;

use bytemuck::{Pod, Zeroable};
use wgpu::{Device, ShaderModuleDescriptor, ShaderSource};

/// Camera uniform for the render pipeline.
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct CameraUniform {
    /// Center X coordinate in world space.
    pub center_x: f32,
    /// Center Y coordinate in world space.
    pub center_y: f32,
    /// Scale X (2.0 / viewport_width for NDC).
    pub scale_x: f32,
    /// Scale Y (2.0 / viewport_height for NDC).
    pub scale_y: f32,
}

impl CameraUniform {
    /// Create camera uniform for the given world and viewport sizes.
    ///
    /// The camera is centered on the world center and scales to fit the viewport.
    pub fn new(
        world_width: f32,
        world_height: f32,
        _viewport_width: f32,
        _viewport_height: f32,
    ) -> Self {
        Self::with_zoom_and_offset(world_width, world_height, 1.0, 0.0, 0.0)
    }

    /// Create camera uniform with zoom and pan offset.
    ///
    /// - `zoom`: 1.0 = default, >1 = zoomed in (see less), <1 = zoomed out (see more)
    /// - `offset_x`, `offset_y`: pan offset in world coordinates
    pub fn with_zoom_and_offset(
        world_width: f32,
        world_height: f32,
        zoom: f32,
        offset_x: f32,
        offset_y: f32,
    ) -> Self {
        // Center is world center plus offset
        let center_x = world_width / 2.0 + offset_x;
        let center_y = world_height / 2.0 + offset_y;

        // Scale is base scale times zoom
        let scale_x = 2.0 / world_width * zoom;
        let scale_y = 2.0 / world_height * zoom;

        Self {
            center_x,
            center_y,
            scale_x,
            scale_y,
        }
    }
}

/// Helper to load WGSL shader source and optionally enable FP16.
///
/// If `use_f16` is true:
/// - Adds `enable f16;` at the top.
/// - Replaces `struct PosType { x: f32, y: f32, ... }` with `struct PosType { x: f16, y: f16, ... }`.
/// - Replaces `vec2<f32>` with `vec2<f16>` in buffer definitions.
/// - Replaces `f32` casts with `f16`.
///
/// Note: This is a simple string replacement and assumes standard formatting.
pub(crate) fn load_shader(device: &Device, label: &str, source: &str) -> wgpu::ShaderModule {
    let use_f16 = device.features().contains(wgpu::Features::SHADER_F16);
    let mut code = String::new();

    if use_f16 {
        code.push_str("enable f16;\n");
        // POS is always f32 for precision
        let s1 = source.replace("POS_FLOAT", "f32");
        // VEL is f16 for bandwidth
        let s2 = s1.replace("VEL_FLOAT", "f16");
        code.push_str(&s2);
    } else {
        // Fallback: everything f32
        let s1 = source.replace("POS_FLOAT", "f32");
        let s2 = s1.replace("VEL_FLOAT", "f32");
        code.push_str(&s2);
    }

    device.create_shader_module(ShaderModuleDescriptor {
        label: Some(label),
        source: ShaderSource::Wgsl(std::borrow::Cow::Owned(code)),
    })
}

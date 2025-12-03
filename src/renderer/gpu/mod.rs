//! GPU rendering using wgpu.
//!
//! This module provides high-performance GPU-accelerated particle simulation
//! and rendering using the wgpu graphics API.
//!
//! # Architecture
//!
//! The GPU renderer consists of:
//! - `GpuContext`: Core wgpu device, queue, and surface management
//! - `SimulationBuffers`: GPU buffers for particle data and simulation parameters
//! - `ComputePipelines`: Compute shaders for force calculation and particle advancement
//! - `RenderPipelines`: Render shaders for particle visualization
//!
//! # Usage
//!
//! ```ignore
//! let context = GpuContext::new(window, /*vsync=*/ true).await?;
//! let buffers = SimulationBuffers::new(&context.device, ...);
//! let pipelines = ComputePipelines::new(&context.device)?;
//! let render = RenderPipelines::new(&context.device, surface_format)?;
//!
//! // Each frame:
//! pipelines.compute_forces(&context, &buffers);
//! pipelines.advance_particles(&context, &buffers);
//! render.draw_particles(&context, &buffers);
//! ```

mod buffers;
mod context;
mod pipelines;

pub use buffers::{
    BrushParamsUniform, BrushRenderUniform, GlowParamsUniform, InfiniteParamsUniform,
    MirrorParamsUniform, RenderBuffers, SimParamsUniform, SimulationBuffers, SpatialHashBuffers,
    SpatialParamsUniform,
};
pub use context::GpuContext;
pub use pipelines::{BrushPipelines, ComputePipelines, RenderPipelines, SpatialHashPipelines};

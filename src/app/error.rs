//! Unified error type for application init and runtime operations.
//!
//! Replaces the mix of `String`, `anyhow::Error`, and `.expect(...)` panics
//! that previously peppered the init path. Init-time failures (GPU context,
//! window creation, etc.) now flow through [`AppError`] and surface as a
//! logged error + clean exit instead of an unfriendly panic.

use thiserror::Error;

/// Errors raised by application init and runtime operations.
///
/// GPU-side errors originating in `gpu_state.rs` are intentionally NOT mapped
/// here yet — that file is owned by a parallel agent and its error paths are
/// deferred. Until they land, GPU init failures still surface via the
/// `Gpu(#[from] anyhow::Error)` variant when explicitly converted at the call
/// site (e.g. `GpuContext::new(...)?` in `init_gpu`).
#[derive(Debug, Error)]
pub enum AppError {
    /// GPU context or pipeline initialization failed.
    ///
    /// Wraps the underlying `anyhow::Error` produced by `GpuContext::new` and
    /// related wgpu setup so callers can propagate without converting to
    /// `String` first.
    #[error("GPU initialization failed: {0}")]
    Gpu(#[from] anyhow::Error),

    /// Window creation failed (winit).
    #[error("Window creation failed: {0}")]
    Window(String),

    /// A custom generator or palette lookup referenced an out-of-range index.
    #[error("{0}")]
    Index(String),

    /// A custom DSL generator produced an invalid matrix or failed to evaluate.
    #[error("{0}")]
    Generator(String),

    /// Simulation config failed validation.
    #[error("Invalid simulation config: {0}")]
    InvalidConfig(String),
}

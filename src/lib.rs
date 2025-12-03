//! # Par Particle Life
//!
//! A GPU-accelerated particle life simulation library in Rust.
//!
//! ## Features
//!
//! - **GPU Rendering**: Uses wgpu for high-performance particle simulation
//! - **Interactive UI**: egui-based controls for real-time parameter adjustment
//! - **Multiple Simulations**: Particle Life and Game of Life
//!
//! ## Example
//!
//! ```no_run
//! use par_particle_life::app::App;
//!
//! fn main() -> anyhow::Result<()> {
//!     App::run(false)
//! }
//! ```

pub mod app;
pub mod generators;
pub mod renderer;
pub mod simulation;
pub mod ui;
pub mod utils;
pub mod video_recorder;

pub use app::App;
pub use simulation::{BoundaryMode, InteractionMatrix, Particle, RadiusMatrix, SimulationConfig};

//! # Par Particle Life
//!
//! A GPU-accelerated particle life simulation library in Rust.
//!
//! `App` is pure state plus simulation methods — it is embeddable headlessly
//! (preset converters, property-based physics tests, headless render farms)
//! without pulling in winit, egui, or a window. Running the interactive GUI
//! is the binary's job (`src/main.rs` constructs an `AppHandler` and drives
//! the winit event loop); see `app::handler::AppHandler` if you need to
//! reproduce that wiring.
//!
//! ## Features
//!
//! - **GPU Rendering**: Uses wgpu for high-performance particle simulation
//! - **Interactive UI**: egui-based controls for real-time parameter adjustment
//!   (built into the `par-particle-life` binary, not the library)
//! - **Headless embedding**: construct `App::new(...)` to inspect or mutate
//!   simulation state without launching a window
//!
//! ## Example
//!
//! ```no_run
//! use par_particle_life::app::handler::AppHandler;
//! use winit::event_loop::{ControlFlow, EventLoop};
//!
//! fn main() -> anyhow::Result<()> {
//!     let event_loop = EventLoop::new()?;
//!     event_loop.set_control_flow(ControlFlow::Poll);
//!     let mut handler = AppHandler::new(false);
//!     event_loop.run_app(&mut handler)?;
//!     Ok(())
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

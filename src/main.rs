//! Par Particle Life - GPU-accelerated particle simulation in Rust.
//!
//! A port of Sandbox Science's particle life simulation, featuring:
//! - GPU-accelerated physics using wgpu
//! - Interactive egui-based UI
//! - Multiple simulation modes (Particle Life, Game of Life)
//!
//! The GUI runner lives in this binary (ARC-004); the library exposes the
//! pure state and simulation methods so it can be embedded headlessly.

use anyhow::Result;
use clap::Parser;
use par_particle_life::app::handler::AppHandler;
use winit::event_loop::{ControlFlow, EventLoop};

/// Par Particle Life - GPU-accelerated particle simulation in Rust.
///
/// A port of Sandbox Science's particle life simulation, featuring:
/// - GPU-accelerated physics using wgpu
/// - Interactive egui-based UI
/// - Multiple simulation modes (Particle Life, Game of Life)
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Resets application configuration to defaults on startup.
    #[arg(long)]
    reset_config: bool,
}

fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();

    // Run the application
    run_app(cli.reset_config)
}

/// GUI runner entry point.
///
/// Owns the winit `EventLoop` and drives an [`AppHandler`] — the platform
/// layer that binds wgpu + egui + winit to the library's [`App`] state.
/// Lives in the binary (not the library) so library embedders can run the
/// simulation headlessly without pulling in winit.
fn run_app(reset_config: bool) -> Result<()> {
    log::info!("Par Particle Life starting...");

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app_handler = AppHandler::new(reset_config);
    event_loop.run_app(&mut app_handler)?;

    Ok(())
}

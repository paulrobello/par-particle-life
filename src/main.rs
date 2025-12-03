//! Par Particle Life - GPU-accelerated particle simulation in Rust.
//!
//! A port of Sandbox Science's particle life simulation, featuring:
//! - GPU-accelerated physics using wgpu
//! - Interactive egui-based UI
//! - Multiple simulation modes (Particle Life, Game of Life)

use anyhow::Result;
use clap::Parser;
use par_particle_life::App;

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
    App::run(cli.reset_config)
}

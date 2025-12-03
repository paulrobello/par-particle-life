//! Application module containing the main app state and entry point.

mod config;
mod gpu_state;
pub(crate) mod handler;
mod input;
mod preset;
mod state;

pub use config::AppConfig;
pub use input::{BrushState, BrushTool, CameraState};
pub use preset::Preset;
pub use state::App;

//! Application module containing the main app state and entry point.

mod config;
pub mod error;
mod gpu_state;
pub mod handler;
mod input;
mod preset;
mod state;

pub use config::AppConfig;
pub use error::AppError;
pub use input::{BrushState, BrushTool, CameraState};
pub use preset::Preset;
pub use state::App;

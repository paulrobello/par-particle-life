//! Application handler module for the winit event loop.
//!
//! This module contains the `AppHandler` struct which manages the application
//! lifecycle, including GPU initialization, event handling, rendering, and UI.

mod brush;
mod buffer_sync;
mod events;
mod gpu_compute;
mod init;
mod presets_ops;
mod recording;
mod render;
mod ui;
mod update;

use std::time::Instant;

use crate::app::gpu_state::GpuState;
use crate::app::{App, BrushState, CameraState, Preset};
use crate::generators::RuleType;
use crate::simulation::ObstacleShape;
use crate::video_recorder::{VideoFormat, VideoRecorder};

/// Tracks whether the current rule selection is a built-in type or custom generator.
#[derive(Debug, Clone)]
#[allow(dead_code)] // variants/fields consumed by subsequent tasks
pub(crate) enum RuleSelection {
    BuiltIn(RuleType),
    Custom(usize),
}

/// Application handler for the winit event loop.
///
/// Public so the binary (`src/main.rs`) can construct it as part of the
/// GUI runner that lives outside the library (ARC-004). The struct's fields
/// remain crate-private — external code can build one via [`AppHandler::new`]
/// and drive it via `winit::ApplicationHandler`, but cannot poke internal
/// state directly.
pub struct AppHandler {
    /// The application state.
    pub(crate) app: App,
    /// GPU context (created when window is available).
    pub(crate) gpu: Option<GpuState>,
    /// Pending vsync toggle to apply after the current frame is presented.
    pub(crate) pending_vsync: Option<bool>,
    /// Last frame time for FPS calculation.
    pub(crate) last_frame: Instant,
    /// Frame count for FPS display.
    pub(crate) frame_count: u32,
    /// Last FPS calculation time.
    pub(crate) last_fps_time: Instant,
    /// Current FPS.
    pub(crate) fps: f32,
    /// Smoothed FPS (EMA) to reduce jitter in the HUD.
    pub(crate) fps_ema: f32,
    /// Show UI sidebar.
    pub(crate) show_ui: bool,
    /// UI: Is Simulation section open?
    pub(crate) ui_simulation_open: bool,
    /// UI: Is Physics section open?
    pub(crate) ui_physics_open: bool,
    /// UI: Is Generators section open?
    pub(crate) ui_generators_open: bool,
    /// UI: Is Interaction Matrix section open?
    pub(crate) ui_interaction_matrix_open: bool,
    /// UI: Is Brush Tools section open?
    pub(crate) ui_brush_tools_open: bool,
    /// UI: Is Rendering section open?
    pub(crate) ui_rendering_open: bool,
    /// UI: Is Presets section open?
    pub(crate) ui_presets_open: bool,
    /// UI: Is Keyboard Shortcuts section open?
    pub(crate) ui_keyboard_shortcuts_open: bool,
    /// UI: Is Obstacles section open?
    pub(crate) ui_obstacles_open: bool,
    /// Available presets list.
    pub(crate) preset_list: Vec<String>,
    /// Currently selected preset name for loading.
    pub(crate) selected_preset: String,
    /// Name for saving new preset.
    pub(crate) save_preset_name: String,
    /// Status message for preset operations.
    pub(crate) preset_status: String,
    /// Currently selected custom palette name for loading.
    pub(crate) selected_custom_palette: String,
    /// Name for saving custom palettes.
    pub(crate) save_custom_palette_name: String,
    /// Last captured file path (screenshot or video) for "Open" button.
    pub(crate) last_capture_path: Option<String>,
    /// Screenshot requested flag.
    pub(crate) screenshot_requested: bool,
    /// Screenshot counter for unique filenames.
    pub(crate) screenshot_counter: u32,
    /// Video recording active flag.
    pub(crate) is_recording: bool,
    /// Hide UI when capturing screenshots/recordings.
    pub(crate) capture_hide_ui: bool,
    /// Recorded frames for native GIF export (fallback when ffmpeg unavailable).
    pub(crate) recorded_frames: Vec<image::RgbaImage>,
    /// Frame skip counter for recording (record every N frames).
    pub(crate) video_frame_skip: u32,
    /// Current frame counter for skip logic.
    pub(crate) video_frame_counter: u32,
    /// Video file counter for unique filenames.
    pub(crate) video_counter: u32,
    /// Video recorder for ffmpeg-based encoding.
    pub(crate) video_recorder: Option<VideoRecorder>,
    /// Selected video output format.
    pub(crate) video_format: VideoFormat,
    /// Whether to use ffmpeg for video encoding (true) or native GIF (false).
    pub(crate) use_ffmpeg: bool,
    /// Flag to stop recording after current frame (avoids borrow conflicts).
    pub(crate) pending_stop_recording: bool,
    /// Camera state for pan/zoom.
    pub(crate) camera: CameraState,
    /// Brush state for user interaction tools.
    pub(crate) brush: BrushState,
    /// Current mouse position in screen coordinates.
    pub(crate) mouse_screen_pos: glam::Vec2,
    /// Flag indicating particles were modified and need GPU buffer sync.
    pub(crate) needs_sync: bool,
    /// Flag indicating spatial hash buffers need recreating (e.g., cell size changed).
    pub(crate) needs_sync_spatial_buffers: bool,
    /// Last time metrics were logged.
    pub(crate) last_log_time: Instant,
    /// When true, run exactly one frame then pause (step-by-step mode).
    pub(crate) step_requested: bool,
    /// Current rule selection (built-in or custom generator).
    #[allow(dead_code)] // consumed by subsequent tasks
    pub(crate) rule_selection: RuleSelection,
    /// Index of the currently selected obstacle (-1 = none).
    pub(crate) selected_obstacle: i32,
    /// Shape for the next obstacle placed via the Obstacle tool.
    pub(crate) obstacle_tool_shape: ObstacleShape,
    /// True when the user is dragging the selected obstacle.
    pub(crate) obstacle_dragging: bool,
    /// World-space offset from obstacle center to mouse at drag start.
    pub(crate) obstacle_drag_offset: glam::Vec2,
    /// Whether the cursor was last seen over the egui UI panel.
    pub(crate) cursor_over_ui: bool,
    /// Right edge of the UI panel in screen pixels (0 = no panel shown).
    pub(crate) ui_panel_right_edge: f32,
}

impl AppHandler {
    /// Application name for directory paths.
    const APP_NAME: &'static str = "par-particle-life";

    /// Get the screenshots directory, creating it if necessary.
    ///
    /// Uses platform-specific picture directory (e.g., ~/Pictures on Linux,
    /// Pictures folder on macOS/Windows) with an app-specific subdirectory.
    pub(crate) fn screenshots_dir() -> std::path::PathBuf {
        let base = dirs::picture_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        base.join(Self::APP_NAME)
    }

    /// Get the videos directory, creating it if necessary.
    ///
    /// Uses platform-specific video directory (e.g., ~/Videos on Linux,
    /// Movies folder on macOS, Videos on Windows) with an app-specific subdirectory.
    pub(crate) fn videos_dir() -> std::path::PathBuf {
        let base = dirs::video_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        base.join(Self::APP_NAME)
    }

    /// Ensure the screenshots directory exists.
    pub(crate) fn ensure_screenshots_dir() -> anyhow::Result<std::path::PathBuf> {
        let dir = Self::screenshots_dir();
        if !dir.exists() {
            std::fs::create_dir_all(&dir)?;
        }
        Ok(dir)
    }

    /// Ensure the videos directory exists.
    pub(crate) fn ensure_videos_dir() -> anyhow::Result<std::path::PathBuf> {
        let dir = Self::videos_dir();
        if !dir.exists() {
            std::fs::create_dir_all(&dir)?;
        }
        Ok(dir)
    }

    /// Construct a new handler around a fresh [`App`].
    ///
    /// Public so the binary's `main` can build one as part of the GUI runner
    /// (ARC-004 moved `App::run` out of the library). Library embedders do
    /// not need this — they use [`App::new`] directly for headless access.
    pub fn new(reset_config: bool) -> Self {
        let app = App::new(reset_config);
        let preset_list = Preset::list_presets().unwrap_or_default();

        // Capture config values before moving 'app'
        let ui_simulation_open = app.config.ui_simulation_open;
        let ui_physics_open = app.config.ui_physics_open;
        let ui_generators_open = app.config.ui_generators_open;
        let ui_interaction_matrix_open = app.config.ui_interaction_matrix_open;
        let ui_brush_tools_open = app.config.ui_brush_tools_open;
        let ui_rendering_open = app.config.ui_rendering_open;
        let ui_presets_open = app.config.ui_presets_open;
        let ui_keyboard_shortcuts_open = app.config.ui_keyboard_shortcuts_open;
        let ui_obstacles_open = app.config.ui_obstacles_open;

        let current_rule = app.current_rule;

        let mouse_screen_pos = glam::Vec2::ZERO;
        let last_log_time = Instant::now();

        log::info!("Startup Settings:");
        log::info!("  Particles: {}", app.sim_config.num_particles);
        log::info!("  Types: {}", app.sim_config.num_types);
        log::info!(
            "  World Size: {}x{}",
            app.sim_config.world_size.x,
            app.sim_config.world_size.y
        );
        log::info!(
            "  Spatial Hash Cell Size: {}",
            app.sim_config.spatial_hash_cell_size
        );
        log::info!("  F16 Mode: Enabled (if supported)");

        Self {
            app,
            gpu: None,
            pending_vsync: None,
            last_frame: Instant::now(),
            frame_count: 0,
            last_fps_time: Instant::now(),
            fps: 0.0,
            fps_ema: 0.0,
            show_ui: true,
            ui_simulation_open,
            ui_physics_open,
            ui_generators_open,
            ui_interaction_matrix_open,
            ui_brush_tools_open,
            ui_rendering_open,
            ui_presets_open,
            ui_keyboard_shortcuts_open,
            ui_obstacles_open,
            preset_list,
            selected_preset: String::new(),
            save_preset_name: String::from("my_preset"),
            preset_status: String::new(),
            selected_custom_palette: String::new(),
            save_custom_palette_name: String::from("my_palette"),
            last_capture_path: None,
            screenshot_requested: false,
            screenshot_counter: 0,
            is_recording: false,
            capture_hide_ui: true,
            recorded_frames: Vec::new(),
            video_frame_skip: 2,
            video_frame_counter: 0,
            video_counter: 0,
            video_recorder: None,
            video_format: VideoFormat::MP4,
            use_ffmpeg: true,
            pending_stop_recording: false,
            camera: CameraState::default(),
            brush: BrushState::default(),
            mouse_screen_pos,
            needs_sync: false,
            needs_sync_spatial_buffers: false,
            last_log_time,
            step_requested: false,
            rule_selection: RuleSelection::BuiltIn(current_rule),
            selected_obstacle: -1,
            obstacle_tool_shape: ObstacleShape::Circle,
            obstacle_dragging: false,
            obstacle_drag_offset: glam::Vec2::ZERO,
            cursor_over_ui: false,
            ui_panel_right_edge: 0.0,
        }
    }
}

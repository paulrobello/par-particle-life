//! Application configuration.

use serde::{Deserialize, Serialize};

use crate::generators::{colors::PaletteType, positions::PositionPattern, rules::RuleType};
use crate::simulation::{BoundaryMode, SimulationConfig};

/// Application-level configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Window title.
    pub title: String,
    /// Initial window width.
    pub window_width: u32,
    /// Initial window height.
    pub window_height: u32,
    /// Target frames per second.
    pub target_fps: u32,
    /// Enable VSync.
    pub vsync: bool,
    /// UI: Is Simulation section open?
    pub ui_simulation_open: bool,
    /// UI: Is Physics section open?
    pub ui_physics_open: bool,
    /// UI: Is Generators section open?
    pub ui_generators_open: bool,
    /// UI: Is Interaction Matrix section open?
    pub ui_interaction_matrix_open: bool,
    /// UI: Is Brush Tools section open?
    pub ui_brush_tools_open: bool,
    /// UI: Is Rendering section open?
    pub ui_rendering_open: bool,
    /// UI: Is Presets section open?
    pub ui_presets_open: bool,
    /// UI: Is Keyboard Shortcuts section open?
    pub ui_keyboard_shortcuts_open: bool,

    /// Physics: force factor.
    #[serde(default = "default_phys_force_factor")]
    pub phys_force_factor: f32,
    /// Physics: friction.
    #[serde(default = "default_phys_friction")]
    pub phys_friction: f32,
    /// Physics: repel strength.
    #[serde(default = "default_phys_repel_strength")]
    pub phys_repel_strength: f32,
    /// Physics: max velocity.
    #[serde(default = "default_phys_max_velocity")]
    pub phys_max_velocity: f32,
    /// Physics: boundary mode.
    #[serde(default = "default_phys_boundary_mode")]
    pub phys_boundary_mode: BoundaryMode,
    /// Physics: wall repel strength.
    #[serde(default = "default_phys_wall_repel_strength")]
    pub phys_wall_repel_strength: f32,
    /// Physics: mirror wrap count.
    #[serde(default = "default_phys_mirror_wrap_count")]
    pub phys_mirror_wrap_count: u32,

    /// Simulation: number of particles.
    #[serde(default = "default_sim_num_particles")]
    pub sim_num_particles: u32,
    /// Simulation: number of types.
    #[serde(default = "default_sim_num_types")]
    pub sim_num_types: u32,

    /// Generators: current rule type.
    #[serde(default = "default_gen_rule")]
    pub gen_rule: RuleType,
    /// Generators: current palette type.
    #[serde(default = "default_gen_palette")]
    pub gen_palette: PaletteType,
    /// Generators: current spawn pattern.
    #[serde(default = "default_gen_pattern")]
    pub gen_pattern: PositionPattern,

    /// Rendering: particle size.
    #[serde(default = "default_particle_size")]
    pub render_particle_size: f32,
    /// Rendering: background color.
    #[serde(default = "default_background_color")]
    pub render_background_color: [f32; 3],
    /// Rendering: glow enabled.
    #[serde(default = "default_glow_enabled")]
    pub render_glow_enabled: bool,
    /// Rendering: glow intensity.
    #[serde(default = "default_glow_intensity")]
    pub render_glow_intensity: f32,
    /// Rendering: glow size.
    #[serde(default = "default_glow_size")]
    pub render_glow_size: f32,
    /// Rendering: glow steepness.
    #[serde(default = "default_glow_steepness")]
    pub render_glow_steepness: f32,
    /// Rendering: spatial hash cell size.
    #[serde(default = "default_spatial_hash_cell_size")]
    pub render_spatial_hash_cell_size: f32,

    /// Simulation: auto-scale radii with particle density.
    #[serde(default = "default_auto_scale_radii")]
    pub auto_scale_radii: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            title: "Par Particle Life".to_string(),
            window_width: 1920,
            window_height: 1080,
            target_fps: 60,
            vsync: false,
            // UI section open/closed state
            ui_simulation_open: true,
            ui_physics_open: true,
            ui_generators_open: true,
            ui_interaction_matrix_open: false, // Default false as per UI
            ui_brush_tools_open: true,
            ui_rendering_open: false,          // Default false as per UI
            ui_presets_open: false,            // Default false as per UI
            ui_keyboard_shortcuts_open: false, // Default false as per UI

            // Physics defaults
            phys_force_factor: default_phys_force_factor(),
            phys_friction: default_phys_friction(),
            phys_repel_strength: default_phys_repel_strength(),
            phys_max_velocity: default_phys_max_velocity(),
            phys_boundary_mode: default_phys_boundary_mode(),
            phys_wall_repel_strength: default_phys_wall_repel_strength(),
            phys_mirror_wrap_count: default_phys_mirror_wrap_count(),

            // Simulation defaults (mirror SimulationConfig::default)
            sim_num_particles: default_sim_num_particles(),
            sim_num_types: default_sim_num_types(),

            // Generator defaults
            gen_rule: default_gen_rule(),
            gen_palette: default_gen_palette(),
            gen_pattern: default_gen_pattern(),

            // Rendering defaults (mirror SimulationConfig::default)
            render_particle_size: default_particle_size(),
            render_background_color: default_background_color(),
            render_glow_enabled: default_glow_enabled(),
            render_glow_intensity: default_glow_intensity(),
            render_glow_size: default_glow_size(),
            render_glow_steepness: default_glow_steepness(),
            render_spatial_hash_cell_size: default_spatial_hash_cell_size(),

            // Density scaling
            auto_scale_radii: default_auto_scale_radii(),
        }
    }
}

fn default_sim_num_particles() -> u32 {
    SimulationConfig::default().num_particles
}

fn default_sim_num_types() -> u32 {
    SimulationConfig::default().num_types
}

fn default_gen_rule() -> RuleType {
    RuleType::Random
}

fn default_gen_palette() -> PaletteType {
    PaletteType::Rainbow
}

fn default_gen_pattern() -> PositionPattern {
    PositionPattern::Disk
}

fn default_particle_size() -> f32 {
    SimulationConfig::default().particle_size
}

fn default_background_color() -> [f32; 3] {
    SimulationConfig::default().background_color
}

fn default_glow_enabled() -> bool {
    SimulationConfig::default().enable_glow
}

fn default_glow_intensity() -> f32 {
    SimulationConfig::default().glow_intensity
}

fn default_glow_size() -> f32 {
    SimulationConfig::default().glow_size
}

fn default_glow_steepness() -> f32 {
    SimulationConfig::default().glow_steepness
}

fn default_spatial_hash_cell_size() -> f32 {
    SimulationConfig::default().spatial_hash_cell_size
}

fn default_phys_force_factor() -> f32 {
    SimulationConfig::default().force_factor
}

fn default_phys_friction() -> f32 {
    SimulationConfig::default().friction
}

fn default_phys_repel_strength() -> f32 {
    SimulationConfig::default().repel_strength
}

fn default_phys_max_velocity() -> f32 {
    500.0
}

fn default_phys_boundary_mode() -> BoundaryMode {
    SimulationConfig::default().boundary_mode
}

fn default_phys_wall_repel_strength() -> f32 {
    SimulationConfig::default().wall_repel_strength
}

fn default_phys_mirror_wrap_count() -> u32 {
    SimulationConfig::default().mirror_wrap_count
}

fn default_auto_scale_radii() -> bool {
    true
}

impl AppConfig {
    /// Get the application's configuration directory.
    pub fn config_dir() -> anyhow::Result<std::path::PathBuf> {
        let mut path = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
        path.push("par-particle-life");
        Ok(path)
    }

    /// Load the application configuration from a file, or return a default if not found.
    pub fn load() -> Self {
        let mut config_path = match Self::config_dir() {
            Ok(path) => path,
            Err(e) => {
                log::warn!(
                    "Failed to get config directory, using default config: {}",
                    e
                );
                return Self::default();
            }
        };
        config_path.push("config.json");

        match std::fs::read_to_string(&config_path) {
            Ok(contents) => match serde_json::from_str(&contents) {
                Ok(config) => config,
                Err(e) => {
                    log::warn!(
                        "Failed to parse config file {}, using default: {}",
                        config_path.display(),
                        e
                    );
                    Self::default()
                }
            },
            Err(e) => {
                log::info!(
                    "Config file {} not found or could not be read, using default: {}",
                    config_path.display(),
                    e
                );
                Self::default()
            }
        }
    }

    /// Save the application configuration to a file.
    pub fn save(&self) -> anyhow::Result<()> {
        let mut config_path = Self::config_dir()?;
        std::fs::create_dir_all(&config_path)?; // Ensure directory exists
        config_path.push("config.json");

        let contents = serde_json::to_string_pretty(self)?;
        std::fs::write(&config_path, contents)?;
        log::info!("Saved config to {}", config_path.display());
        Ok(())
    }
}

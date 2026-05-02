//! Preset save/load functionality for simulation states.

use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::generators::{colors::PaletteType, positions::PositionPattern, rules::RuleType};
use crate::simulation::{InteractionMatrix, RadiusMatrix, SimulationConfig};

/// A saved simulation preset containing all configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    /// Name of the preset.
    pub name: String,
    /// Simulation configuration.
    pub sim_config: SimulationConfig,
    /// Interaction matrix.
    pub interaction_matrix: InteractionMatrix,
    /// Radius matrices.
    pub radius_matrix: RadiusMatrix,
    /// Rule type used to generate the matrix.
    pub rule_type: RuleType,
    /// Color palette type.
    pub palette_type: PaletteType,
    /// Position pattern.
    pub position_pattern: PositionPattern,
    /// Per-type mass values.
    #[serde(default = "default_type_masses")]
    pub type_masses: Vec<f32>,
    /// Per-type size multiplier values.
    #[serde(default = "default_type_sizes")]
    pub type_sizes: Vec<f32>,
}

fn default_type_masses() -> Vec<f32> {
    vec![1.0; 7]
}

fn default_type_sizes() -> Vec<f32> {
    vec![1.0; 7]
}

impl Preset {
    /// Create a new preset from the current simulation state.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: impl Into<String>,
        sim_config: &SimulationConfig,
        interaction_matrix: &InteractionMatrix,
        radius_matrix: &RadiusMatrix,
        rule_type: RuleType,
        palette_type: PaletteType,
        position_pattern: PositionPattern,
        type_masses: &[f32],
        type_sizes: &[f32],
    ) -> Self {
        Self {
            name: name.into(),
            sim_config: sim_config.clone(),
            interaction_matrix: interaction_matrix.clone(),
            radius_matrix: radius_matrix.clone(),
            rule_type,
            palette_type,
            position_pattern,
            type_masses: type_masses.to_vec(),
            type_sizes: type_sizes.to_vec(),
        }
    }

    /// Save the preset to a JSON file.
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let json = serde_json::to_string_pretty(self).context("Failed to serialize preset")?;
        std::fs::write(path, json)
            .with_context(|| format!("Failed to write preset to {}", path.display()))?;
        Ok(())
    }

    /// Load a preset from a JSON file.
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let json = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read preset from {}", path.display()))?;
        let preset: Self = serde_json::from_str(&json).context("Failed to deserialize preset")?;
        Ok(preset)
    }

    /// Get the default presets directory.
    pub fn presets_dir() -> std::path::PathBuf {
        // Use XDG data directory or fall back to current directory
        if let Some(data_dir) = dirs::data_dir() {
            data_dir.join("par-particle-life").join("presets")
        } else {
            std::path::PathBuf::from("presets")
        }
    }

    /// Ensure the presets directory exists.
    pub fn ensure_presets_dir() -> Result<std::path::PathBuf> {
        let dir = Self::presets_dir();
        if !dir.exists() {
            std::fs::create_dir_all(&dir).with_context(|| {
                format!("Failed to create presets directory: {}", dir.display())
            })?;
        }
        Ok(dir)
    }

    /// List all presets in the presets directory.
    pub fn list_presets() -> Result<Vec<String>> {
        let dir = Self::presets_dir();
        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut presets = Vec::new();
        for entry in std::fs::read_dir(&dir)
            .with_context(|| format!("Failed to read presets directory: {}", dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false)
                && let Some(name) = path.file_stem()
            {
                presets.push(name.to_string_lossy().into_owned());
            }
        }

        presets.sort();
        Ok(presets)
    }
}

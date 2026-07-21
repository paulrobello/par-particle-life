//! Preset save/load functionality for simulation states.

use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::generators::{
    colors::{CustomPalette, PaletteType},
    positions::PositionPattern,
    rules::RuleType,
};
use crate::simulation::{
    InteractionMatrix, MatrixVariationConfig, Obstacle, RadiusMatrix, SimulationConfig,
};

/// A saved simulation preset containing all configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    /// Name of the preset.
    pub name: String,
    /// Simulation configuration.
    pub sim_config: SimulationConfig,
    /// Interaction matrix.
    pub interaction_matrix: InteractionMatrix,
    /// Base interaction matrix for time-varying transforms.
    #[serde(default)]
    pub matrix_variation_base: Option<InteractionMatrix>,
    /// Time-varying interaction matrix settings.
    #[serde(default)]
    pub matrix_variation: MatrixVariationConfig,
    /// Elapsed matrix variation time in seconds.
    #[serde(default)]
    pub matrix_variation_time: f32,
    /// Radius matrices.
    pub radius_matrix: RadiusMatrix,
    /// Rule type used to generate the matrix.
    pub rule_type: RuleType,
    /// Color palette type.
    pub palette_type: PaletteType,
    /// Saved custom palette, if one was active.
    #[serde(default)]
    pub custom_palette: Option<CustomPalette>,
    /// Position pattern.
    pub position_pattern: PositionPattern,
    /// Per-type mass values.
    #[serde(default = "default_type_masses")]
    pub type_masses: Vec<f32>,
    /// Per-type size multiplier values.
    #[serde(default = "default_type_sizes")]
    pub type_sizes: Vec<f32>,
    /// Obstacle zones.
    #[serde(default)]
    pub obstacles: Vec<Obstacle>,
}

fn default_type_masses() -> Vec<f32> {
    vec![1.0; 7]
}

fn default_type_sizes() -> Vec<f32> {
    vec![1.0; 7]
}

impl Preset {
    /// Create a new preset from the current simulation state.
    //
    // too_many_arguments: this constructor snapshots heterogeneous pieces of
    // simulation state (config, matrices, generator choices, per-type slices,
    // obstacles) into the serialisable `Preset`. There is no single natural
    // bundle — each argument maps 1:1 to a distinct `Preset` field — so
    // collapsing them into a wrapper struct would just rename the args
    // without reducing the surface.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: impl Into<String>,
        sim_config: &SimulationConfig,
        interaction_matrix: &InteractionMatrix,
        radius_matrix: &RadiusMatrix,
        matrix_variation_base: &InteractionMatrix,
        matrix_variation: MatrixVariationConfig,
        matrix_variation_time: f32,
        rule_type: RuleType,
        palette_type: PaletteType,
        custom_palette: Option<CustomPalette>,
        position_pattern: PositionPattern,
        type_masses: &[f32],
        type_sizes: &[f32],
        obstacles: &[Obstacle],
    ) -> Self {
        Self {
            name: name.into(),
            sim_config: sim_config.clone(),
            interaction_matrix: interaction_matrix.clone(),
            matrix_variation_base: Some(matrix_variation_base.clone()),
            matrix_variation,
            matrix_variation_time,
            radius_matrix: radius_matrix.clone(),
            rule_type,
            palette_type,
            custom_palette,
            position_pattern,
            type_masses: type_masses.to_vec(),
            type_sizes: type_sizes.to_vec(),
            obstacles: obstacles.to_vec(),
        }
    }

    /// Return true when the path looks like a preset JSON file.
    pub fn is_preset_file(path: impl AsRef<Path>) -> bool {
        path.as_ref()
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
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
    ///
    /// Enforces a file-size cap (1 MiB) before reading to avoid OOM on a
    /// crafted `.json` (SEC-001). A legitimate preset is at most a few KB.
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        const MAX_PRESET_FILE_SIZE: u64 = 1024 * 1024; // 1 MiB
        let file_size = std::fs::metadata(path)
            .with_context(|| format!("Failed to stat preset at {}", path.display()))?
            .len();
        if file_size > MAX_PRESET_FILE_SIZE {
            anyhow::bail!(
                "Preset file {} is {} bytes, exceeds the {} byte limit",
                path.display(),
                file_size,
                MAX_PRESET_FILE_SIZE
            );
        }
        let json = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read preset from {}", path.display()))?;
        let preset: Self = serde_json::from_str(&json).context("Failed to deserialize preset")?;
        Ok(preset)
    }

    /// Validate the loaded preset before it is applied to the live simulation.
    ///
    /// Checks:
    /// - `sim_config.validate()` (num_particles/num_types/force_factor/friction/
    ///   repel_strength/world_size sanity — SEC-001 / ARC-016)
    /// - Interaction matrix shape (`size * size == data.len()`) — a mismatch
    ///   currently panics on access.
    /// - Radius matrix shape and consistency.
    /// - `interaction_matrix.validate()` and `radius_matrix.validate()` for
    ///   NaN/Inf/out-of-range values.
    /// - `type_masses` / `type_sizes` lengths match `num_types`.
    pub fn validate(&self) -> Result<()> {
        self.sim_config.validate().map_err(|e| {
            anyhow::anyhow!("Preset '{}' has invalid simulation config: {e}", self.name)
        })?;

        let n = self.sim_config.num_types as usize;
        if self.interaction_matrix.size != n || self.interaction_matrix.data.len() != n * n {
            anyhow::bail!(
                "Preset '{}' interaction_matrix shape mismatch: size={}, data.len={}, \
                 expected {}x{} ({})",
                self.name,
                self.interaction_matrix.size,
                self.interaction_matrix.data.len(),
                n,
                n,
                n * n
            );
        }
        self.interaction_matrix.validate().map_err(|e| {
            anyhow::anyhow!(
                "Preset '{}' interaction_matrix has invalid values: {e}",
                self.name
            )
        })?;

        if self.radius_matrix.size != n
            || self.radius_matrix.min_radius.len() != n * n
            || self.radius_matrix.max_radius.len() != n * n
        {
            anyhow::bail!(
                "Preset '{}' radius_matrix shape mismatch: size={}, min_len={}, max_len={}, \
                 expected {}x{} ({})",
                self.name,
                self.radius_matrix.size,
                self.radius_matrix.min_radius.len(),
                self.radius_matrix.max_radius.len(),
                n,
                n,
                n * n
            );
        }
        self.radius_matrix.validate().map_err(|e| {
            anyhow::anyhow!(
                "Preset '{}' radius_matrix has invalid values: {e}",
                self.name
            )
        })?;

        if !self.type_masses.is_empty() && self.type_masses.len() != n {
            anyhow::bail!(
                "Preset '{}' type_masses length {} does not match num_types {}",
                self.name,
                self.type_masses.len(),
                n
            );
        }
        if !self.type_sizes.is_empty() && self.type_sizes.len() != n {
            anyhow::bail!(
                "Preset '{}' type_sizes length {} does not match num_types {}",
                self.name,
                self.type_sizes.len(),
                n
            );
        }

        Ok(())
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
            if Self::is_preset_file(&path)
                && let Some(name) = path.file_stem()
            {
                presets.push(name.to_string_lossy().into_owned());
            }
        }

        presets.sort();
        Ok(presets)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::Preset;

    #[test]
    fn preset_file_detection_accepts_json_files_case_insensitively() {
        assert!(Preset::is_preset_file(Path::new("example.json")));
        assert!(Preset::is_preset_file(Path::new("example.JSON")));
    }

    #[test]
    fn preset_file_detection_rejects_non_json_files_and_directories() {
        assert!(!Preset::is_preset_file(Path::new("example.txt")));
        assert!(!Preset::is_preset_file(Path::new("example")));
    }
}

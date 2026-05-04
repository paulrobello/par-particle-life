//! Time-varying interaction matrix transforms.

use serde::{Deserialize, Serialize};

use super::InteractionMatrix;

/// Dynamic transform applied to a preserved base interaction matrix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MatrixVariationMode {
    /// Add a periodic wave to each cell with a stable per-cell phase offset.
    #[default]
    Oscillate,
    /// Add slower, row/column-biased drift waves.
    Drift,
}

impl MatrixVariationMode {
    /// Human-readable name for UI controls.
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Oscillate => "Oscillate",
            Self::Drift => "Drift",
        }
    }
}

/// Settings for time-varying interaction matrices.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MatrixVariationConfig {
    /// Whether matrix variation is active.
    #[serde(default)]
    pub enabled: bool,
    /// Variation algorithm.
    #[serde(default)]
    pub mode: MatrixVariationMode,
    /// Maximum additive deviation from the base matrix.
    #[serde(default = "default_amplitude")]
    pub amplitude: f32,
    /// Phase speed in radians per second.
    #[serde(default = "default_speed")]
    pub speed: f32,
}

fn default_amplitude() -> f32 {
    0.15
}

fn default_speed() -> f32 {
    0.35
}

impl Default for MatrixVariationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: MatrixVariationMode::Oscillate,
            amplitude: default_amplitude(),
            speed: default_speed(),
        }
    }
}

impl MatrixVariationConfig {
    /// Apply the configured variation to `base` at `elapsed_seconds`.
    pub fn apply(self, base: &InteractionMatrix, elapsed_seconds: f32) -> InteractionMatrix {
        if !self.enabled || self.amplitude <= 0.0 || self.speed <= 0.0 {
            return base.clone();
        }

        let mut matrix = base.clone();
        let phase = elapsed_seconds * self.speed;
        let amplitude = self.amplitude.clamp(0.0, 1.0);

        for i in 0..base.size {
            for j in 0..base.size {
                let cell_phase = i as f32 * 1.37 + j as f32 * 0.73;
                let delta = match self.mode {
                    MatrixVariationMode::Oscillate => amplitude * (phase + cell_phase).sin(),
                    MatrixVariationMode::Drift => {
                        let row = (phase * 0.23 + i as f32 * 0.37).sin();
                        let col = (phase * 0.17 + j as f32 * 0.61).cos();
                        amplitude * 0.5 * (row + col)
                    }
                };
                matrix.set(i, j, (base.get(i, j) + delta).clamp(-1.0, 1.0));
            }
        }

        matrix
    }
}

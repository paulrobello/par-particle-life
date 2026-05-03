//! User-defined custom rule generators loaded from JSON files.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::expression::{EvalContext, Expr, ExprError};
use crate::simulation::InteractionMatrix;

/// A user-defined custom rule generator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomGenerator {
    /// Display name shown in the Rules dropdown.
    pub name: String,
    /// Tooltip description.
    #[serde(default)]
    pub description: String,
    /// DSL expression evaluated per cell (i, j, n available).
    pub expression: String,
    /// Fixed type count, or None for any count.
    #[serde(default)]
    pub num_types: Option<usize>,
    /// Parsed AST, cached after first use.
    #[serde(skip)]
    pub compiled: Option<Expr>,
}

impl CustomGenerator {
    /// Get the custom generators directory path.
    pub fn custom_dir() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("par-particle-life")
            .join("custom-generators")
    }

    /// Ensure the custom generators directory exists.
    pub fn ensure_dir() -> anyhow::Result<PathBuf> {
        let dir = Self::custom_dir();
        if !dir.exists() {
            std::fs::create_dir_all(&dir)?;
        }
        Ok(dir)
    }

    /// List all custom generators from the directory.
    pub fn list() -> anyhow::Result<Vec<CustomGenerator>> {
        let dir = Self::custom_dir();
        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut generators = Vec::new();
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "json") {
                let content = std::fs::read_to_string(&path)?;
                match serde_json::from_str::<CustomGenerator>(&content) {
                    Ok(generator) => generators.push(generator),
                    Err(e) => {
                        log::warn!("Failed to parse custom generator {}: {e}", path.display());
                    }
                }
            }
        }

        generators.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(generators)
    }

    /// Save this generator to a JSON file in the custom generators directory.
    pub fn save_to_file(&self) -> anyhow::Result<()> {
        let dir = Self::ensure_dir()?;
        let filename = self.name.to_lowercase().replace([' ', '/'], "-");
        let path = dir.join(format!("{filename}.json"));
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)?;
        Ok(())
    }

    /// Generate an interaction matrix using this custom generator's expression.
    pub fn generate(&mut self, num_types: usize) -> Result<InteractionMatrix, ExprError> {
        if let Some(required) = self.num_types
            && num_types != required
        {
            return Err(ExprError::Eval(format!(
                "'{}' requires exactly {} types, got {}",
                self.name, required, num_types
            )));
        }

        if self.compiled.is_none() {
            let expr = Expr::parse(&self.expression)?;
            self.compiled = Some(expr);
        }

        let expr = self.compiled.as_ref().unwrap();
        let mut matrix = InteractionMatrix::new(num_types);

        for i in 0..num_types {
            for j in 0..num_types {
                let ctx = EvalContext {
                    i: i as f32,
                    j: j as f32,
                    n: num_types as f32,
                };
                let val = expr.eval(&ctx)?;
                matrix.set(i, j, val);
            }
        }

        for val in &mut matrix.data {
            *val = (*val * 100.0).round() / 100.0;
        }

        Ok(matrix)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_generator_uniform_attract() {
        let mut generator = CustomGenerator {
            name: "Uniform Attract".into(),
            description: String::new(),
            expression: "i == j ? 0.0 : 0.5".into(),
            num_types: None,
            compiled: None,
        };

        let matrix = generator.generate(4).unwrap();
        assert_eq!(matrix.size, 4);
        for i in 0..4 {
            assert!((matrix.get(i, i)).abs() < 0.001);
        }
        for i in 0..4 {
            for j in 0..4 {
                if i != j {
                    assert!((matrix.get(i, j) - 0.5).abs() < 0.001);
                }
            }
        }
    }

    #[test]
    fn test_custom_generator_type_constraint_error() {
        let mut generator = CustomGenerator {
            name: "Fixed3".into(),
            description: String::new(),
            expression: "0.0".into(),
            num_types: Some(3),
            compiled: None,
        };

        let result = generator.generate(5);
        assert!(result.is_err());
    }

    #[test]
    fn test_custom_generator_parse_error() {
        let mut generator = CustomGenerator {
            name: "Bad".into(),
            description: String::new(),
            expression: "invalid_var + 1".into(),
            num_types: None,
            compiled: None,
        };

        let result = generator.generate(4);
        assert!(result.is_err());
    }

    #[test]
    fn test_custom_generator_cached_ast() {
        let mut generator = CustomGenerator {
            name: "Cache".into(),
            description: String::new(),
            expression: "i == j ? 0.0 : -0.3".into(),
            num_types: None,
            compiled: None,
        };

        let _ = generator.generate(4).unwrap();
        assert!(generator.compiled.is_some());

        let m2 = generator.generate(4).unwrap();
        assert_eq!(m2.size, 4);
    }
}

//! Procedural generators for rules, colors, and positions.

pub mod colors;
pub mod positions;
pub mod rules;

pub use colors::{ColorPalette, PaletteType};
pub use positions::{PositionPattern, SpawnConfig};
pub use rules::{RuleGenerator, RuleType};

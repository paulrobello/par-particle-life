//! Procedural generators for rules, colors, and positions.

pub mod colors;
pub mod custom;
pub mod expression;
pub mod positions;
pub mod rules;

pub use colors::{ColorPalette, PaletteType};
pub use custom::CustomGenerator;
pub use expression::{EvalContext, Expr, ExprError};
pub use positions::{PositionPattern, SpawnConfig};
pub use rules::{RuleGenerator, RuleType};

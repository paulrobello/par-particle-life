//! Procedural generators for rules, colors, and positions.

pub mod colors;
pub mod custom;
pub mod expression;
pub mod positions;
pub mod rules;

pub use colors::PaletteType;
pub use custom::CustomGenerator;
pub use expression::{EvalContext, Expr, ExprError};
pub use positions::{PositionPattern, SpawnConfig};
pub use rules::RuleType;

/// Deterministic CPU RNG used by every generator in this module.
///
/// Replaces the previous unseeded `rand::rng()` calls so generated matrices,
/// palettes, and spawn patterns are reproducible across runs for the same
/// inputs (ARC-019). All generators in `rules`, `colors`, and `positions`
/// draw from a `ChaCha8Rng` seeded with a fixed constant.
///
/// This intentionally does **not** reach the DSL evaluator's `random()`
/// function (in `expression.rs`); that path still uses `rand::random()`
/// pending a coordinated fix.
pub(crate) fn seeded_rng() -> rand_chacha::ChaCha8Rng {
    use rand::SeedableRng;
    // Fixed seed: same inputs always yield the same matrix/palette/pattern.
    // Bumping the seed is a one-line way to reshuffle every generator at once.
    rand_chacha::ChaCha8Rng::seed_from_u64(0x5EED_5EED_5EED_5EED)
}

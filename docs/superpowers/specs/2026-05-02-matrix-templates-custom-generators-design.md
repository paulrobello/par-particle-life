# Interaction Matrix Templates & User-Defined Custom Rule Generators

**Date:** 2026-05-02
**Status:** Draft

## Overview

Two additions to the rule generation system:

1. **Matrix Templates** — 3 new built-in `RuleType` variants for common matrix patterns
2. **Custom Rule Generators** — user-authored generators via JSON files with an embedded expression DSL

## Feature 1: Matrix Template Generators

### New RuleType Variants

Add 3 variants to `RuleType` in `src/generators/rules.rs` (values 31, 32, 33):

#### BlockDiagonal (value 31)

Alliance groups with positive intra-block, negative inter-block.

- Block count: 2 for ≤4 types, 3 for >4 types
- Within block: random positive values in [0.1, 0.8]
- Between blocks: random negative values in [-0.8, -0.1]
- Each block gets at least 1 type
- `display_name()`: "Block-Diagonal (Alliances)"
- `category()`: "Experimental"

#### CyclicPursuit (value 32)

Each type pursues the next in a cycle, creating rotating/vortex patterns.

- `matrix[i][(i+1) % n] = 0.8` (attraction to next)
- `matrix[i][(i+n-1) % n] = -0.5` (repel previous)
- Diagonal = 0
- All other cells: small random noise in [-0.1, 0.1]
- Requires ≥3 types; falls back to symmetric behavior for 2 types
- `display_name()`: "Cyclic Pursuit"
- `category()`: "Experimental"

#### RandomSparse (value 33)

Sparse interaction graph where most type pairs don't interact.

- ~70% of off-diagonal cells are zero
- Non-zero cells get random values in [-1, 1]
- Diagonal = 0
- `display_name()`: "Random Sparse"
- `category()`: "Experimental"

### Implementation Notes

- Each variant gets a private generator function following the existing 31-function pattern
- All values rounded to 2 decimal places (consistent with existing generators)
- No changes to `InteractionMatrix`, UI, or any other files for this feature alone

## Feature 2: Custom Rule Generators

### Expression DSL

A small recursive-descent parser in `src/generators/expression.rs`. ~200-250 lines, no external dependencies.

#### Grammar

```
expr      := ternary
ternary   := comparison ('?' expr ':' expr)?
comparison := additive (('==' | '!=' | '<' | '>' | '<=' | '>=') additive)?
additive  := multiplicative (('+' | '-') multiplicative)*
mult      := unary (('*' | '/') unary)*
unary     := '-' unary | primary
primary   := number | variable | function '(' args ')' | '(' expr ')'
```

#### Variables

| Variable | Type | Description |
|----------|------|-------------|
| `i` | usize | Row index (source type) |
| `j` | usize | Column index (target type) |
| `n` | usize | Total type count |

#### Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `abs(x)` | f32 → f32 | Absolute value |
| `sin(x)` | f32 → f32 | Sine (radians) |
| `cos(x)` | f32 → f32 | Cosine (radians) |
| `random()` | () → f32 | Uniform random in [-1, 1] |
| `min(a, b)` | (f32, f32) → f32 | Minimum |
| `max(a, b)` | (f32, f32) → f32 | Maximum |
| `pow(b, e)` | (f32, f32) → f32 | Exponentiation |

#### AST Representation

```rust
enum Expr {
    Literal(f32),
    Variable(Var),
    BinaryOp { op: BinOp, left: Box<Expr>, right: Box<Expr> },
    UnaryNeg(Box<Expr>),
    Ternary { cond: Box<Expr>, then_expr: Box<Expr>, else_expr: Box<Expr> },
    Call { func: Func, args: Vec<Expr> },
}
```

#### Error Handling

- `ExprError` enum: `ParseError(String)`, `EvalError(String)`
- Division by zero returns 0.0 with a warning (not a hard error) — keeps simulations running
- Custom generator errors shown in `preset_status`, matrix unchanged

### Custom Generator Storage

#### File Format

JSON files in `<data_dir>/par-particle-life/custom-generators/`:

```json
{
  "name": "Predator-Prey",
  "description": "Each type chases the next",
  "expression": "i == (j + 1) % n ? 0.8 : i == j ? 0.0 : -0.3",
  "num_types": null
}
```

Fields:
- `name` (required) — display name in the Rules dropdown
- `description` (optional, default "") — tooltip text
- `expression` (required) — DSL formula evaluated per cell
- `num_types` (optional) — fixed type count, or null for any count

#### CustomGenerator Struct

New file `src/generators/custom.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomGenerator {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub expression: String,
    #[serde(default)]
    pub num_types: Option<usize>,
    #[serde(skip)]
    pub compiled: Option<Expr>,
}
```

#### Directory Management

Mirrors the preset pattern:
- `CustomGenerator::custom_dir()` → `<data_dir>/par-particle-life/custom-generators/`
- `CustomGenerator::ensure_dir()` → creates directory tree
- `CustomGenerator::list()` → scans for `.json` files, deserializes, returns `Vec<CustomGenerator>`
- `CustomGenerator::save_to_file(&self, name: &str)` → writes pretty JSON

#### Generation

- Lazy compilation: first `generate()` call parses expression, caches AST in `compiled`
- `generate(&mut self, num_types: usize) -> Result<InteractionMatrix, ExprError>`
- If `num_types` is set and doesn't match, return error

### App State Changes

#### `src/app/state.rs`

- `App` gains `custom_generators: Vec<CustomGenerator>` field
- Loaded at startup in `App::new()`
- New method: `generate_custom_rules(index: usize, num_types: usize) -> Result<InteractionMatrix, ExprError>`

#### `src/app/handler/mod.rs`

New `RuleSelection` enum for tracking dropdown state:

```rust
enum RuleSelection {
    BuiltIn(RuleType),
    Custom(usize),
}
```

- `AppHandler` gains `rule_selection: RuleSelection` field (default: `BuiltIn(RuleType::Random)`)
- `regenerate_rules()` dispatches based on `rule_selection`

#### `src/generators/mod.rs`

- Add `pub mod expression;` and `pub mod custom;`
- Re-export `CustomGenerator` and `RuleSelection`

### UI Changes

#### Rules ComboBox (`src/app/handler/ui.rs`)

After iterating `RuleType::all()`:

1. If `custom_generators` is non-empty, add `egui::Separator()`
2. Iterate custom generators, show each by `name`
3. Selection tracked via `rule_selection` enum
4. On custom generator error, show message in `preset_status`, leave matrix unchanged

#### New Buttons in Generators Section

- **"Open Custom Generators Folder"** — opens directory in OS file manager
- **"Reload Custom Generators"** — re-scans directory, refreshes dropdown

No inline editor — users create/edit JSON files externally, consistent with preset workflow.

### Example Custom Generators

Users could create these files:

**Predator-Prey** (`predator-prey.json`):
```json
{
  "name": "Predator-Prey",
  "expression": "i == (j + 1) % n ? 0.8 : i == j ? 0.0 : -0.3"
}
```

**Uniform Attraction** (`uniform-attract.json`):
```json
{
  "name": "Uniform Attraction",
  "expression": "i == j ? 0.0 : 0.5"
}
```

**Distance-Based** (`distance-based.json`):
```json
{
  "name": "Distance-Based",
  "expression": "abs(i - j) == 1 ? 0.6 : i == j ? 0.0 : -0.2"
}
```

## Files Changed

| File | Change |
|------|--------|
| `src/generators/rules.rs` | Add 3 variants, 3 generator functions, update `all()`, `display_name()`, `category()` |
| `src/generators/expression.rs` | **New** — expression DSL parser and evaluator |
| `src/generators/custom.rs` | **New** — `CustomGenerator` struct, directory management, generation |
| `src/generators/mod.rs` | Add module declarations and re-exports |
| `src/app/state.rs` | Add `custom_generators` field, `generate_custom_rules()` method |
| `src/app/handler/mod.rs` | Add `RuleSelection` enum, `rule_selection` field, update `regenerate_rules()` |
| `src/app/handler/ui.rs` | Update Rules ComboBox, add custom generator buttons |
| `src/app/handler/events.rs` | Update keyboard shortcut M to dispatch via `rule_selection` |

## Out of Scope

- Inline DSL editor in the UI (users edit JSON files externally)
- Validation/linting of expressions before saving
- Sharing custom generators between users (beyond sharing JSON files)
- Custom position or color generators (future work)
- Template transform buttons (symmetrize/antisymmetrize current matrix — future work)

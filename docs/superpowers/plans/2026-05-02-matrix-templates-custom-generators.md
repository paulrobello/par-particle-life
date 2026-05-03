# Matrix Templates & Custom Rule Generators — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add 3 new matrix template generators and a user-defined custom rule generator system with an expression DSL.

**Architecture:** New generators extend the existing `RuleType` enum dispatch pattern. Custom generators are loaded from JSON files, parsed via a hand-written expression DSL, and integrated into the Rules ComboBox as a separate section. The expression parser is a standalone module with no external dependencies.

**Tech Stack:** Rust, serde (JSON), rand, egui

**Spec:** `docs/superpowers/specs/2026-05-02-matrix-templates-custom-generators-design.md`

---

## File Structure

| File | Action | Responsibility |
|------|--------|---------------|
| `src/generators/rules.rs` | Modify | Add 3 enum variants + generator functions |
| `src/generators/expression.rs` | Create | Expression DSL parser and evaluator |
| `src/generators/custom.rs` | Create | CustomGenerator struct, directory I/O, generation |
| `src/generators/mod.rs` | Modify | Add module declarations and re-exports |
| `src/app/state.rs` | Modify | Add `custom_generators` field and dispatch |
| `src/app/handler/mod.rs` | Modify | Add `RuleSelection` enum and field |
| `src/app/handler/ui.rs` | Modify | Update Rules ComboBox, add buttons |
| `src/app/handler/events.rs` | Modify | Update M key shortcut |

---

### Task 1: Add BlockDiagonal Rule Generator

**Files:**
- Modify: `src/generators/rules.rs`

- [ ] **Step 1: Add enum variant**

In `src/generators/rules.rs`, add `BlockDiagonal = 31` to the `RuleType` enum after `DriftedPatchwork = 30` (line 48):

```rust
    DriftedPatchwork = 30,
    BlockDiagonal = 31,
```

- [ ] **Step 2: Update `all()` method**

Add `RuleType::BlockDiagonal` at the end of the array in `all()` (after line 85):

```rust
            RuleType::DriftedPatchwork,
            RuleType::BlockDiagonal,
```

- [ ] **Step 3: Update `display_name()`**

Add match arm in `display_name()` (after line 122):

```rust
            RuleType::BlockDiagonal => "Block-Diagonal (Alliances)",
```

- [ ] **Step 4: Update `category()`**

No change needed — `_ => "Experimental"` already covers new variants.

- [ ] **Step 5: Add dispatch arm in `generate_rules()`**

Add match arm in the `match rule_type` block (after line 184):

```rust
        RuleType::BlockDiagonal => block_diagonal_generator(num_types),
```

- [ ] **Step 6: Implement `block_diagonal_generator()`**

Add before the `#[cfg(test)]` module (before line 925):

```rust
/// Block-Diagonal: alliance groups with positive intra-block, negative inter-block.
fn block_diagonal_generator(n: usize) -> InteractionMatrix {
    let mut rng = rand::rng();
    let mut matrix = InteractionMatrix::new(n);

    let num_blocks = if n <= 4 { 2 } else { 3 };
    let block_of = |t: usize| -> usize {
        let block_size = (n + num_blocks - 1) / num_blocks;
        (t / block_size).min(num_blocks - 1)
    };

    for i in 0..n {
        for j in 0..n {
            if i == j {
                matrix.set(i, j, 0.0);
            } else if block_of(i) == block_of(j) {
                let val: f32 = rng.random::<f32>() * 0.7 + 0.1; // [0.1, 0.8]
                matrix.set(i, j, val);
            } else {
                let val: f32 = -(rng.random::<f32>() * 0.7 + 0.1); // [-0.8, -0.1]
                matrix.set(i, j, val);
            }
        }
    }

    matrix
}
```

- [ ] **Step 7: Run tests**

Run: `cargo test --lib generators::rules`
Expected: All tests pass, including `test_all_generators_produce_valid_matrices` which iterates `RuleType::all()`.

- [ ] **Step 8: Run lint and format**

Run: `cargo fmt && cargo clippy --lib -- -D warnings`
Expected: No errors or warnings.

- [ ] **Step 9: Commit**

```bash
git add src/generators/rules.rs
git commit -m "feat: add BlockDiagonal rule generator"
```

---

### Task 2: Add CyclicPursuit and RandomSparse Rule Generators

**Files:**
- Modify: `src/generators/rules.rs`

- [ ] **Step 1: Add enum variants**

After `BlockDiagonal = 31`:

```rust
    BlockDiagonal = 31,
    CyclicPursuit = 32,
    RandomSparse = 33,
```

- [ ] **Step 2: Update `all()` method**

Add after `RuleType::BlockDiagonal`:

```rust
            RuleType::BlockDiagonal,
            RuleType::CyclicPursuit,
            RuleType::RandomSparse,
```

- [ ] **Step 3: Update `display_name()`**

Add match arms:

```rust
            RuleType::BlockDiagonal => "Block-Diagonal (Alliances)",
            RuleType::CyclicPursuit => "Cyclic Pursuit",
            RuleType::RandomSparse => "Random Sparse",
```

- [ ] **Step 4: Add dispatch arms**

```rust
        RuleType::BlockDiagonal => block_diagonal_generator(num_types),
        RuleType::CyclicPursuit => cyclic_pursuit_generator(num_types),
        RuleType::RandomSparse => random_sparse_generator(num_types),
```

- [ ] **Step 5: Implement `cyclic_pursuit_generator()`**

```rust
/// Cyclic Pursuit: each type chases the next in a cycle.
fn cyclic_pursuit_generator(n: usize) -> InteractionMatrix {
    let mut rng = rand::rng();
    let mut matrix = InteractionMatrix::new(n);

    if n < 3 {
        // Fallback to symmetric for < 3 types
        let mut m = random_generator(n);
        m.symmetrize();
        return m;
    }

    for i in 0..n {
        for j in 0..n {
            if j == i {
                matrix.set(i, j, 0.0);
            } else if j == (i + 1) % n {
                matrix.set(i, j, 0.8); // Attract next
            } else if j == (i + n - 1) % n {
                matrix.set(i, j, -0.5); // Repel previous
            } else {
                let noise: f32 = rng.random::<f32>() * 0.2 - 0.1; // [-0.1, 0.1]
                matrix.set(i, j, noise);
            }
        }
    }

    matrix
}
```

- [ ] **Step 6: Implement `random_sparse_generator()`**

```rust
/// Random Sparse: ~70% zero cells, rest random [-1, 1].
fn random_sparse_generator(n: usize) -> InteractionMatrix {
    let mut rng = rand::rng();
    let mut matrix = InteractionMatrix::new(n);

    for i in 0..n {
        for j in 0..n {
            if i == j {
                matrix.set(i, j, 0.0);
            } else if rng.random::<f32>() < 0.3 {
                let val: f32 = rng.random::<f32>() * 2.0 - 1.0;
                matrix.set(i, j, val);
            }
            // else stays 0.0
        }
    }

    matrix
}
```

- [ ] **Step 7: Run tests and lint**

Run: `cargo test --lib generators::rules && cargo fmt && cargo clippy --lib -- -D warnings`
Expected: All pass.

- [ ] **Step 8: Commit**

```bash
git add src/generators/rules.rs
git commit -m "feat: add CyclicPursuit and RandomSparse rule generators"
```

---

### Task 3: Create Expression DSL Parser

**Files:**
- Create: `src/generators/expression.rs`

- [ ] **Step 1: Create the expression module**

Create `src/generators/expression.rs` with the full parser and evaluator:

```rust
//! Expression DSL for user-defined custom rule generators.
//!
//! Supports: i, j, n variables; +, -, *, /, % operators;
//! ==, !=, <, >, <=, >= comparisons; ternary (?: );
//! abs, sin, cos, random, min, max, pow functions.

use std::fmt;

/// Expression AST node.
#[derive(Debug, Clone)]
pub enum Expr {
    Literal(f32),
    Var(Var),
    BinOp {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    UnaryNeg(Box<Expr>),
    Ternary {
        cond: Box<Expr>,
        then_expr: Box<Expr>,
        else_expr: Box<Expr>,
    },
    Call {
        func: Func,
        args: Vec<Expr>,
    },
}

/// Variable identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Var {
    I,
    J,
    N,
}

/// Binary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
}

/// Built-in function.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Func {
    Abs,
    Sin,
    Cos,
    Random,
    Min,
    Max,
    Pow,
}

/// Expression evaluation context.
#[derive(Debug, Clone, Copy)]
pub struct EvalContext {
    pub i: f32,
    pub j: f32,
    pub n: f32,
}

/// Expression error.
#[derive(Debug, Clone)]
pub enum ExprError {
    Parse(String),
    Eval(String),
}

impl fmt::Display for ExprError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExprError::Parse(msg) => write!(f, "Parse error: {msg}"),
            ExprError::Eval(msg) => write!(f, "Eval error: {msg}"),
        }
    }
}

impl std::error::Error for ExprError {}

impl Expr {
    /// Parse an expression string into an AST.
    pub fn parse(input: &str) -> Result<Self, ExprError> {
        let tokens = tokenize(input)?;
        let mut parser = Parser::new(&tokens);
        let expr = parser.parse_expr()?;
        if parser.pos < tokens.len() {
            return Err(ExprError::Parse(format!(
                "Unexpected token '{}' at position {}",
                tokens[parser.pos].kind.display(),
                parser.pos
            )));
        }
        Ok(expr)
    }

    /// Evaluate the expression with the given context.
    pub fn eval(&self, ctx: &EvalContext) -> Result<f32, ExprError> {
        match self {
            Expr::Literal(v) => Ok(*v),
            Expr::Var(Var::I) => Ok(ctx.i),
            Expr::Var(Var::J) => Ok(ctx.j),
            Expr::Var(Var::N) => Ok(ctx.n),
            Expr::BinOp { op, left, right } => {
                let l = left.eval(ctx)?;
                let r = right.eval(ctx)?;
                match op {
                    BinOp::Add => Ok(l + r),
                    BinOp::Sub => Ok(l - r),
                    BinOp::Mul => Ok(l * r),
                    BinOp::Div => {
                        if r == 0.0 {
                            Ok(0.0) // Division by zero returns 0, not error
                        } else {
                            Ok(l / r)
                        }
                    }
                    BinOp::Mod => {
                        if r == 0.0 {
                            Ok(0.0)
                        } else {
                            Ok(l % r)
                        }
                    }
                    BinOp::Eq => Ok(if l == r { 1.0 } else { 0.0 }),
                    BinOp::Ne => Ok(if l != r { 1.0 } else { 0.0 }),
                    BinOp::Lt => Ok(if l < r { 1.0 } else { 0.0 }),
                    BinOp::Gt => Ok(if l > r { 1.0 } else { 0.0 }),
                    BinOp::Le => Ok(if l <= r { 1.0 } else { 0.0 }),
                    BinOp::Ge => Ok(if l >= r { 1.0 } else { 0.0 }),
                }
            }
            Expr::UnaryNeg(inner) => Ok(-inner.eval(ctx)?),
            Expr::Ternary {
                cond,
                then_expr,
                else_expr,
            } => {
                let c = cond.eval(ctx)?;
                if c != 0.0 {
                    then_expr.eval(ctx)
                } else {
                    else_expr.eval(ctx)
                }
            }
            Expr::Call { func, args } => eval_func(*func, args, ctx),
        }
    }
}

fn eval_func(func: Func, args: &[Expr], ctx: &EvalContext) -> Result<f32, ExprError> {
    match func {
        Func::Abs => {
            let v = expect_arg(func, args, 1, ctx)?;
            Ok(v[0].abs())
        }
        Func::Sin => {
            let v = expect_arg(func, args, 1, ctx)?;
            Ok(v[0].sin())
        }
        Func::Cos => {
            let v = expect_arg(func, args, 1, ctx)?;
            Ok(v[0].cos())
        }
        Func::Random => {
            let _ = expect_arg(func, args, 0, ctx)?;
            Ok(rand::random::<f32>() * 2.0 - 1.0)
        }
        Func::Min => {
            let v = expect_arg(func, args, 2, ctx)?;
            Ok(v[0].min(v[1]))
        }
        Func::Max => {
            let v = expect_arg(func, args, 2, ctx)?;
            Ok(v[0].max(v[1]))
        }
        Func::Pow => {
            let v = expect_arg(func, args, 2, ctx)?;
            Ok(v[0].powf(v[1]))
        }
    }
}

fn expect_arg(
    func: Func,
    args: &[Expr],
    expected: usize,
    ctx: &EvalContext,
) -> Result<Vec<f32>, ExprError> {
    if args.len() != expected {
        return Err(ExprError::Eval(format!(
            "{func:?}() expects {expected} argument(s), got {}",
            args.len()
        )));
    }
    args.iter().map(|a| a.eval(ctx)).collect()
}

// === Tokenizer ===

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f32),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    Question,
    Colon,
    LParen,
    RParen,
    Comma,
}

impl Token {
    fn kind(&self) -> &Token {
        self
    }

    fn display(&self) -> &str {
        match self {
            Token::Number(_) => "number",
            Token::Ident(_) => "identifier",
            Token::Plus => "+",
            Token::Minus => "-",
            Token::Star => "*",
            Token::Slash => "/",
            Token::Percent => "%",
            Token::Eq => "==",
            Token::Ne => "!=",
            Token::Lt => "<",
            Token::Gt => ">",
            Token::Le => "<=",
            Token::Ge => ">=",
            Token::Question => "?",
            Token::Colon => ":",
            Token::LParen => "(",
            Token::RParen => ")",
            Token::Comma => ",",
        }
    }
}

fn tokenize(input: &str) -> Result<Vec<Token>, ExprError> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        if ch.is_whitespace() {
            chars.next();
            continue;
        }

        if ch.is_ascii_digit() || ch == '.' {
            let start = chars.clone();
            let mut num_str = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_ascii_digit() || c == '.' {
                    num_str.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            let val: f32 = num_str
                .parse()
                .map_err(|e| ExprError::Parse(format!("Invalid number '{num_str}': {e}")))?;
            tokens.push(Token::Number(val));
            continue;
        }

        if ch.is_ascii_alphabetic() || ch == '_' {
            let mut ident = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_ascii_alphanumeric() || c == '_' {
                    ident.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            tokens.push(Token::Ident(ident));
            continue;
        }

        match ch {
            '+' => { tokens.push(Token::Plus); chars.next(); }
            '-' => { tokens.push(Token::Minus); chars.next(); }
            '*' => { tokens.push(Token::Star); chars.next(); }
            '/' => { tokens.push(Token::Slash); chars.next(); }
            '%' => { tokens.push(Token::Percent); chars.next(); }
            '?' => { tokens.push(Token::Question); chars.next(); }
            ':' => { tokens.push(Token::Colon); chars.next(); }
            '(' => { tokens.push(Token::LParen); chars.next(); }
            ')' => { tokens.push(Token::RParen); chars.next(); }
            ',' => { tokens.push(Token::Comma); chars.next(); }
            '=' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::Eq);
                } else {
                    return Err(ExprError::Parse("Single '=' not supported, use '=='".into()));
                }
            }
            '!' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::Ne);
                } else {
                    return Err(ExprError::Parse("'!' without '=' is not supported".into()));
                }
            }
            '<' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::Le);
                } else {
                    tokens.push(Token::Lt);
                }
            }
            '>' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::Ge);
                } else {
                    tokens.push(Token::Gt);
                }
            }
            _ => {
                return Err(ExprError::Parse(format!("Unexpected character '{ch}'")));
            }
        }
    }

    Ok(tokens)
}

// === Parser ===

struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<&Token> {
        let tok = self.tokens.get(self.pos);
        self.pos += 1;
        tok
    }

    fn expect(&mut self, expected: &Token) -> Result<(), ExprError> {
        match self.advance() {
            Some(tok) if tok == expected => Ok(()),
            Some(tok) => Err(ExprError::Parse(format!(
                "Expected '{}', got '{}'",
                expected.display(),
                tok.display()
            ))),
            None => Err(ExprError::Parse(format!(
                "Expected '{}', got end of input",
                expected.display()
            ))),
        }
    }

    // expr := ternary
    fn parse_expr(&mut self) -> Result<Expr, ExprError> {
        self.parse_ternary()
    }

    // ternary := comparison ('?' expr ':' expr)?
    fn parse_ternary(&mut self) -> Result<Expr, ExprError> {
        let cond = self.parse_comparison()?;
        if self.peek() == Some(&Token::Question) {
            self.advance();
            let then_expr = self.parse_expr()?;
            self.expect(&Token::Colon)?;
            let else_expr = self.parse_expr()?;
            Ok(Expr::Ternary {
                cond: Box::new(cond),
                then_expr: Box::new(then_expr),
                else_expr: Box::new(else_expr),
            })
        } else {
            Ok(cond)
        }
    }

    // comparison := additive (('==' | '!=' | '<' | '>' | '<=' | '>=') additive)?
    fn parse_comparison(&mut self) -> Result<Expr, ExprError> {
        let left = self.parse_additive()?;
        let cmp_tokens = [
            Token::Eq, Token::Ne, Token::Lt, Token::Gt, Token::Le, Token::Ge,
        ];
        if let Some(tok) = self.peek() {
            if cmp_tokens.contains(tok) {
                let op = match self.advance().unwrap() {
                    Token::Eq => BinOp::Eq,
                    Token::Ne => BinOp::Ne,
                    Token::Lt => BinOp::Lt,
                    Token::Gt => BinOp::Gt,
                    Token::Le => BinOp::Le,
                    Token::Ge => BinOp::Ge,
                    _ => unreachable!(),
                };
                let right = self.parse_additive()?;
                return Ok(Expr::BinOp {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                });
            }
        }
        Ok(left)
    }

    // additive := multiplicative (('+' | '-') multiplicative)*
    fn parse_additive(&mut self) -> Result<Expr, ExprError> {
        let mut left = self.parse_multiplicative()?;
        loop {
            match self.peek() {
                Some(Token::Plus) | Some(Token::Minus) => {
                    let op = match self.advance().unwrap() {
                        Token::Plus => BinOp::Add,
                        Token::Minus => BinOp::Sub,
                        _ => unreachable!(),
                    };
                    let right = self.parse_multiplicative()?;
                    left = Expr::BinOp {
                        op,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }
        Ok(left)
    }

    // multiplicative := unary (('*' | '/' | '%') unary)*
    fn parse_multiplicative(&mut self) -> Result<Expr, ExprError> {
        let mut left = self.parse_unary()?;
        loop {
            match self.peek() {
                Some(Token::Star) | Some(Token::Slash) | Some(Token::Percent) => {
                    let op = match self.advance().unwrap() {
                        Token::Star => BinOp::Mul,
                        Token::Slash => BinOp::Div,
                        Token::Percent => BinOp::Mod,
                        _ => unreachable!(),
                    };
                    let right = self.parse_unary()?;
                    left = Expr::BinOp {
                        op,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }
        Ok(left)
    }

    // unary := '-' unary | primary
    fn parse_unary(&mut self) -> Result<Expr, ExprError> {
        if self.peek() == Some(&Token::Minus) {
            self.advance();
            let inner = self.parse_unary()?;
            Ok(Expr::UnaryNeg(Box::new(inner)))
        } else {
            self.parse_primary()
        }
    }

    // primary := number | variable | function '(' args ')' | '(' expr ')'
    fn parse_primary(&mut self) -> Result<Expr, ExprError> {
        match self.peek() {
            Some(Token::Number(_)) => {
                let val = match self.advance().unwrap() {
                    Token::Number(v) => v,
                    _ => unreachable!(),
                };
                Ok(Expr::Literal(val))
            }
            Some(Token::Ident(_)) => {
                let name = match self.advance().unwrap() {
                    Token::Ident(s) => s,
                    _ => unreachable!(),
                };
                // Check if it's a function call
                if self.peek() == Some(&Token::LParen) {
                    let func = match name.as_str() {
                        "abs" => Func::Abs,
                        "sin" => Func::Sin,
                        "cos" => Func::Cos,
                        "random" => Func::Random,
                        "min" => Func::Min,
                        "max" => Func::Max,
                        "pow" => Func::Pow,
                        _ => {
                            return Err(ExprError::Parse(format!(
                                "Unknown function '{name}'"
                            )));
                        }
                    };
                    self.advance(); // consume '('
                    let mut args = Vec::new();
                    if self.peek() != Some(&Token::RParen) {
                        args.push(self.parse_expr()?);
                        while self.peek() == Some(&Token::Comma) {
                            self.advance();
                            args.push(self.parse_expr()?);
                        }
                    }
                    self.expect(&Token::RParen)?;
                    Ok(Expr::Call { func, args })
                } else {
                    // Variable
                    let var = match name.as_str() {
                        "i" => Var::I,
                        "j" => Var::J,
                        "n" => Var::N,
                        _ => {
                            return Err(ExprError::Parse(format!(
                                "Unknown variable '{name}'"
                            )));
                        }
                    };
                    Ok(Expr::Var(var))
                }
            }
            Some(Token::LParen) => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                Ok(expr)
            }
            Some(tok) => Err(ExprError::Parse(format!(
                "Unexpected token '{}'",
                tok.display()
            ))),
            None => Err(ExprError::Parse("Unexpected end of input".into())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_str(expr: &str, i: f32, j: f32, n: f32) -> f32 {
        let parsed = Expr::parse(expr).unwrap();
        let ctx = EvalContext { i, j, n };
        parsed.eval(&ctx).unwrap()
    }

    #[test]
    fn test_literal() {
        assert!((eval_str("3.14", 0.0, 0.0, 0.0) - 3.14).abs() < 0.001);
    }

    #[test]
    fn test_variable_i() {
        assert!((eval_str("i", 2.0, 0.0, 0.0) - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_variable_j() {
        assert!((eval_str("j", 0.0, 3.0, 0.0) - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_variable_n() {
        assert!((eval_str("n", 0.0, 0.0, 7.0) - 7.0).abs() < 0.001);
    }

    #[test]
    fn test_arithmetic() {
        assert!((eval_str("i + j", 2.0, 3.0, 0.0) - 5.0).abs() < 0.001);
        assert!((eval_str("i - j", 5.0, 3.0, 0.0) - 2.0).abs() < 0.001);
        assert!((eval_str("i * j", 2.0, 3.0, 0.0) - 6.0).abs() < 0.001);
        assert!((eval_str("i / j", 6.0, 3.0, 0.0) - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_division_by_zero() {
        assert!((eval_str("1.0 / 0.0", 0.0, 0.0, 0.0)).abs() < 0.001);
    }

    #[test]
    fn test_unary_neg() {
        assert!((eval_str("-i", 3.0, 0.0, 0.0) - (-3.0)).abs() < 0.001);
    }

    #[test]
    fn test_comparison_eq() {
        assert!((eval_str("i == j", 2.0, 2.0, 0.0) - 1.0).abs() < 0.001);
        assert!((eval_str("i == j", 2.0, 3.0, 0.0)).abs() < 0.001);
    }

    #[test]
    fn test_ternary() {
        assert!((eval_str("i == j ? 0.5 : -0.3", 2.0, 2.0, 0.0) - 0.5).abs() < 0.001);
        assert!((eval_str("i == j ? 0.5 : -0.3", 2.0, 3.0, 0.0) - (-0.3)).abs() < 0.001);
    }

    #[test]
    fn test_nested_ternary() {
        let result = eval_str("i == j ? 0.0 : i == (j + 1) % n ? 0.8 : -0.3", 1.0, 2.0, 4.0);
        assert!((result - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_functions() {
        assert!((eval_str("abs(-0.5)", 0.0, 0.0, 0.0) - 0.5).abs() < 0.001);
        assert!((eval_str("min(i, j)", 2.0, 5.0, 0.0) - 2.0).abs() < 0.001);
        assert!((eval_str("max(i, j)", 2.0, 5.0, 0.0) - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_modulo() {
        assert!((eval_str("(j + 1) % n", 0.0, 2.0, 4.0) - 3.0).abs() < 0.001);
        assert!((eval_str("(j + 1) % n", 0.0, 3.0, 4.0)).abs() < 0.001);
    }

    #[test]
    fn test_parentheses() {
        assert!((eval_str("(i + j) * 2", 3.0, 4.0, 0.0) - 14.0).abs() < 0.001);
    }

    #[test]
    fn test_parse_error_unknown_variable() {
        assert!(Expr::parse("x + y").is_err());
    }

    #[test]
    fn test_parse_error_unknown_function() {
        assert!(Expr::parse("foo(1)").is_err());
    }

    #[test]
    fn test_parse_error_single_equals() {
        assert!(Expr::parse("i = j").is_err());
    }

    #[test]
    fn test_sin_cos() {
        let val = eval_str("sin(0)", 0.0, 0.0, 0.0);
        assert!(val.abs() < 0.001);
        let val = eval_str("cos(0)", 0.0, 0.0, 0.0);
        assert!((val - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_pow() {
        assert!((eval_str("pow(2, 3)", 0.0, 0.0, 0.0) - 8.0).abs() < 0.001);
    }
}
```

- [ ] **Step 2: Register module in mod.rs**

In `src/generators/mod.rs`, add after `pub mod rules;`:

```rust
pub mod expression;
pub mod custom;
```

Add to re-exports:

```rust
pub use custom::CustomGenerator;
pub use expression::{EvalContext, Expr, ExprError};
```

- [ ] **Step 3: Run tests**

Run: `cargo test --lib generators::expression`
Expected: All 15+ tests pass.

- [ ] **Step 4: Run lint and format**

Run: `cargo fmt && cargo clippy --lib -- -D warnings`
Expected: No errors or warnings.

- [ ] **Step 5: Commit**

```bash
git add src/generators/expression.rs src/generators/mod.rs
git commit -m "feat: add expression DSL parser for custom rule generators"
```

---

### Task 4: Create CustomGenerator Struct and Directory Management

**Files:**
- Create: `src/generators/custom.rs`

- [ ] **Step 1: Create the custom generator module**

Create `src/generators/custom.rs`:

```rust
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
                    Ok(gen) => generators.push(gen),
                    Err(e) => {
                        log::warn!(
                            "Failed to parse custom generator {}: {e}",
                            path.display()
                        );
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
        let filename = self
            .name
            .to_lowercase()
            .replace(' ', "-")
            .replace('/', "-");
        let path = dir.join(format!("{filename}.json"));
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)?;
        Ok(())
    }

    /// Generate an interaction matrix using this custom generator's expression.
    pub fn generate(&mut self, num_types: usize) -> Result<InteractionMatrix, ExprError> {
        // Validate type count constraint
        if let Some(required) = self.num_types {
            if num_types != required {
                return Err(ExprError::Eval(format!(
                    "'{}' requires exactly {} types, got {}",
                    self.name, required, num_types
                )));
            }
        }

        // Parse expression if not cached
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

        // Round to 2 decimal places, consistent with built-in generators
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
        let mut gen = CustomGenerator {
            name: "Uniform Attract".into(),
            description: String::new(),
            expression: "i == j ? 0.0 : 0.5".into(),
            num_types: None,
            compiled: None,
        };

        let matrix = gen.generate(4).unwrap();
        assert_eq!(matrix.size, 4);
        // Diagonal should be 0
        for i in 0..4 {
            assert!((matrix.get(i, i)).abs() < 0.001, "Diagonal should be 0");
        }
        // Off-diagonal should be 0.5
        for i in 0..4 {
            for j in 0..4 {
                if i != j {
                    assert!(
                        (matrix.get(i, j) - 0.5).abs() < 0.001,
                        "Off-diagonal should be 0.5 at ({i},{j})"
                    );
                }
            }
        }
    }

    #[test]
    fn test_custom_generator_type_constraint_error() {
        let mut gen = CustomGenerator {
            name: "Fixed3".into(),
            description: String::new(),
            expression: "0.0".into(),
            num_types: Some(3),
            compiled: None,
        };

        let result = gen.generate(5);
        assert!(result.is_err());
    }

    #[test]
    fn test_custom_generator_parse_error() {
        let mut gen = CustomGenerator {
            name: "Bad".into(),
            description: String::new(),
            expression: "invalid_var + 1".into(),
            num_types: None,
            compiled: None,
        };

        let result = gen.generate(4);
        assert!(result.is_err());
    }

    #[test]
    fn test_custom_generator_cached_ast() {
        let mut gen = CustomGenerator {
            name: "Cache".into(),
            description: String::new(),
            expression: "i == j ? 0.0 : -0.3".into(),
            num_types: None,
            compiled: None,
        };

        // First call parses
        let _ = gen.generate(4).unwrap();
        assert!(gen.compiled.is_some());

        // Second call uses cache
        let m2 = gen.generate(4).unwrap();
        assert_eq!(m2.size, 4);
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --lib generators::custom`
Expected: All 4 tests pass.

- [ ] **Step 3: Run lint and format**

Run: `cargo fmt && cargo clippy --lib -- -D warnings`
Expected: No errors or warnings.

- [ ] **Step 4: Commit**

```bash
git add src/generators/custom.rs
git commit -m "feat: add CustomGenerator struct with directory management and generation"
```

---

### Task 5: Integrate Custom Generators into App State

**Files:**
- Modify: `src/app/state.rs`
- Modify: `src/app/handler/mod.rs`

- [ ] **Step 1: Add custom_generators to App**

In `src/app/state.rs`, add import and field:

At line 8-10, add to the generators import:

```rust
use crate::generators::{
    colors::{Color, PaletteType, generate_colors},
    custom::CustomGenerator,
    positions::{PositionPattern, SpawnConfig, generate_positions},
    rules::{RuleType, generate_rules},
};
```

Add field to `App` struct (after `obstacles` at line 47):

```rust
    /// Obstacle zones that deflect particles.
    pub obstacles: Vec<Obstacle>,
    /// User-defined custom rule generators.
    pub custom_generators: Vec<CustomGenerator>,
```

- [ ] **Step 2: Load custom generators at startup**

In `App::new()`, after `let obstacles = Vec::new();` (line 119), add:

```rust
        let custom_generators = CustomGenerator::list().unwrap_or_else(|e| {
            log::warn!("Failed to load custom generators: {e}");
            Vec::new()
        });
```

Add `custom_generators` to the `Self { ... }` struct (after `obstacles,`):

```rust
            obstacles,
            custom_generators,
```

- [ ] **Step 3: Add custom generation dispatch method**

Add method to `App` impl block (after `regenerate_colors` at line 190):

```rust
    /// Generate rules from a custom generator, with error handling.
    pub fn generate_custom_rules(
        &mut self,
        index: usize,
    ) -> Result<InteractionMatrix, String> {
        let num_types = self.sim_config.num_types as usize;
        self.custom_generators
            .get_mut(index)
            .ok_or_else(|| format!("Custom generator index {index} out of range"))?
            .generate(num_types)
            .map_err(|e| e.to_string())
    }
```

- [ ] **Step 4: Add RuleSelection enum to AppHandler**

In `src/app/handler/mod.rs`, add import and enum before `AppHandler` struct:

```rust
use crate::generators::RuleType;

/// Tracks whether the current rule selection is a built-in type or custom generator.
#[derive(Debug, Clone)]
pub(crate) enum RuleSelection {
    BuiltIn(RuleType),
    Custom(usize),
}
```

Add field to `AppHandler` struct (after `step_requested` at line 106):

```rust
    /// When true, run exactly one frame then pause (step-by-step mode).
    pub(crate) step_requested: bool,
    /// Current rule selection (built-in or custom generator).
    pub(crate) rule_selection: RuleSelection,
```

- [ ] **Step 5: Initialize RuleSelection in AppHandler::new()**

In the `Self { ... }` block (after `step_requested: false,` at line 221), add:

```rust
            step_requested: false,
            rule_selection: RuleSelection::BuiltIn(app.current_rule),
```

- [ ] **Step 6: Run tests and lint**

Run: `cargo test --lib && cargo fmt && cargo clippy --lib -- -D warnings`
Expected: All pass.

- [ ] **Step 7: Commit**

```bash
git add src/app/state.rs src/app/handler/mod.rs
git commit -m "feat: integrate custom generators into app state and handler"
```

---

### Task 6: Update UI — Rules ComboBox and Custom Generator Buttons

**Files:**
- Modify: `src/app/handler/ui.rs`

- [ ] **Step 1: Update Rules ComboBox to include custom generators**

In `src/app/handler/ui.rs`, replace the Rules ComboBox section (lines 364-380) with:

```rust
                            // Rule type
                            let rule_label = match &self.rule_selection {
                                RuleSelection::BuiltIn(rt) => rt.display_name().to_owned(),
                                RuleSelection::Custom(idx) => {
                                    self.app.custom_generators.get(*idx)
                                        .map(|g| g.name.clone())
                                        .unwrap_or_else(|| "Custom (missing)".into())
                                }
                            };

                            let mut selection_changed = false;
                            let mut new_selection = self.rule_selection.clone();

                            egui::ComboBox::from_label("Rules")
                                .selected_text(&rule_label)
                                .show_ui(ui, |ui| {
                                    for &rule in RuleType::all() {
                                        let name = rule.display_name();
                                        let mut selected = matches!(&self.rule_selection, RuleSelection::BuiltIn(rt) if *rt == rule);
                                        if ui.selectable_label(selected, name).clicked() {
                                            new_selection = RuleSelection::BuiltIn(rule);
                                            selection_changed = true;
                                        }
                                    }

                                    if !self.app.custom_generators.is_empty() {
                                        ui.separator();
                                        for (idx, gen) in self.app.custom_generators.iter().enumerate() {
                                            let mut selected = matches!(&self.rule_selection, RuleSelection::Custom(i) if *i == idx);
                                            if ui.selectable_label(selected, &gen.name).clicked() {
                                                new_selection = RuleSelection::Custom(idx);
                                                selection_changed = true;
                                            }
                                        }
                                    }
                                });

                            if selection_changed {
                                self.rule_selection = new_selection;
                                match &self.rule_selection {
                                    RuleSelection::BuiltIn(rule) => {
                                        self.app.current_rule = *rule;
                                        self.app.config.gen_rule = *rule;
                                        self.app.regenerate_rules();
                                        self.sync_interaction_matrix();
                                    }
                                    RuleSelection::Custom(idx) => {
                                        match self.app.generate_custom_rules(*idx) {
                                            Ok(matrix) => {
                                                self.app.interaction_matrix = matrix;
                                                self.sync_interaction_matrix();
                                                self.preset_status.clear();
                                            }
                                            Err(e) => {
                                                self.preset_status = format!("Custom generator error: {e}");
                                            }
                                        }
                                    }
                                }
                            }
```

Note: the imports at the top of `ui.rs` need `RuleSelection`. Check the existing imports and add if needed:

```rust
use super::RuleSelection;
```

- [ ] **Step 2: Add custom generator buttons**

After the "Randomize Rules" button block (after line 385), add:

```rust
                            ui.horizontal(|ui| {
                                if ui.button("Open Custom Generators").clicked() {
                                    if let Ok(dir) = crate::app::Preset::ensure_dir() {
                                        // Use custom generators dir
                                        let dir = std::path::PathBuf::from(
                                            crate::generators::custom::CustomGenerator::custom_dir()
                                        );
                                        let _ = std::fs::create_dir_all(&dir);
                                        open_path(&dir);
                                    }
                                }
                                if ui.button("Reload").clicked() {
                                    self.app.custom_generators = crate::generators::custom::CustomGenerator::list()
                                        .unwrap_or_default();
                                }
                            });
```

Check if `open_path` is a local helper or if we need to inline the OS-specific `open`/`xdg-open`/`explorer` logic. Look at the existing "Open Presets Folder" button in `draw_presets_ui()` (around line 917-929) for the pattern — extract or reuse.

If there's no shared `open_path` helper, extract one from the presets folder code. Add a private helper method to `AppHandler`:

```rust
    fn open_in_file_manager(path: &std::path::Path) {
        #[cfg(target_os = "macos")]
        let _ = std::process::Command::new("open").arg(path).spawn();
        #[cfg(target_os = "linux")]
        let _ = std::process::Command::new("xdg-open").arg(path).spawn();
        #[cfg(target_os = "windows")]
        let _ = std::process::Command::new("explorer").arg(path).spawn();
    }
```

Then use `Self::open_in_file_manager(&dir)` instead of `open_path`.

- [ ] **Step 3: Run tests and lint**

Run: `cargo test --lib && cargo fmt && cargo clippy --lib -- -D warnings`
Expected: All pass.

- [ ] **Step 4: Commit**

```bash
git add src/app/handler/ui.rs
git commit -m "feat: add custom generators to Rules dropdown with management buttons"
```

---

### Task 7: Update Keyboard Shortcut and Final Integration

**Files:**
- Modify: `src/app/handler/events.rs`

- [ ] **Step 1: Update M key shortcut to use RuleSelection**

In `src/app/handler/events.rs`, replace the M key handler (lines 137-140):

```rust
                    PhysicalKey::Code(KeyCode::KeyM) => {
                        match &self.rule_selection {
                            RuleSelection::BuiltIn(_) => {
                                self.app.regenerate_rules();
                                self.sync_interaction_matrix();
                            }
                            RuleSelection::Custom(idx) => {
                                match self.app.generate_custom_rules(*idx) {
                                    Ok(matrix) => {
                                        self.app.interaction_matrix = matrix;
                                        self.sync_interaction_matrix();
                                        self.preset_status.clear();
                                    }
                                    Err(e) => {
                                        self.preset_status = format!("Custom generator error: {e}");
                                    }
                                }
                            }
                        }
                    }
```

- [ ] **Step 2: Update module docstring in rules.rs**

Update the module doc comment at line 1-5 from "31 different algorithms" to "34 different algorithms":

```rust
//! Rule generators for creating interaction matrices.
//!
//! This module contains 34 different algorithms for generating
//! particle interaction matrices, ranging from simple random
//! patterns to complex mathematical constructs.
```

- [ ] **Step 3: Update ideas.md**

Remove the two completed items from `ideas.md`:
- Remove "### Interaction Matrix Templates" section (lines 23-24)
- Remove "### User-Defined Custom Rule Generators" section (lines 17-19)

- [ ] **Step 4: Update CHANGELOG.md**

Add entries under the `[Unreleased]` section:

```
### Added
- Interaction Matrix Templates: BlockDiagonal, CyclicPursuit, RandomSparse rule generators
- Custom Rule Generators: user-defined generators via JSON files with expression DSL
- Expression DSL supporting i, j, n variables, arithmetic, comparisons, ternary, and functions (abs, sin, cos, random, min, max, pow)
- "Open Custom Generators" and "Reload" buttons in the Generators UI section
```

- [ ] **Step 5: Run full check**

Run: `make checkall`
Expected: All format, lint, and test steps pass.

- [ ] **Step 6: Manual test**

Run: `make run`
Verify:
1. Rules dropdown shows 34 built-in generators including Block-Diagonal, Cyclic Pursuit, Random Sparse
2. Custom generators section appears after built-in generators (empty initially)
3. "Open Custom Generators" button opens the directory
4. Create a test JSON file in the custom-generators directory, click "Reload", verify it appears in dropdown
5. Select custom generator, verify matrix is generated correctly
6. Press M key with custom generator selected, verify re-generation

- [ ] **Step 7: Commit**

```bash
git add src/app/handler/events.rs src/generators/rules.rs ideas.md CHANGELOG.md
git commit -m "feat: complete matrix templates and custom generators integration"
```

---

## Self-Review Checklist

**Spec coverage:**
- [x] BlockDiagonal generator → Task 1
- [x] CyclicPursuit generator → Task 2
- [x] RandomSparse generator → Task 2
- [x] Expression DSL parser → Task 3
- [x] CustomGenerator struct + directory management → Task 4
- [x] App state integration → Task 5
- [x] RuleSelection enum → Task 5
- [x] UI Rules ComboBox update → Task 6
- [x] Open Custom Generators button → Task 6
- [x] Reload Custom Generators button → Task 6
- [x] Keyboard shortcut M update → Task 7
- [x] Error display via preset_status → Task 6

**Placeholder scan:** No TBDs, TODOs, or vague requirements found.

**Type consistency:**
- `RuleSelection::BuiltIn(RuleType)` / `RuleSelection::Custom(usize)` used consistently across Tasks 5-7
- `CustomGenerator::generate()` returns `Result<InteractionMatrix, ExprError>` consistently
- `App::generate_custom_rules()` wraps error as `Result<InteractionMatrix, String>` consistently

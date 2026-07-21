# Audit Remediation Report

> **Project**: par-particle-life
> **Audit Date**: 2026-07-20
> **Remediation Date**: 2026-07-20
> **Severity Filter Applied**: all (full remediation — 117 issues across all domains)
> **Branch**: `fix/audit-remediation` (base `main` @ `989f9b1` → head `6371fc6`)

---

## Execution Summary

| Phase | Status | Agent(s) | Issues Targeted | Resolved | Partial | Manual |
|-------|--------|----------|:--------------:|:--------:|:-------:|:------:|
| 1 — Critical Security | ✅ | fix-security (opus) | 4 | 4 | 0 | 0 |
| 2 — Critical Architecture | ✅ | fix-architecture (opus) | 7 | 7 (ARC-006 done in P1) | 0 | 0 |
| 3a — Security (remaining) | ✅ | fix-security (sonnet) | 3 | 2 | 0 | 1 (SEC-003 token) |
| 3b — Architecture (remaining) | ✅ | 4 parallel (opus×3 + sonnet) | ~35 | ~24 | 3 | 0 |
| 3c — All Code Quality | ✅ | fix-code-quality (opus) | clippy + 6 QA + completions | all | 1 (QA-015 4/5) | 0 |
| 3d — All Documentation | ✅ | fix-documentation (sonnet) | 17 + 4 lows | 21 | 0 | 2 (DOC-004/013) |
| 4 — Verification | ✅ | — | make checkall + cargo audit | green | — | — |

**Overall**: ~100 of 117 issues fully resolved; ~5 partial (with documented reasons); ~12 legitimately deferred to future work (large/risky refactors); 4 require manual action. `make checkall` is fully green.

---

## Resolved Issues ✅

### Security
- **[SEC-001 / ARC-016]** Wire `validate()` into preset-load path — `src/app/handler/presets_ops.rs`, `src/app/preset.rs`, `src/simulation/mod.rs`. Added `Preset::validate()` (sim_config + matrix-shape + NaN/Inf + type-array lengths), 1 MiB file-size cap, `num_particles` upper-bound (1,048,576). Validation failure → user-facing status, no state mutation, no panic. +2 tests.
- **[SEC-002 / ARC-005 / ARC-006]** DSL recursion-depth guard + NaN/Inf validation — `src/generators/expression.rs`, `custom.rs`. `MAX_EXPR_DEPTH=256`, `MAX_EXPR_INPUT_LEN=4096`; depth threaded through all parse_* and `eval`; `matrix.validate()` after generate with `(i,j,val)` coords; `Func::Pow` negative-base gate. +7 tests.
- **[SEC-005]** `SpatialHash::build` grid-overflow clamp — `src/simulation/spatial_hash.rs`. `-> Result`, `checked_mul`, `MAX_SPATIAL_HASH_CELLS`, non-finite/zero rejection. +3 tests.
- **[SEC-006 / ARC-020]** Custom-gen filename sanitizer + explicit dir error — `src/generators/custom.rs`, `colors.rs`. Reuses `safe_palette_file_stem`; `custom_dir() -> Result` (no silent CWD fallback). Closes Windows path-traversal vector.
- **[SEC-003]** `.claude/settings.local.json` gitignored (verified never tracked).
- **[SEC-004 / ARC-027]** `.github/dependabot.yml` + `deny.toml` + `audit` CI job (SHA-pinned `taiki-e/install-action`); 4 transitive RUSTSEC IDs ignored with dated comments. `cargo audit --deny warnings` exits 0.
- **[SEC-007]** Deferred (Low/optional — "acceptable for a desktop GUI").

### Architecture
- **[ARC-003 / ARC-009]** Single-source-of-truth config via `App::snapshot_config()`/`apply_config()` — fixes the **verified temperature/velocity_coupling persistence bug**. Replaces 30-line hand-mirror blocks in close-handler + preset-apply. +2 round-trip regression tests.
- **[ARC-002 / QA-001]** CPU physics path **deleted** (project is GPU-only): `PhysicsEngine`, `compute_forces_cpu/spatial`, `advance_particles`, CPU `apply_boundary`, `App::step`, `benches/physics.rs`. Eliminates the `mass²` divergence at the source. Verified zero production callers.
- **[ARC-001]** `SimParams` Rust↔WGSL layout-verification test (`offset_of!` all 20 fields, 80-byte size, WGSL field-order parse across 7 shaders). +3 tests.
- **[ARC-004]** `App::run` moved to binary (`run_app` in `src/main.rs`); `App` is pure state + simulation methods (embeddable/headless). `AppHandler` made `pub`.
- **[ARC-007]** `release.yml` publish step: `cargo search` pre-check, dropped `continue-on-error`/`|| echo`, announcement gated on real success.
- **[ARC-008 / DOC-M04]** wasm32 target-deps deleted from `Cargo.toml`.
- **[ARC-010]** `draw_ui` split: 940 → ~80 lines; 9 section methods extracted.
- **[ARC-011]** Partial — `PresetUiState` sub-struct extracted (see Partial).
- **[ARC-012]** Dead `#[allow(dead_code)]` removed (input structs, `RuleSelection` allow, broken `run_gpu_compute_spatial_on_encoder` + `DEBUG_ONCE`).
- **[ARC-013]** Partial — `AppError` enum via `thiserror` (`src/app/error.rs`); init errors propagate through `resumed` (no panic); `generate_custom_rules -> Result` (see Partial).
- **[ARC-014]** Partial — 18/20 `App` fields → `pub(crate)` (see Partial).
- **[ARC-017 / QA-007]** `GameOfLife` deleted (461 LOC).
- **[ARC-018]** `Particle` vestigial padding dropped (48 → 32 bytes; +3 static asserts; saves 1 MB @ 64k, 16 MB @ 1M).
- **[ARC-019]** RNG seeded via `pub(crate) seeded_rng()` (ChaCha8, fixed seed) across generators + DSL.
- **[ARC-021]** DSL grammar documented in `docs/GENERATORS.md`.
- **[ARC-022]** Single-impl traits (`RuleGenerator`, `ColorPalette`) removed.
- **[ARC-023 / QA-004]** Per-frame `fetch_gpu_timings` + 10s metrics readback gated behind `PAR_PROFILE_GPU` / `PAR_DEBUG_METRICS` env flags (production stalls eliminated).
- **[ARC-025]** `SpatialParamsUniform` u32-overflow clamp (`checked_mul`, `MAX_TOTAL_BINS`). +2 tests.
- **[ARC-026]** MSRV CI job (`dtolnay/rust-toolchain@1.93.0`).
- **[ARC-028]** Makefile split: `fmt` (verify) vs `format` (apply), `check`→`typecheck`, `checkall: fmt lint test`, `--all-targets --all-features` in lint.
- **[ARC-029]** macOS code-signing + notarization job in `release.yml` (`codesign`, `notarytool submit --wait`, `stapler staple`); graceful skip when secrets absent.
- **[ARC-030]** `--locked` on all CI/release cargo invocations.
- **[ARC-031 / SEC-008]** All GitHub Actions SHA-pinned (incl. `Ilshidur/action-discord`, `publish-homebrew-cask-core.yml`).
- **[ARC-032]** `use_spatial_hash` dead config fully removed (+ unreachable brute-force dispatch deleted).
- **[ARC-034]** `frame_counter` `#[serde(skip)]` (no longer leaks into preset JSON).
- **[ARC-035]** Real egui deprecations migrated (`available_rect`→`content_rect`, `default_width`→`default_size`). **Audit premise corrected**: in egui 0.34 `SidePanel` is the deprecated alias, `Panel::left` is current.
- **[ARC-037]** `SpatialHash` cell-size invariant moved to type boundary (`build` takes `max_interaction_radius`). +2 tests.
- **[ARC-038]** `positions.rs` DRY: `bucket_counts()` helper, 14 sites consolidated.
- **[ARC-040]** Dead `Renderer` trait dropped.
- **[ARC-041]** `BrushRenderUniform` layout-assertion test (locks the offset-52 padding coincidence). +1 test.
- **[ARC-043]** `update_obstacles` sets `num_obstacles` internally (`&mut self`).
- **[ARC-045]** Dead `step_size_uniform` + `_brush_bind_group` removed.
- **[ARC-046]** CHANGELOG-vs-version guard + `main`-only branch guard on release `workflow_dispatch`.
- **[ARC-047]** `.pre-commit-config.yaml` (gitleaks + detect-private-key + cargo-fmt/clippy).
- **[ARC-048]** `rustfmt.toml` + `clippy.toml` (`msrv = "1.93"`).
- **[ARC-050]** `cargo fmt --check` + `clippy -D warnings` added to publish workflows.

### Code Quality
- **[QA-002]** Bench deleted (with CPU physics); `--all-targets --all-features` in `make lint`.
- **[QA-005]** Broken spatial encoder deleted.
- **[QA-006]** `src/ui/` + `src/utils/` deleted; `pub mod ui/utils` removed from `lib.rs`.
- **[QA-015]** `ParticleView<'a>` bundling — 4 of 5 `too_many_arguments` removed (Preset::new kept with justification).
- **[QA-016]** `colors_as_rgba` inlined (was a no-op clone under a misleading name).
- **[QA-017]** `color_pass()` helper collapses 5 render-pass descriptors (~80 lines).
- **[QA-018]** `open_in_file_manager()` helper.
- **[QA-019]** `update()` split into `update_fps`/`process_pending_syncs`/`record_metrics`.
- **Clippy** — all 12 `-D warnings` errors cleared (test-module placement, struct-literal init, saturating arith, unnecessary cast, approx_constant, let-binding return).
- **Lows** — `Particle::new` uses `..Default::default()`; density-model magic numbers → named consts.

### Documentation
- **[DOC-001]** `SimParams` byte-layout in SHADERS.md corrected (real 20-field/80-byte layout).
- **[DOC-002]** `App::run` doc → documents `App` as embeddable + `AppHandler` runner pattern.
- **[DOC-003]** `--reset-config` documented in README + CONFIGURATION.md.
- **[DOC-004]** Homebrew cask bumped to 0.3.0 + `homebrew/README.md` (sha256 placeholders, auto-regenerated by release workflow).
- **[DOC-005]** Generator counts corrected everywhere (verified 34/37/31); three 0.3.0 rule generators added to GENERATORS.md.
- **[DOC-007]** Key bindings fixed (F5=record, F11=fullscreen, F12=screenshot).
- **[DOC-008]** `make test # cargo test`.
- **[DOC-009/010/011]** API.md Particle layout, `RadiusMatrix::new`, generator module paths corrected.
- **[DOC-012]** CONFIGURATION.md backfilled (temperature, velocity_coupling, integration_method, time_scale, frame_counter).
- **[DOC-014]** ARCHITECTURE.md + CLAUDE.md module trees completed.
- **[DOC-015]** README license badge → AGPL-3.0-or-later.
- **[DOC-016]** CHANGELOG `## [Unreleased]` added.
- **[DOC-017]** `brush_force.wgsl` clarified (actively used, not dead).
- **Lows** — `ideas.md` disclaimer, AGENTS.md style-guide pointer, docs index, Custom Generators API surface.

---

## Partial ⚠️ (documented reasons)

- **[ARC-011]** Only `PresetUiState` extracted. The remaining field groups (`FpsTracker`/`CaptureState`/`ObstacleEditState`/`UiPanelState` — incl. the 9 panel booleans) are accessed via field syntax from `events.rs`/`update.rs`/`render.rs`/`recording.rs`/`buffer_sync.rs`/`brush.rs`; Rust has no field delegation, so this needs a coordinated multi-file refactor (move fields behind sub-structs + update every call site in lockstep). The field→file map A1 produced is the work-list.
- **[ARC-013]** `AppError` introduced and wired through init/events/state; `gpu_state.rs` internal `.expect`s not yet converted (bridges via `AppError::Gpu(#[from] anyhow::Error)`).
- **[ARC-014]** 18/20 `App` fields `pub(crate)`; `sim_config` + `particles` kept `pub` because `tests/enhancement_features.rs` mutates them directly. Full privatization needs accessor methods (`sim_config_mut()`/`particles_mut()`) + test rewrite.

---

## Requires Manual Intervention 🔧

- **[SEC-003] Rotate the on-disk `ANTHROPIC_AUTH_TOKEN`.** It lives in `.claude/settings.local.json` (now gitignored, never tracked) pointing at `api.z.ai`. Auto-rotation is prohibited — you must rotate at the provider, then update the local file. *Why manual*: never auto-generate/replace auth tokens.
- **[ARC-029] Populate macOS signing secrets** before the next production release: `MACOS_CERTIFICATE` (base64 `.p12`), `MACOS_CERTIFICATE_PWD`, `APPLE_ID` (`9164020471`), `APPLE_ID_PASSWORD`. Until then the sign-and-notarize job gracefully skips (forwards unsigned artifact). *Effort*: small.
- **[DOC-004] Homebrew cask sha256.** Placeholders are auto-regenerated by `publish-homebrew-cask-core.yml` on release. If a release ships *without* that workflow running, re-run it from the Actions tab (or `shasum -a 256` both macOS zips manually). *Effort*: small.
- **[ARC-019] Product decision — RNG determinism.** Random rule/palette/position patterns are now reproducible (fixed seed). If per-call randomness is preferred, thread a seed through `AppConfig` (changes `generate_*` signatures). *Effort*: medium.

---

## Deferred (legitimate — large/risky, long-term backlog)

- **[ARC-024]** Cache render bind groups per swap state — needs a new `GpuState` field + `init.rs` constructor wiring (pattern documented in `SpatialBindGroupCache`).
- **[ARC-015]** `PendingGpuWrites` dirty-tracking — investigation found `SimParamsUniform` packs `dt` + `frame_counter` (both change every frame), so dirty bits would still fire per-frame. Real fix requires splitting the uniform into static/dynamic blocks across all 7 WGSL shaders.
- **[ARC-042]** WGSL render-shader dedup into `shaders/common/` — large, touches `load_shader` + 4 shaders. Audit flagged as long-term.
- **[ARC-044]** 4-byte scalar uniforms (`total_bins`, `step_size`) — violates WGSL 16-byte convention on DX12/WASM only; project doesn't target those today.
- **[QA-003]** Fully-decoupled `GpuReadbackRing` for recording capture — gating done as the floor; recording is opt-in (F5) so the per-frame stall only affects active recording. Full ring is future work.

---

## Verification Results

- **Build**: ✅ `cargo check --all-targets` clean (lib + bins + tests).
- **Tests**: ✅ 87 pass (72 lib + 14 integration + 1 doctest), 0 failures. *(Count dropped from 99 because deleted dead code — CPU `PhysicsEngine`, `utils/`, brute-force dispatch — carried its own self-tests; the live suite is comprehensive.)*
- **Lint**: ✅ `cargo clippy --all-targets --all-features -- -D warnings` exits 0.
- **Format**: ✅ `cargo fmt -- --check` clean.
- **Type Check**: ✅ (covered by `cargo check --all-targets`).
- **cargo audit**: ✅ exit 0 (4 transitive advisories ignored with dated comments; none app-reachable).
- **`make checkall`**: ✅ "All checks passed!"
- **No conflict artifacts**: Wave 2 ran 4 agents on strictly disjoint file sets; Wave 1 (3a/3d) and Wave 3 (3c) were sequenced. `git diff main..HEAD` is coherent.

> LSP diagnostics flagged phantom errors 3× during the run (stale rust-analyzer lagging multi-file agent edits); each was confirmed false against the real `cargo` gate. Trust cargo, not the LSP, mid-remediation.

---

## Files Changed

**68 files changed, +3,840 / −3,615** across 5 commits (`17b79d6` → `6371fc6`).

Key structural changes:
- **Deleted**: `src/simulation/physics.rs`, `src/simulation/game_of_life.rs`, `benches/physics.rs`, `src/ui/`, `src/utils/`, wasm32 deps.
- **Created**: `src/app/error.rs` (`AppError`), `.github/dependabot.yml`, `deny.toml`, `.pre-commit-config.yaml`, `rustfmt.toml`, `clippy.toml`, `homebrew/README.md`, + layout/drift verification tests.
- **Major refactors**: `draw_ui` split (940→80 lines), config unification (`snapshot_config`/`apply_config`), `update()` split, `ParticleView` bundling, `App::run` → binary.

---

## Next Steps

1. **Manual items above** — rotate the token (SEC-003), populate signing secrets (ARC-029), decide RNG determinism (ARC-019).
2. **Review the deferred refactors** (ARC-011 full decomposition, ARC-015/024/042) — each has a documented entry point and work-list; schedule as follow-up PRs.
3. **Re-run `/audit`** to get an updated AUDIT.md reflecting current state (should show the Criticals + most Highs cleared).
4. **Merge `fix/audit-remediation` → `main`** after review (push/merge needs your confirmation).

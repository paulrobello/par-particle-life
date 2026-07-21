# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

Audit-driven remediation pass. No user-facing API additions; the changes below are correctness, hardening, and documentation.

### Added

- `SimulationConfig::validate()` now enforces the documented `num_particles` upper bound (1,048,576) to prevent crafted-preset OOM.
- Compile-time `SimParams` Rust↔WGSL layout verification test (`sim_params_uniform_field_offsets_match_wgsl_layout`) covering all 20 fields and the 80-byte total size.
- Depth guard in the custom-generator expression DSL (`expression too deeply nested` past ~256 levels) to prevent stack overflow on hostile or corrupt custom-generator files.
- NaN/Infinity validation on custom-generator matrix output (surfaces the offending `(i, j)` cell instead of silently poisoning the GPU compute shader).
- `SpatialHash::build` integer-overflow clamp on the grid bounds.
- Homebrew cask README documenting the release-time regeneration flow.
- Documentation: `--reset-config` CLI flag, generator counts corrected to 34/37/31, F5/F11/F12 key bindings corrected, SimParams byte-layout corrected, `Particle` and `RadiusMatrix` API signatures corrected, `App`/`AppHandler` split documented, custom-generator DSL section added.

### Changed

- `App` is now pure state plus simulation methods — `App::run` was moved out of the library. The GUI runner is `run_app(reset_config: bool)` in `src/main.rs`, driven by `AppHandler::new(...)` + `winit::EventLoop`. The library can be embedded headlessly without pulling in winit/egui.
- Config persistence unified through `App::snapshot_config()` / `App::apply_config()`; the close handler, preset-apply, and reset paths all funnel through these instead of hand-maintained field-by-field mirrors.
- Preset-load path now runs `SimulationConfig::validate()` plus a matrix-shape check before assigning into live state.
- `crates.io` publish step in `release.yml` no longer swallows failures with `continue-on-error`; success notifications are now gated on actual publish success.
- `homebrew/Casks/par-particle-life.rb` bumped to 0.3.0; sha256 placeholders are rewritten by the `Publish Homebrew Cask (core)` workflow on every release.
- MSRV documented as 1.93 (was incorrectly stated as 1.88 across README/CLAUDE).

### Removed

- CPU `PhysicsEngine` (the `physics.rs` brute-force/spatial paths and `App::step`) — the project is GPU-only; the CPU path was dead in production and diverged from the GPU path. The `benches/physics.rs` benchmark that depended on it was removed.
- wasm32 target dependencies (`wasm-bindgen`, `web-sys`, `console_error_panic_hook`) — the project never supported web builds; the manifest entry was misleading and the GPU init path is incompatible with wasm.

### Fixed

- Persisted `temperature` and `velocity_coupling` no longer silently reset to defaults on every normal quit (the close handler now round-trips through `snapshot_config`).

## [0.3.0] - 2026-05-03

### Added

- **Simulation Speed Control**: Time-scale slider (0.1x–5.0x) for slow-motion and fast-forward. Step-by-step mode to advance one frame at a time while paused.
- **Velocity Coupling**: Alignment force based on relative neighbor velocity (0.0–1.0), enabling boid-like flocking behavior. Implemented in both GPU compute shaders and CPU fallback.
- **Velocity Verlet Integration**: Optional integration mode alongside Euler, with UI/config/preset support and GPU/CPU advancement paths.
- **Obstacle Regions**: GPU-accelerated circular and rectangular obstacle zones that deflect particles with configurable bounce strength. Up to 16 obstacles with per-obstacle position, size, and bounce controls. Semi-transparent egui overlay rendering. Included in preset save/load.
- **Interactive Obstacle Brush Tool**: New Obstacle brush tool for click-to-place, drag-to-move, and scroll-to-resize obstacles. Shape preview cursor (circle/rectangle) follows mouse. Selected obstacle highlighted in yellow. Shape and bounce controls in brush tools panel.
- **Particle Count Hot-Swap**: Changing particle count in the UI now preserves existing simulation state when GPU buffers have capacity (up to 128k). Only newly appended particles are written to GPU.
- **GPU-Driven Draw Brush**: Draw brush spawning now uses a GPU compute shader instead of CPU readback, eliminating per-frame blocking during drawing.
- **Tooltips**: Hover tooltips on simulation, physics, rendering, and brush tool settings.
- Interaction Matrix Templates: BlockDiagonal, CyclicPursuit, RandomSparse rule generators
- Custom Rule Generators: user-defined generators via JSON files with expression DSL
- Expression DSL supporting i, j, n variables, arithmetic, comparisons, ternary, and built-in functions
- "Open Custom Generators" and "Reload" buttons in the Generators UI section
- Rules dropdown now uses display_name() for all generators
- Time-varying interaction matrices with oscillation/drift modes, amplitude and speed controls, and preset/config persistence
- Gradient position generators: Linear Gradient, Radial Gradient, and Angular Gradient
- Custom color palette editor with save/load support under the presets directory
- Drag-and-drop preset loading for `.json` preset files dropped onto the window
- Fullscreen mode via F11 and a UI button; recording shortcut moved to F5

### Changed

- Obstacle panel simplified: removed position sliders and add buttons (replaced by interactive brush tool)
- Brush tool labels now display name only (removed icon prefix from Erase)
- Obstacle shape icons in panel rendered with egui painter instead of unicode

## [0.2.0] - 2026-05-02

### Added

- Temperature: Brownian noise (0–50) with PCG hash in GPU shader
- Per-type mass (0.1–10.0) for asymmetric force response
- Variable particle size per type (0.1–5.0) across all render pipelines

## [0.1.0] - 2025-12-02

### Added

- GPU-accelerated particle life simulation using wgpu compute shaders
- 31 rule generators, 37 color palettes, 28 position patterns
- Spatial hashing for O(n·k) neighbor queries (CPU rayon + GPU compute)
- Double-buffered SoA particle storage with ping-pong pattern
- egui immediate-mode UI for live parameter tuning
- Video recording via ffmpeg pipe (MP4/WebM/GIF)
- Preset save/load system
- Glow rendering with configurable intensity, size, and steepness
- Mirror-wrap and infinite boundary modes
- Brush tools for drawing and erasing particles
- Screenshot capture (F11)
- Homebrew cask distribution
- Makefile deploy target for release pipeline

### Changed

- Migrated wgpu 27→29, egui 0.33→0.34, glam 0.31→0.32, rand 0.9→0.10
- Updated cask description

### Infrastructure

- graphify knowledge graph integration

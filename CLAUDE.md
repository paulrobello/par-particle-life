# CLAUDE.md

Guidance for Claude Code when working with this repository.

## Project Overview

Par Particle Life is a GPU-accelerated particle life simulation in Rust. Particles of different types attract/repel based on interaction matrices, creating emergent life-like behaviors.

**Key Technologies:** wgpu (GPU compute/render), egui (UI), glam (math)

**Reference:** Ported from `../SandboxScience` (TypeScript/WebGPU)

## Quick Reference

```bash
# Development workflow
make run          # Build and run release
make checkall     # Format, lint, test (run before commits)

# Individual checks
make format       # rustfmt
make lint         # clippy (warnings as errors)
make test         # cargo test
```

## Architecture Overview

- **`src/main.rs`** - Binary entry point (clap CLI, runs the winit event loop via `AppHandler`)
- **`src/lib.rs`** - Library root, public exports
- **`src/app/`** - Application state, config, presets, input handling
  - `state.rs` - Core App struct and simulation state (embeddable headlessly)
  - `config.rs` - Persistent `AppConfig`
  - `gpu_state.rs` - GPU context and bind group caching
  - `handler/` - Event loop (modular: events, init, update, render, gpu_compute, buffer_sync, ui, brush, recording, presets_ops)
- **`src/simulation/`** - Particle data, spatial hash, boundaries, obstacles, matrix variation
  - `particle.rs` - `Particle`, `InteractionMatrix`, `RadiusMatrix`, SoA types
  - `spatial_hash.rs` - CPU-side spatial hash (tooling/tests; runtime hash runs on GPU)
  - `boundary.rs` - Boundary mode implementations
  - `obstacle.rs` - Obstacle shapes and `ObstacleData`
  - `matrix_variation.rs` - Time-varying interaction matrices
  - `game_of_life.rs` - Alternative simulation mode
- **`src/generators/`** - Rules (34), colors (37), positions (31) generators + `custom`/`expression` (user JSON DSL)
- **`src/renderer/gpu/`** - wgpu context, buffers, pipelines
- **`src/ui/`** - UI helpers
- **`src/utils/`** - Color and math utilities
- **`src/video_recorder.rs`** - ffmpeg-based video recording
- **`shaders/`** - WGSL compute and render shaders

See [README.md](README.md) for detailed architecture documentation.

## Key Patterns

### GPU Double-Buffering
Particle buffers use ping-pong pattern to avoid compute shader race conditions.

### Spatial Hashing
O(n*k) neighbor queries via cell-based partitioning. Cell size must be >= max interaction radius.

### Generator System
Each generator type (rules, colors, positions) has an enum with `all()` method returning available variants.

## Claude-Specific Notes

- Ask user for screenshots if needed - don't take them yourself
- Video recording requires ffmpeg (MP4/WebM/GIF formats)
- Presets stored in platform-specific app data directory
- Rust 1.93+ required (edition 2024)

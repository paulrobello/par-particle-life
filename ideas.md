# Par Particle Life — Enhancement Ideas

> **Disclaimer:** This file is an aspirational, non-authoritative scratchpad of ideas under consideration. It is not a roadmap and not a commitment. Items may be stale, partially implemented, abandoned, or already shipped. Treat the [CHANGELOG](CHANGELOG.md) and the [open GitHub issues](https://github.com/paulrobello/par-particle-life/issues) as the source of truth for what is actually planned or done.

Remove completed items from this list and update CHANGELOG.md

## Generators

### Gallery of Curated Presets
Build a curated collection of "best of" presets with descriptive names (e.g., "Predator-Prey Swarm", "Crystalline Lattice", "Spiral Galaxy"). The current preset system is bare — adding a gallery with thumbnails would make the app more approachable.

## UI & Interaction

### Interactive Matrix Heatmap View
Replace the current scroll-wheel NxN grid with a proper heatmap editor: click-drag to paint values, drag sliders for row/column, double-click to set exact value. The current tooltip + scroll interaction is hard to discover and slow for large matrices.

### Undo/Redo for Rule Changes
Track interaction matrix edits so users can undo/redo changes. The matrix editor is the primary creative tool — accidental scroll-wheel changes are currently irreversible.

### Particle Count Heatmap Overlay
Render a real-time density heatmap overlay showing where particles cluster. Useful for understanding emergent structure that isn't visible at the individual particle level.

### Onboarding Tutorial
Add a first-run tutorial overlay that highlights key UI sections and demonstrates the basic workflow: pick rules → adjust matrix → use brush tools.

---

## Recording & Export

### High-Resolution / Supersampled Screenshots
Allow capturing screenshots at 2x/4x resolution for print-quality output. Currently screenshots are at display resolution.

### Animated GIF Export of Matrix Changes
Record a timelapse of interaction matrix edits as a GIF — useful for sharing rule-tuning workflows.

### Simulation State Export/Import
Export the complete simulation state (particle positions, velocities, config) as a JSON file and reimport it. Currently presets save config but not particle state. Would enable exact reproduction and sharing of interesting configurations.

### SVG / Vector Export
Export the current frame as an SVG with circles for particles. Useful for print media and publication figures.

---

## Performance & GPU

Ranked by expected practical performance ROI in the current architecture.

| Rank | Idea | Complexity | Possible performance benefit | Notes |
| --- | --- | --- | --- | --- |
| 1 | Adaptive Workgroup Size | Medium | Low–Medium; GPU/scene dependent | Workgroup size is hardcoded to 256 in Rust dispatch and WGSL shaders. Benefit requires profiling and likely multiple shader/pipeline variants or overridable constants. |
| 2 | Async Compute Queue | Very High | Uncertain / Low until proven by profiling | Compute and render currently use one wgpu queue and render freshly computed buffers in the same frame. True overlap would require multi-frame buffering/scheduling redesign, and wgpu/backend queue support may limit gains. |

### Adaptive Workgroup Size
Dynamically adjust workgroup sizes based on GPU capabilities (detected via wgpu adapter limits). Currently hardcoded to 256.

### Async Compute Queue
Run spatial hash computation on a dedicated compute queue while rendering the previous frame on the graphics queue. Currently compute and render are serialized, leaving GPU headroom unused.

---

## Platform & Distribution

### WebAssembly Build
The WASM dependencies are already in `Cargo.toml` but not integrated. Publishing as a web app would dramatically increase reach. Would need web-specific input handling and file system alternatives.

---

## Game of Life Integration

### Wire Up the Existing Game of Life Module
A complete Conway's Game of Life implementation exists in `src/simulation/game_of_life.rs` with tests but is not connected to the UI. Add a simulation mode switcher (Particle Life ↔ Game of Life) to make this code accessible. Could also support hybrid modes where Game of Life cells coexist with particles.

---

## Analysis & Metrics

### Real-Time Statistics Dashboard
Display live metrics: average velocity, kinetic energy, clustering coefficient, type distribution, spatial entropy. Currently only FPS and GPU timing are shown.

### Recording Playback with Timeline
Add a recording playback mode where captured video frames can be scrubbed with a timeline. Paired with the state export idea, this enables "record → replay → branch" workflows.

### Automatic Interestingness Detection
Track entropy/energy metrics and auto-highlight or auto-record when the simulation passes through visually interesting transient states (high metric change rate).

# Par Particle Life — Enhancement Ideas

Remove completed items from this list and update CHANGELOG.md

## Simulation & Physics

### Multi-Step Integration Methods
Add Verlet / RK4 integration as an alternative to the current Euler method. Higher-order integration would allow larger time steps with the same stability, enabling more complex emergent behaviors at higher force factors.

### Time-Varying Interaction Matrices
Allow the interaction matrix to evolve over time — slow drift, periodic oscillation, or response to global metrics (e.g., entropy, clustering). Could produce dynamic ecosystems that shift between behaviors rather than settling into a single steady state.

### Velocity-Dependent Forces
Add an option where the interaction force between two particles depends on their relative velocity (e.g., alignment forces like in boid models). This would bridge particle life with flocking/orientation-based simulations.

### Obstacle Regions
Add static circular/rectangular obstacle zones that particles cannot enter. Particles would bounce off or wrap around obstacles, enabling "maze" and "container" experiments.

---

## Generators

### User-Defined Custom Rule Generators
Allow users to write simple rule generator functions via a text input (e.g., a mini-DSL or JSON template) rather than being limited to the 31 built-in rule types. Store custom generators alongside presets.

### Gallery of Curated Presets
Build a curated collection of "best of" presets with descriptive names (e.g., "Predator-Prey Swarm", "Crystalline Lattice", "Spiral Galaxy"). The current preset system is bare — adding a gallery with thumbnails would make the app more approachable.

### Interaction Matrix Templates
Provide named templates for common matrix patterns: fully symmetric, fully antisymmetric, block-diagonal (alliances), cyclic pursuit, random sparse. Currently users must hand-edit the NxN grid or rely on random generation.

### Gradient Position Generators
Add position generators where particle types are placed along a gradient (linear, radial, angular) rather than randomly or in discrete clusters. Useful for studying front propagation and mixing dynamics.

### Save/Load Custom Palettes
Allow users to create and save custom color palettes (pick N colors) rather than only using the 37 built-in generators. Store them in the presets directory.

---

## UI & Interaction

### Interactive Matrix Heatmap View
Replace the current scroll-wheel NxN grid with a proper heatmap editor: click-drag to paint values, drag sliders for row/column, double-click to set exact value. The current tooltip + scroll interaction is hard to discover and slow for large matrices.

### Simulation Speed Control
Add a time-scale slider (0.1x–5x) and a step-by-step mode (advance one frame at a time). Currently you can only pause or run at full speed.

### Undo/Redo for Rule Changes
Track interaction matrix edits so users can undo/redo changes. The matrix editor is the primary creative tool — accidental scroll-wheel changes are currently irreversible.

### Minimap / Overview
Add a small minimap in the corner showing the full simulation extent when zoomed in. With the 0.1x–10x zoom range, users can lose spatial context.

### Particle Count Heatmap Overlay
Render a real-time density heatmap overlay showing where particles cluster. Useful for understanding emergent structure that isn't visible at the individual particle level.

### Drag-and-Drop Preset Loading
Allow dragging a preset JSON file onto the window to load it. Faster than navigating to the presets folder.

### Fullscreen Mode
Add a fullscreen toggle (F11 currently records — could use F5 or a button). Useful for presentations and immersive viewing.

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

### Async Compute Queue
Run spatial hash computation on a dedicated compute queue while rendering the previous frame on the graphics queue. Currently compute and render are serialized, leaving GPU headroom unused.

### Adaptive Workgroup Size
Dynamically adjust workgroup sizes based on GPU capabilities (detected via wgpu adapter limits). Currently hardcoded to 256.

### Particle Count Hot-Swap
Allow changing particle count without full simulation reset — spawn new particles at random positions or remove from lowest-density regions. Currently changing count requires a full reset.

### GPU-Driven Particle Spawning
Move the Draw brush tool's particle spawning to a compute shader instead of CPU-side buffer manipulation. Currently, brush drawing locks CPU-GPU sync.

---

## Platform & Distribution

### WebAssembly Build
The WASM dependencies are already in `Cargo.toml` but not integrated. Publishing as a web app would dramatically increase reach. Would need web-specific input handling and file system alternatives.

### Android / iOS Build
wgpu supports mobile targets. A touch-optimized UI with pinch-zoom and tap-brush would make the simulation accessible on tablets.

### Automatic CI on Push
The CI workflow (`ci.yml`) is currently manual-only (`workflow_dispatch`). Add `on: push` to `main` and `on: pull_request` so tests and lints run automatically, catching regressions early.

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

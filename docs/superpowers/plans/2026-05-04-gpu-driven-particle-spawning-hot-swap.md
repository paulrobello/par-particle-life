# GPU-Driven Particle Spawning and Particle Count Hot-Swap Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add smooth particle count changes and GPU-driven Draw brush spawning without resetting the running simulation or blocking on GPU readback during drawing.

**Architecture:** Keep particle buffers allocated with headroom and track active particle count separately from buffer capacity. Hot-swap count changes preserve existing GPU particle state when capacity is available; GPU Draw brush spawning writes newly spawned particles directly into both ping-pong particle buffers, then increments the active count on the CPU without reading particles back. Rare capacity overflows fall back to readback + buffer recreation so correctness is preserved before optimizing further.

**Tech Stack:** Rust 2024, wgpu 29 compute pipelines, WGSL shaders, egui UI, existing `SimulationBuffers` ping-pong SoA layout.

**Branch:** `feature/gpu-spawn-hot-swap-plan`

---

## File Structure

- Modify `src/simulation/particle.rs`
  - Add `Particle::random_in_world()` helper for CPU-side hot-swap additions.
- Modify `src/app/state.rs`
  - Add `App::set_particle_count_preserving_existing()` for CPU vector length bookkeeping and fallback buffer rebuilds.
- Modify `src/renderer/gpu/buffers.rs`
  - Add active-vs-capacity count tracking to `SimulationBuffers`.
  - Add `SpawnParamsUniform` for GPU Draw brush spawning.
  - Add byte-offset range write helpers for appended CPU particles.
- Modify `src/renderer/gpu/mod.rs`
  - Re-export `SpawnParamsUniform` for handler code and integration tests.
- Modify `src/renderer/gpu/pipelines/brush.rs`
  - Add spawn compute pipeline, bind group layout, spawn params buffer, and helpers.
- Create `shaders/particle_spawn.wgsl`
  - Compute shader that fills new particle slots in current and next ping-pong buffers.
- Modify `src/app/handler/brush.rs`
  - Replace Draw brush CPU readback path with GPU spawn dispatch.
  - Keep Erase brush on existing CPU readback path for this plan.
- Modify `src/app/handler/buffer_sync.rs`
  - Add hot-swap count sync helper and capacity fallback path.
- Modify `src/app/handler/ui.rs`
  - Use hot-swap helper for particle count changes instead of full regeneration.
- Modify `src/app/handler/gpu_compute.rs`, `src/app/handler/render.rs`, and `src/app/gpu_state.rs` only where active count/capacity behavior requires naming or cache invalidation adjustments.
- Modify `tests/enhancement_features.rs`
  - Add CPU-level tests for count preservation, truncation, and capacity helper behavior.

---

## Task 1: Add CPU-Level Particle Count Helpers

**Files:**
- Modify: `src/simulation/particle.rs`
- Modify: `src/app/state.rs`
- Test: `tests/enhancement_features.rs`

- [ ] **Step 1: Write failing tests for preserving existing particles when growing/shrinking**

Append these tests to `tests/enhancement_features.rs`:

```rust
use par_particle_life::simulation::Particle;

#[test]
fn set_particle_count_preserves_existing_particles_when_growing() {
    let mut app = par_particle_life::app::App::new(true);
    app.sim_config.num_types = 4;
    app.sim_config.world_size = glam::Vec2::new(800.0, 600.0);
    app.particles = vec![
        Particle::with_velocity(10.0, 20.0, 1.0, 2.0, 0),
        Particle::with_velocity(30.0, 40.0, 3.0, 4.0, 1),
    ];
    app.sim_config.num_particles = 2;

    let original = app.particles.clone();
    app.set_particle_count_preserving_existing(5);

    assert_eq!(app.particles.len(), 5);
    assert_eq!(app.sim_config.num_particles, 5);
    assert_eq!(app.particles[0].x, original[0].x);
    assert_eq!(app.particles[0].y, original[0].y);
    assert_eq!(app.particles[0].vx, original[0].vx);
    assert_eq!(app.particles[1].particle_type, original[1].particle_type);
    for particle in &app.particles[2..] {
        assert!((0.0..=800.0).contains(&particle.x));
        assert!((0.0..=600.0).contains(&particle.y));
        assert!(particle.particle_type < 4);
        assert_eq!(particle.vx, 0.0);
        assert_eq!(particle.vy, 0.0);
    }
}

#[test]
fn set_particle_count_truncates_without_regenerating_prefix() {
    let mut app = par_particle_life::app::App::new(true);
    app.particles = vec![
        Particle::with_velocity(10.0, 20.0, 1.0, 2.0, 0),
        Particle::with_velocity(30.0, 40.0, 3.0, 4.0, 1),
        Particle::with_velocity(50.0, 60.0, 5.0, 6.0, 2),
    ];
    app.sim_config.num_particles = 3;

    app.set_particle_count_preserving_existing(2);

    assert_eq!(app.particles.len(), 2);
    assert_eq!(app.sim_config.num_particles, 2);
    assert_eq!(app.particles[0].x, 10.0);
    assert_eq!(app.particles[1].y, 40.0);
    assert_eq!(app.particles[1].vy, 4.0);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```bash
cargo test set_particle_count
```

Expected: compilation fails because `App::set_particle_count_preserving_existing` does not exist.

- [ ] **Step 3: Add `Particle::random_in_world`**

In `src/simulation/particle.rs`, add this method inside `impl Particle`:

```rust
    /// Create a zero-velocity particle at a random position inside the world.
    pub fn random_in_world<R: rand::Rng + ?Sized>(
        rng: &mut R,
        world_width: f32,
        world_height: f32,
        num_types: u32,
    ) -> Self {
        let x = rng.random::<f32>() * world_width;
        let y = rng.random::<f32>() * world_height;
        let particle_type = rng.random_range(0..num_types.max(1));
        Self::new(x, y, particle_type)
    }
```

Also add the import at the top of `src/simulation/particle.rs`:

```rust
use rand::Rng;
```

- [ ] **Step 4: Add `App::set_particle_count_preserving_existing`**

In `src/app/state.rs`, add this method inside `impl App` near `regenerate_particles`:

```rust
    /// Resize the particle list while preserving existing particle state.
    /// New particles are spawned at random zero-velocity positions.
    pub fn set_particle_count_preserving_existing(&mut self, target_count: u32) {
        let target_len = target_count as usize;
        let current_len = self.particles.len();

        if target_len < current_len {
            self.particles.truncate(target_len);
        } else if target_len > current_len {
            let mut rng = rand::rng();
            let additional = target_len - current_len;
            self.particles.reserve(additional);
            for _ in 0..additional {
                self.particles.push(Particle::random_in_world(
                    &mut rng,
                    self.sim_config.world_size.x,
                    self.sim_config.world_size.y,
                    self.sim_config.num_types,
                ));
            }
        }

        self.sim_config.num_particles = target_count;
        self.physics.resize(target_len);
    }
```

- [ ] **Step 5: Run tests to verify they pass**

Run:

```bash
cargo test set_particle_count
```

Expected: both tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/simulation/particle.rs src/app/state.rs tests/enhancement_features.rs
git commit -m "feat: add preserving particle count resize helper"
```

---

## Task 2: Add Active Count and Capacity Tracking to Simulation Buffers

**Files:**
- Modify: `src/renderer/gpu/buffers.rs`
- Modify: `src/app/handler/gpu_compute.rs`
- Modify: `src/app/handler/render.rs`
- Test: `tests/enhancement_features.rs`

- [ ] **Step 1: Write failing tests for capacity helpers**

Append these tests to `tests/enhancement_features.rs`:

```rust
#[test]
fn particle_buffer_capacity_has_ui_hot_swap_headroom() {
    assert_eq!(
        par_particle_life::renderer::gpu::SimulationBuffers::capacity_for_particle_count(1_000),
        128_000
    );
    assert_eq!(
        par_particle_life::renderer::gpu::SimulationBuffers::capacity_for_particle_count(64_000),
        128_000
    );
    assert_eq!(
        par_particle_life::renderer::gpu::SimulationBuffers::capacity_for_particle_count(200_000),
        200_000
    );
}

#[test]
fn particle_range_byte_offsets_match_soa_layout() {
    assert_eq!(
        par_particle_life::renderer::gpu::SimulationBuffers::pos_type_byte_offset(3),
        (3 * std::mem::size_of::<par_particle_life::simulation::ParticlePosType>()) as u64
    );
    assert_eq!(
        par_particle_life::renderer::gpu::SimulationBuffers::velocity_byte_offset_f32(3),
        (3 * std::mem::size_of::<par_particle_life::simulation::ParticleVel>()) as u64
    );
    assert_eq!(
        par_particle_life::renderer::gpu::SimulationBuffers::velocity_byte_offset_f16(3),
        (3 * std::mem::size_of::<par_particle_life::simulation::ParticleVelHalf>()) as u64
    );
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```bash
cargo test particle_buffer_capacity_has_ui_hot_swap_headroom
cargo test particle_range_byte_offsets_match_soa_layout
```

Expected: compilation fails because the helper methods do not exist or `renderer::gpu` does not export `SimulationBuffers` publicly enough for the tests.

- [ ] **Step 3: Add capacity fields and helper methods**

In `src/renderer/gpu/buffers.rs`, update `SimulationBuffers`:

```rust
    /// Active particle count used by compute and render dispatches.
    pub num_particles: u32,
    /// Allocated particle capacity in each particle buffer.
    pub capacity_particles: u32,
```

Add these methods in `impl SimulationBuffers`:

```rust
    /// Minimum capacity that lets the current UI particle-count choices hot-swap without reallocating.
    pub const MIN_HOT_SWAP_CAPACITY: u32 = 128_000;

    /// Choose a buffer capacity for the requested active particle count.
    pub fn capacity_for_particle_count(active_particles: u32) -> u32 {
        active_particles.max(Self::MIN_HOT_SWAP_CAPACITY)
    }

    /// Byte offset into position/type buffers for a particle index.
    pub fn pos_type_byte_offset(index: u32) -> u64 {
        index as u64 * std::mem::size_of::<ParticlePosType>() as u64
    }

    /// Byte offset into f32 velocity buffers for a particle index.
    pub fn velocity_byte_offset_f32(index: u32) -> u64 {
        index as u64 * std::mem::size_of::<ParticleVel>() as u64
    }

    /// Byte offset into f16 velocity buffers for a particle index.
    pub fn velocity_byte_offset_f16(index: u32) -> u64 {
        index as u64 * std::mem::size_of::<ParticleVelHalf>() as u64
    }

    /// Current velocity byte offset for the configured velocity representation.
    pub fn velocity_byte_offset(&self, index: u32) -> u64 {
        if self.use_f16 {
            Self::velocity_byte_offset_f16(index)
        } else {
            Self::velocity_byte_offset_f32(index)
        }
    }

    /// Set the active particle count without reallocating buffers.
    pub fn set_num_particles(&mut self, num_particles: u32) {
        assert!(
            num_particles <= self.capacity_particles,
            "active particle count exceeds allocated GPU capacity"
        );
        self.num_particles = num_particles;
    }

    /// Return true when the buffers can hold the requested active count.
    pub fn has_capacity_for(&self, target_count: u32) -> bool {
        target_count <= self.capacity_particles
    }
```

- [ ] **Step 4: Allocate particle buffers using capacity**

In `SimulationBuffers::new`, compute capacity and initialize padded CPU vectors:

```rust
        let num_particles = particles.len() as u32;
        let capacity_particles = Self::capacity_for_particle_count(num_particles);
        let num_types = config.num_types;

        let mut padded_particles = particles.to_vec();
        padded_particles.resize(capacity_particles as usize, Particle::default());
```

Then replace each conversion over `particles.iter()` used for particle buffers with `padded_particles.iter()`.

In the returned struct, set both values:

```rust
            num_particles,
            capacity_particles,
```

- [ ] **Step 5: Use active count for buffer copy sizes**

Keep `velocity_buffer_size()` active-count based:

```rust
    /// Byte size for active particles in one velocity buffer.
    pub fn velocity_buffer_size(&self) -> u64 {
        let num = self.num_particles as u64;
        if self.use_f16 {
            num * std::mem::size_of::<ParticleVelHalf>() as u64
        } else {
            num * std::mem::size_of::<ParticleVel>() as u64
        }
    }
```

Add capacity-size helpers for full-buffer creation/debugging when needed:

```rust
    /// Byte size for allocated particles in one position/type buffer.
    pub fn pos_type_capacity_buffer_size(&self) -> u64 {
        self.capacity_particles as u64 * std::mem::size_of::<ParticlePosType>() as u64
    }

    /// Byte size for allocated particles in one velocity buffer.
    pub fn velocity_capacity_buffer_size(&self) -> u64 {
        let num = self.capacity_particles as u64;
        if self.use_f16 {
            num * std::mem::size_of::<ParticleVelHalf>() as u64
        } else {
            num * std::mem::size_of::<ParticleVel>() as u64
        }
    }
```

- [ ] **Step 6: Re-run focused tests**

Run:

```bash
cargo test particle_buffer_capacity_has_ui_hot_swap_headroom
cargo test particle_range_byte_offsets_match_soa_layout
```

Expected: both tests pass.

- [ ] **Step 7: Run compile check**

Run:

```bash
cargo check
```

Expected: build succeeds. If render/compute code still uses `num_particles`, it should continue to mean active count.

- [ ] **Step 8: Commit**

```bash
git add src/renderer/gpu/buffers.rs src/app/handler/gpu_compute.rs src/app/handler/render.rs tests/enhancement_features.rs
git commit -m "feat: track active particle count separately from capacity"
```

---

## Task 3: Add Non-Reset Particle Count Hot-Swap Path

**Files:**
- Modify: `src/renderer/gpu/buffers.rs`
- Modify: `src/app/handler/buffer_sync.rs`
- Modify: `src/app/handler/ui.rs`
- Modify: `src/app/handler/brush.rs`
- Test: existing unit tests plus manual app behavior

- [ ] **Step 1: Add particle range write helpers**

In `src/renderer/gpu/buffers.rs`, add this method to `impl SimulationBuffers`:

```rust
    /// Write a contiguous active particle range into both ping-pong buffers.
    pub fn write_particle_range(&self, queue: &Queue, start_index: u32, particles: &[Particle]) {
        if particles.is_empty() {
            return;
        }

        let end_index = start_index + particles.len() as u32;
        assert!(
            end_index <= self.capacity_particles,
            "particle range exceeds allocated GPU capacity"
        );

        let pos_type_data: Vec<ParticlePosType> = particles.iter().map(ParticlePosType::from).collect();
        let pos_type_bytes = bytemuck::cast_slice(&pos_type_data);
        let pos_offset = Self::pos_type_byte_offset(start_index);
        queue.write_buffer(&self.pos_type[0], pos_offset, pos_type_bytes);
        queue.write_buffer(&self.pos_type[1], pos_offset, pos_type_bytes);

        if self.use_f16 {
            let vel_data: Vec<ParticleVelHalf> = particles.iter().map(ParticleVelHalf::from).collect();
            let vel_bytes = bytemuck::cast_slice(&vel_data);
            let vel_offset = Self::velocity_byte_offset_f16(start_index);
            queue.write_buffer(&self.velocities[0], vel_offset, vel_bytes);
            queue.write_buffer(&self.velocities[1], vel_offset, vel_bytes);
            queue.write_buffer(&self.velocity_scratch, vel_offset, vel_bytes);
        } else {
            let vel_data: Vec<ParticleVel> = particles.iter().map(ParticleVel::from).collect();
            let vel_bytes = bytemuck::cast_slice(&vel_data);
            let vel_offset = Self::velocity_byte_offset_f32(start_index);
            queue.write_buffer(&self.velocities[0], vel_offset, vel_bytes);
            queue.write_buffer(&self.velocities[1], vel_offset, vel_bytes);
            queue.write_buffer(&self.velocity_scratch, vel_offset, vel_bytes);
        }
    }
```

- [ ] **Step 2: Add hot-swap handler helper**

In `src/app/handler/buffer_sync.rs`, add this method to `impl AppHandler`:

```rust
    /// Change active particle count without resetting existing GPU particle state when possible.
    pub(crate) fn hot_swap_particle_count(&mut self, target_count: u32) {
        if target_count == self.app.sim_config.num_particles {
            return;
        }

        let previous_count = self.app.sim_config.num_particles;
        self.app.config.sim_num_particles = target_count;
        self.app.rebalance_radii_for_density();

        let gpu_has_capacity = self
            .gpu
            .as_ref()
            .map(|gpu| gpu.buffers.has_capacity_for(target_count))
            .unwrap_or(false);

        if gpu_has_capacity {
            self.app.set_particle_count_preserving_existing(target_count);

            if target_count > previous_count {
                let start = previous_count as usize;
                let appended = &self.app.particles[start..];
                if let Some(gpu) = &mut self.gpu {
                    gpu.buffers.write_particle_range(&gpu.context.queue, previous_count, appended);
                }
            }

            if let Some(gpu) = &mut self.gpu {
                gpu.buffers.set_num_particles(target_count);
                gpu.buffers.update_params(&gpu.context.queue, &self.app.sim_config, 1.0 / 60.0);
                gpu.spatial_buffers.update_params(
                    &gpu.context.queue,
                    &self.app.sim_config,
                    self.app.radius_matrix.max_interaction_radius(),
                );
            }
        } else {
            // Rare path: preserve current GPU state by reading it back, then allocate a larger capacity.
            self.sync_particles_from_gpu();
            self.app.set_particle_count_preserving_existing(target_count);
            self.sync_buffers();
        }
    }
```

- [ ] **Step 3: Update particle count UI to use hot-swap helper**

In `src/app/handler/ui.rs`, replace the current particle-count change block:

```rust
                            if num_particles != self.app.sim_config.num_particles {
                                self.app.sim_config.num_particles = num_particles;
                                self.app.config.sim_num_particles = num_particles;
                                self.app.rebalance_radii_for_density();
                                self.app.regenerate_particles();
                                self.sync_buffers();
                            }
```

with:

```rust
                            if num_particles != self.app.sim_config.num_particles {
                                self.hot_swap_particle_count(num_particles);
                            }
```

- [ ] **Step 4: Keep Draw brush on old path until GPU spawn pipeline lands**

Do not change `draw_particles()` in this task. This keeps the hot-swap feature independently testable before replacing Draw brush internals.

- [ ] **Step 5: Run verification**

Run:

```bash
cargo test set_particle_count
cargo test particle_buffer_capacity_has_ui_hot_swap_headroom
cargo test particle_range_byte_offsets_match_soa_layout
cargo check
```

Expected: tests and compile check pass.

- [ ] **Step 6: Manual smoke test hot-swap**

Run:

```bash
cargo run --release
```

Expected manual behavior:
- Start simulation.
- Change Particles from `64000` to `128000`.
- Existing visible structure continues instead of resetting to a fresh layout.
- Change Particles down to `32000`.
- Simulation continues with fewer particles and no full reset.

- [ ] **Step 7: Commit**

```bash
git add src/renderer/gpu/buffers.rs src/app/handler/buffer_sync.rs src/app/handler/ui.rs
git commit -m "feat: hot-swap active particle count without reset"
```

---

## Task 4: Add GPU Particle Spawn Shader and Pipeline

**Files:**
- Create: `shaders/particle_spawn.wgsl`
- Modify: `src/renderer/gpu/buffers.rs`
- Modify: `src/renderer/gpu/mod.rs`
- Modify: `src/renderer/gpu/pipelines/brush.rs`
- Test: `cargo check` plus shader pipeline creation through app startup

- [ ] **Step 1: Add spawn uniform type**

In `src/renderer/gpu/buffers.rs`, add this struct near `BrushParamsUniform`:

```rust
/// Parameters for GPU-driven particle spawning.
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct SpawnParamsUniform {
    /// First particle index to write.
    pub start_index: u32,
    /// Number of particles to spawn this dispatch.
    pub spawn_count: u32,
    /// Allocated particle capacity.
    pub capacity_particles: u32,
    /// Number of particle types.
    pub num_types: u32,
    /// Brush position X in world coordinates.
    pub pos_x: f32,
    /// Brush position Y in world coordinates.
    pub pos_y: f32,
    /// Brush radius in world coordinates.
    pub radius: f32,
    /// World width.
    pub world_width: f32,
    /// World height.
    pub world_height: f32,
    /// Draw type, or -1 for random.
    pub draw_type: i32,
    /// Frame counter for deterministic GPU random seeds.
    pub frame_counter: u32,
    /// Padding for 16-byte alignment.
    pub _padding: u32,
}
```

- [ ] **Step 2: Re-export spawn uniform**

In `src/renderer/gpu/mod.rs`, add `SpawnParamsUniform` to the `pub use buffers::{ ... }` list:

```rust
pub use buffers::{
    BrushParamsUniform, BrushRenderUniform, GlowParamsUniform, InfiniteParamsUniform,
    MirrorParamsUniform, RenderBuffers, SimParamsUniform, SimulationBuffers, SpatialHashBuffers,
    SpatialParamsUniform, SpawnParamsUniform,
};
```

- [ ] **Step 3: Create the WGSL shader**

Create `shaders/particle_spawn.wgsl`:

```wgsl
struct PosType {
    x: POS_FLOAT,
    y: POS_FLOAT,
    particle_type: u32,
    _padding: u32,
}

struct SpawnParams {
    start_index: u32,
    spawn_count: u32,
    capacity_particles: u32,
    num_types: u32,
    pos_x: f32,
    pos_y: f32,
    radius: f32,
    world_width: f32,
    world_height: f32,
    draw_type: i32,
    frame_counter: u32,
    _padding: u32,
}

@group(0) @binding(0) var<storage, read_write> pos_current: array<PosType>;
@group(0) @binding(1) var<storage, read_write> vel_current: array<vec2<VEL_FLOAT>>;
@group(0) @binding(2) var<storage, read_write> pos_next: array<PosType>;
@group(0) @binding(3) var<storage, read_write> vel_next: array<vec2<VEL_FLOAT>>;
@group(0) @binding(4) var<uniform> params: SpawnParams;

fn pcg_hash(input: u32) -> u32 {
    var state = input * 747796405u + 2891336453u;
    let word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    return (word >> 22u) ^ word;
}

fn random01(seed: u32) -> f32 {
    return f32(pcg_hash(seed)) / 4294967295.0;
}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let local_index = id.x;
    if (local_index >= params.spawn_count) {
        return;
    }

    let target_index = params.start_index + local_index;
    if (target_index >= params.capacity_particles) {
        return;
    }

    let seed_base = target_index + params.frame_counter * 1664525u + 1013904223u;
    let angle = random01(seed_base) * 6.28318530718;
    let distance = sqrt(random01(seed_base + 1u)) * params.radius;

    var x = params.pos_x + cos(angle) * distance;
    var y = params.pos_y + sin(angle) * distance;
    x = x - params.world_width * floor(x / params.world_width);
    y = y - params.world_height * floor(y / params.world_height);

    var particle_type = 0u;
    if (params.draw_type >= 0) {
        particle_type = min(u32(params.draw_type), params.num_types - 1u);
    } else {
        particle_type = pcg_hash(seed_base + 2u) % max(params.num_types, 1u);
    }

    let spawned_pos = PosType(POS_FLOAT(x), POS_FLOAT(y), particle_type, 0u);
    let spawned_vel = vec2<VEL_FLOAT>(VEL_FLOAT(0.0), VEL_FLOAT(0.0));

    pos_current[target_index] = spawned_pos;
    vel_current[target_index] = spawned_vel;
    pos_next[target_index] = spawned_pos;
    vel_next[target_index] = spawned_vel;
}
```

- [ ] **Step 4: Add spawn pipeline fields**

In `src/renderer/gpu/pipelines/brush.rs`, update imports:

```rust
use crate::renderer::gpu::{BrushParamsUniform, BrushRenderUniform, SimulationBuffers, SpawnParamsUniform};
```

Add fields to `BrushPipelines`:

```rust
    /// Compute pipeline for GPU-driven particle spawning.
    pub spawn_pipeline: ComputePipeline,
    /// Bind group layout for GPU-driven particle spawning.
    pub spawn_bind_group_layout: BindGroupLayout,
    /// Spawn parameters uniform buffer.
    pub spawn_buffer: Buffer,
```

- [ ] **Step 5: Construct spawn pipeline and buffer**

In `BrushPipelines::new`, after the force pipeline creation, load the shader and create layout/pipeline:

```rust
        let spawn_shader = load_shader(
            device,
            "Particle Spawn Shader",
            include_str!("../../../../shaders/particle_spawn.wgsl"),
        );

        let spawn_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Particle Spawn Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry { binding: 0, visibility: ShaderStages::COMPUTE, ty: BindingType::Buffer { ty: BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None }, count: None },
                BindGroupLayoutEntry { binding: 1, visibility: ShaderStages::COMPUTE, ty: BindingType::Buffer { ty: BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None }, count: None },
                BindGroupLayoutEntry { binding: 2, visibility: ShaderStages::COMPUTE, ty: BindingType::Buffer { ty: BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None }, count: None },
                BindGroupLayoutEntry { binding: 3, visibility: ShaderStages::COMPUTE, ty: BindingType::Buffer { ty: BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None }, count: None },
                BindGroupLayoutEntry { binding: 4, visibility: ShaderStages::COMPUTE, ty: BindingType::Buffer { ty: BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None },
            ],
        });

        let spawn_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Particle Spawn Pipeline Layout"),
            bind_group_layouts: &[Some(&spawn_bind_group_layout)],
            immediate_size: 0,
        });

        let spawn_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Particle Spawn Pipeline"),
            layout: Some(&spawn_pipeline_layout),
            module: &spawn_shader,
            entry_point: Some("main"),
            compilation_options: PipelineCompilationOptions::default(),
            cache: None,
        });

        let default_spawn = SpawnParamsUniform {
            start_index: 0,
            spawn_count: 0,
            capacity_particles: 0,
            num_types: 1,
            pos_x: 0.0,
            pos_y: 0.0,
            radius: 100.0,
            world_width: 1000.0,
            world_height: 1000.0,
            draw_type: -1,
            frame_counter: 0,
            _padding: 0,
        };
        let spawn_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Particle Spawn Params Buffer"),
            contents: bytemuck::bytes_of(&default_spawn),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
```

Include these fields in the returned `Self`.

- [ ] **Step 6: Add spawn bind group and uniform update helpers**

In `impl BrushPipelines`, add:

```rust
    /// Create bind group for GPU-driven particle spawning into both ping-pong buffers.
    pub fn create_spawn_bind_group(&self, device: &Device, buffers: &SimulationBuffers) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: Some("Particle Spawn Bind Group"),
            layout: &self.spawn_bind_group_layout,
            entries: &[
                BindGroupEntry { binding: 0, resource: buffers.current_pos_type().as_entire_binding() },
                BindGroupEntry { binding: 1, resource: buffers.current_velocities().as_entire_binding() },
                BindGroupEntry { binding: 2, resource: buffers.next_pos_type().as_entire_binding() },
                BindGroupEntry { binding: 3, resource: buffers.next_velocities().as_entire_binding() },
                BindGroupEntry { binding: 4, resource: self.spawn_buffer.as_entire_binding() },
            ],
        })
    }

    /// Update GPU spawn parameters.
    pub fn update_spawn(&self, queue: &Queue, params: SpawnParamsUniform) {
        queue.write_buffer(&self.spawn_buffer, 0, bytemuck::bytes_of(&params));
    }
```

- [ ] **Step 7: Run compile check**

Run:

```bash
cargo check
```

Expected: compile succeeds. If WGSL type aliases `POS_FLOAT` and `VEL_FLOAT` are injected by `load_shader`, pipeline creation should validate on app startup.

- [ ] **Step 8: Run app startup shader validation**

Run:

```bash
cargo run --release
```

Expected: app opens without a wgpu shader validation error mentioning `particle_spawn.wgsl`.

- [ ] **Step 9: Commit**

```bash
git add shaders/particle_spawn.wgsl src/renderer/gpu/buffers.rs src/renderer/gpu/mod.rs src/renderer/gpu/pipelines/brush.rs
git commit -m "feat: add gpu particle spawn pipeline"
```

---

## Task 5: Replace Draw Brush CPU Readback with GPU Spawn Dispatch

**Files:**
- Modify: `src/app/handler/brush.rs`
- Modify: `src/app/handler/update.rs`
- Modify: `src/app/handler/buffer_sync.rs`
- Test: manual interaction plus `make checkall`

- [ ] **Step 1: Add GPU spawn dispatch helper**

In `src/app/handler/brush.rs`, replace `draw_particles()` with this GPU-oriented version:

```rust
    /// Draw particles at the brush position using a GPU compute shader.
    pub(crate) fn draw_particles(&mut self) {
        let Some(gpu) = &mut self.gpu else { return };

        let current_count = gpu.buffers.num_particles;
        let capacity = gpu.buffers.capacity_particles;
        if current_count >= capacity {
            self.preset_status = format!("Particle capacity reached ({capacity})");
            return;
        }

        let requested = self.brush.draw_intensity as u32;
        let spawn_count = requested.min(capacity - current_count);
        if spawn_count == 0 {
            return;
        }

        let params = crate::renderer::gpu::SpawnParamsUniform {
            start_index: current_count,
            spawn_count,
            capacity_particles: capacity,
            num_types: self.app.sim_config.num_types.max(1),
            pos_x: self.brush.position.x,
            pos_y: self.brush.position.y,
            radius: self.brush.radius,
            world_width: self.app.sim_config.world_size.x,
            world_height: self.app.sim_config.world_size.y,
            draw_type: self.brush.draw_type,
            frame_counter: self.app.sim_config.frame_counter,
            _padding: 0,
        };

        gpu.brush_pipelines.update_spawn(&gpu.context.queue, params);
        let spawn_bind_group = gpu
            .brush_pipelines
            .create_spawn_bind_group(&gpu.context.device, &gpu.buffers);

        let mut encoder = gpu.context.create_encoder("Particle Spawn Encoder");
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Particle Spawn Pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&gpu.brush_pipelines.spawn_pipeline);
            pass.set_bind_group(0, &spawn_bind_group, &[]);
            pass.dispatch_workgroups(spawn_count.div_ceil(256), 1, 1);
        }
        gpu.context.submit(encoder.finish());

        let new_count = current_count + spawn_count;
        gpu.buffers.set_num_particles(new_count);
        self.app.sim_config.num_particles = new_count;
        self.app.config.sim_num_particles = new_count;
        self.app.particles.resize(new_count as usize, crate::simulation::Particle::default());
        self.app.physics.resize(new_count as usize);
    }
```

- [ ] **Step 2: Remove now-unused import**

At the top of `src/app/handler/brush.rs`, remove:

```rust
use rand::RngExt;
```

Keep imports needed by Erase:

```rust
use super::AppHandler;
use crate::app::BrushTool;
use crate::simulation::{BoundaryMode, ObstacleShape};
```

- [ ] **Step 3: Ensure params are updated after spawning**

`src/app/handler/update.rs` already calls `process_brush_tools()` before `buffers.update_params(...)`. Keep that order. Verify the order still reads:

```rust
        self.process_brush_tools();

        if self.needs_sync {
            self.sync_buffers();
            self.needs_sync = false;
        }

        if let Some(gpu_state_ref) = self.gpu.as_ref() {
            gpu_state_ref.buffers.update_params(
                &gpu_state_ref.context.queue,
                &self.app.sim_config,
                dt_capped,
            );
        }
```

- [ ] **Step 4: Run compile check**

Run:

```bash
cargo check
```

Expected: compile succeeds with no unused imports.

- [ ] **Step 5: Manual Draw brush smoke test**

Run:

```bash
cargo run --release
```

Expected manual behavior:
- Select Brush Tools → Draw.
- Hold mouse in the simulation.
- Particle count increases while the app remains responsive.
- No log message from `sync_particles_from_gpu()` or blocking readback appears during Draw.
- Attract/Repel still work because they remain integrated in `particle_advance.wgsl`.
- Erase still works, even if it can still block; erase optimization is outside this plan.

- [ ] **Step 6: Commit**

```bash
git add src/app/handler/brush.rs src/app/handler/update.rs src/app/handler/buffer_sync.rs
git commit -m "feat: spawn draw brush particles on gpu"
```

---

## Task 6: Final Verification and Documentation Cleanup

**Files:**
- Modify: `ideas.md` if implementation is complete enough to mark the related ideas as done.
- Modify: `CHANGELOG.md` if the branch completes implementation, not just planning.
- Verify: all changed Rust/WGSL code

- [ ] **Step 1: Run formatter**

Run:

```bash
cargo fmt
```

Expected: command exits 0.

- [ ] **Step 2: Run lint**

Run:

```bash
cargo clippy -- -D warnings
```

Expected: command exits 0 with no warnings.

- [ ] **Step 3: Run tests**

Run:

```bash
cargo test
```

Expected: all tests pass.

- [ ] **Step 4: Run full project verification**

Run:

```bash
make checkall
```

Expected: format, lint, and tests all pass; output ends with `All checks passed!`.

- [ ] **Step 5: Update ideas and changelog only after implementation passes**

If Tasks 1-5 are implemented and `make checkall` passes:
- Remove `GPU-Driven Particle Spawning` from `ideas.md`.
- Update the Performance & GPU ranking table in `ideas.md` so `Particle Count Hot-Swap` is removed or marked complete.
- Add an entry to `CHANGELOG.md` describing:
  - Particle count changes now preserve existing simulation state when possible.
  - Draw brush spawning now uses a GPU compute path.

- [ ] **Step 6: Update graphify index after code changes**

Run:

```bash
graphify update .
```

Expected: graph update completes or reports no source changes requiring updates.

- [ ] **Step 7: Commit final cleanup**

```bash
git add ideas.md CHANGELOG.md graphify-out src shaders tests
git commit -m "docs: record gpu spawning and hot-swap completion"
```

Skip this commit if no docs or graph files changed in Step 5 or Step 6.

---

## Self-Review Checklist

- Spec coverage: The plan covers particle count hot-swap, active/capacity buffer tracking, GPU-driven Draw spawning, UI integration, fallback behavior, verification, and docs cleanup.
- Placeholder scan: No task relies on unspecified behavior; each code-changing step includes concrete code or exact replacement text.
- Type consistency: `SimulationBuffers::num_particles` remains active count; `SimulationBuffers::capacity_particles` is allocation count; `SpawnParamsUniform` fields match `particle_spawn.wgsl` `SpawnParams` layout.
- Scope control: Erase brush still uses the existing CPU readback path. Async compute queue and adaptive workgroup sizing are not included.

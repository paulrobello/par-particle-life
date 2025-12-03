// Sort particles by their spatial bin.
// Uses bin_offsets from prefix sum and atomic counters to place particles.
// Particles are reordered so particles in the same bin are contiguous.

struct PosType {
    x: POS_FLOAT,
    y: POS_FLOAT,
    particle_type: u32,
    _padding: u32,
}

struct SpatialParams {
    num_particles: u32,
    cell_size: f32,
    grid_width: u32,
    grid_height: u32,
}

@group(0) @binding(0) var<storage, read> particles_in: array<PosType>;
@group(0) @binding(1) var<storage, read_write> particles_out: array<PosType>;
@group(0) @binding(2) var<storage, read> vel_in: array<vec2<VEL_FLOAT>>;
@group(0) @binding(3) var<storage, read_write> vel_out: array<vec2<VEL_FLOAT>>;
@group(0) @binding(4) var<storage, read> bin_offsets: array<u32>;
@group(0) @binding(5) var<storage, read_write> bin_counts: array<atomic<u32>>;
@group(0) @binding(6) var<uniform> params: SpatialParams;

fn get_bin_index(pos: vec2<f32>) -> u32 {
    let bin_x = clamp(
        u32(floor(pos.x / params.cell_size)),
        0u,
        params.grid_width - 1u
    );
    let bin_y = clamp(
        u32(floor(pos.y / params.cell_size)),
        0u,
        params.grid_height - 1u
    );
    return bin_y * params.grid_width + bin_x;
}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= params.num_particles) {
        return;
    }

    let particle = particles_in[id.x];
    let vel = vel_in[id.x];
    
    // Must cast input to f32 for bin index calculation
    let bin_index = get_bin_index(vec2<f32>(f32(particle.x), f32(particle.y)));

    // bin_offsets[bin_index] is already the correct start offset
    let base_offset = bin_offsets[bin_index];

    // Atomically increment the counter to get our unique position within the bin
    let local_offset = atomicAdd(&bin_counts[bin_index], 1u);
    let new_index = base_offset + local_offset;

    particles_out[new_index] = particle;
    vel_out[new_index] = vel;
}


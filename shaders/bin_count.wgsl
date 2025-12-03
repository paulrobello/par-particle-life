// Count particles per bin.
// Each thread processes one particle and increments the corresponding bin counter.

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

@group(0) @binding(0) var<storage, read> particles: array<PosType>;
@group(0) @binding(1) var<storage, read_write> bin_counts: array<atomic<u32>>;
@group(0) @binding(2) var<uniform> params: SpatialParams;

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

    let particle = particles[id.x];
    // Cast to f32 for position calculation
    let bin_index = get_bin_index(vec2<f32>(f32(particle.x), f32(particle.y)));

    // +1 because bin_counts is a prefix sum array:
    // index 0 is always 0 (start of first bin)
    // index 1 is count of bin 0
    // index i+1 stores count of bin i
    atomicAdd(&bin_counts[bin_index + 1u], 1u);
}

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

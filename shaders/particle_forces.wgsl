// Particle force calculation compute shader.
// Calculates forces between all pairs of particles using brute force O(n²) approach.
// For optimization, consider adding spatial hashing later.

struct PosType {
    x: POS_FLOAT,
    y: POS_FLOAT,
    particle_type: u32,
    _padding: u32,
}

struct SimParams {
    num_particles: u32,
    num_types: u32,
    force_factor: f32,
    friction: f32,
    repel_strength: f32,
    max_velocity: f32,
    world_width: f32,
    world_height: f32,
    boundary_mode: u32,
    wall_repel_strength: f32,
    particle_size: f32,
    dt: f32,
    max_bin_density: f32,
    neighbor_budget: u32, // Max neighbors to check per particle (0 = unlimited)
    _padding0: u32,
    _padding1: u32,
    _padding2: u32,
    _padding3: u32,
    _padding4: u32,
    _padding5: u32,
}

@group(0) @binding(0) var<storage, read> pos_type_in: array<PosType>;
@group(0) @binding(1) var<storage, read> vel_in: array<vec2<VEL_FLOAT>>;
@group(0) @binding(2) var<storage, read_write> vel_out: array<vec2<VEL_FLOAT>>;
@group(0) @binding(3) var<uniform> params: SimParams;
@group(0) @binding(4) var<storage, read> interaction_matrix: array<f32>;
@group(0) @binding(5) var<storage, read> min_radius: array<f32>;
@group(0) @binding(6) var<storage, read> max_radius: array<f32>;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.num_particles) {
        return;
    }

    let particle = pos_type_in[i];
    let my_vel = vec2<f32>(vel_in[i]);
    let my_type = particle.particle_type;
    let my_pos = vec2<f32>(f32(particle.x), f32(particle.y));

    let half_width = params.world_width * 0.5;
    let half_height = params.world_height * 0.5;
    // Modes 1, 2, 3 all use wrapped distance calculation (Wrap, MirrorWrap, InfiniteWrap)
    let is_wrap = params.boundary_mode != 0u;

    var total_force = vec2<f32>(0.0, 0.0);

    for (var j = 0u; j < params.num_particles; j = j + 1u) {
        if (j == i) {
            continue;
        }

        let other = pos_type_in[j];
        let other_type = other.particle_type;
        let other_pos = vec2<f32>(f32(other.x), f32(other.y));

        var delta = other_pos - my_pos;

        // Handle wrapping distance calculation
        if (is_wrap) {
            if (abs(delta.x) > half_width) {
                delta.x = delta.x - sign(delta.x) * params.world_width;
            }
            if (abs(delta.y) > half_height) {
                delta.y = delta.y - sign(delta.y) * params.world_height;
            }
        }

        let dist_sq = dot(delta, delta);

        // Skip if particles are overlapping (avoid division by zero)
        if (dist_sq < 0.0001) {
            continue;
        }

        let idx = my_type * params.num_types + other_type;
        let min_r = min_radius[idx];
        let max_r = max_radius[idx];
        let max_r_sq = max_r * max_r;

        // Skip if outside interaction range
        if (dist_sq >= max_r_sq) {
            continue;
        }

        let dist = sqrt(dist_sq);
        let direction = delta / dist;

        var force_magnitude = 0.0;

        if (dist < min_r) {
            // Repulsion at close range (linear falloff)
            force_magnitude = (dist / min_r - 1.0) * params.repel_strength;
        } else {
            // Attraction/repulsion based on interaction matrix
            let strength = interaction_matrix[idx];
            let mid = (min_r + max_r) * 0.5;
            let half_range = mid - min_r;

            if (half_range > 0.0) {
                let t = abs(dist - mid) / half_range;
                force_magnitude = strength * (1.0 - t);
            }
        }

        total_force = total_force + direction * force_magnitude;
    }

    // Apply wall repulsion for Repel mode (configurable strength 0-100)
    // Uses cubic falloff for strong near-wall repulsion
    if (params.boundary_mode == 0u && params.wall_repel_strength > 0.0) {
        let wall_margin = 100.0; // Distance from wall where repulsion starts
        let wall_base_strength = params.wall_repel_strength * 0.2; // Scale 0-100 to 0-20 force

        // Left wall - cubic repulsion
        if (my_pos.x < wall_margin) {
            let t = 1.0 - my_pos.x / wall_margin; // 0 at margin, 1 at wall
            let force = wall_base_strength * t * t * t;
            total_force.x = total_force.x + force;
        }
        // Right wall
        if (my_pos.x > params.world_width - wall_margin) {
            let dist_from_wall = params.world_width - my_pos.x;
            let t = 1.0 - dist_from_wall / wall_margin;
            let force = wall_base_strength * t * t * t;
            total_force.x = total_force.x - force;
        }
        // Top wall (y=0)
        if (my_pos.y < wall_margin) {
            let t = 1.0 - my_pos.y / wall_margin;
            let force = wall_base_strength * t * t * t;
            total_force.y = total_force.y + force;
        }
        // Bottom wall
        if (my_pos.y > params.world_height - wall_margin) {
            let dist_from_wall = params.world_height - my_pos.y;
            let t = 1.0 - dist_from_wall / wall_margin;
            let force = wall_base_strength * t * t * t;
            total_force.y = total_force.y - force;
        }
    }

    // Apply force scaled by force factor
    var updated_vel = my_vel;
    updated_vel.x = updated_vel.x + total_force.x * params.force_factor;
    updated_vel.y = updated_vel.y + total_force.y * params.force_factor;

    vel_out[i] = vec2<VEL_FLOAT>(updated_vel);
}

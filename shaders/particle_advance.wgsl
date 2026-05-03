// Particle advancement compute shader.
// Updates particle positions based on velocities, applies friction, handles brush forces, and boundaries.

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
    velocity_coupling: f32,
    temperature: f32,
    frame_counter: u32,
    num_obstacles: u32,
    _padding2: u32,
    _padding3: u32,
}

struct BrushParams {
    // Brush position in world coordinates
    pos_x: f32,
    pos_y: f32,
    // Brush velocity for directional force
    vel_x: f32,
    vel_y: f32,
    // Brush radius
    radius: f32,
    // Force strength (positive = attract, negative = repel)
    force: f32,
    // Directional force multiplier
    directional_force: f32,
    // Is brush active (0 = inactive, 1 = active)
    is_active: u32,
    // Number of particles (unused in advance, but keeps struct consistent)
    num_particles: u32,
    // Target particle type (-1 for all)
    target_type: i32,
    // Padding
    _padding: vec2<u32>,
}

// Force scaling constants (matched to reference implementation)
const BRUSH_FORCE_MULTIPLIER: f32 = 50.0;
const BRUSH_DIRECTIONAL_STRENGTH: f32 = 40.0;

// PCG-style hash for GPU random noise
fn pcg_hash(input: u32) -> u32 {
    var state = input * 747796405u + 2891336453u;
    let word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    return (word >> 22u) ^ word;
}

@group(0) @binding(0) var<storage, read_write> pos: array<PosType>;
@group(0) @binding(1) var<storage, read_write> vel: array<vec2<VEL_FLOAT>>;
@group(0) @binding(2) var<uniform> params: SimParams;
@group(0) @binding(3) var<uniform> brush: BrushParams;

struct ObstacleData {
    pos_x: f32,
    pos_y: f32,
    size_a: f32,
    size_b: f32,
    shape: u32,
    bounce: f32,
}

@group(0) @binding(4) var<storage, read> obstacles: array<ObstacleData>;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.num_particles) {
        return;
    }

    var particle_pos_data = pos[i];
    var particle_pos = vec2<f32>(f32(particle_pos_data.x), f32(particle_pos_data.y));
    var particle_vel = vec2<f32>(vel[i]);
    let width = params.world_width;
    let height = params.world_height;

    // Apply brush force if active
    if (brush.is_active == 1u) {
        // Check target type filter
        var should_apply = true;
        if (brush.target_type >= 0 && i32(particle_pos_data.particle_type) != brush.target_type) {
            should_apply = false;
        }

        if (should_apply) {
            let brush_pos = vec2<f32>(brush.pos_x, brush.pos_y);
            
            var dist_vec = particle_pos - brush_pos;

            // Handle wrapping distance calculation for wrap modes
            if (params.boundary_mode != 0u) {
                dist_vec.x = dist_vec.x - width * round(dist_vec.x / width);
                dist_vec.y = dist_vec.y - height * round(dist_vec.y / height);
            }

            let dist_sq = dot(dist_vec, dist_vec);
            let radius_sq = brush.radius * brush.radius;

            if (dist_sq < radius_sq && dist_sq > 0.1) {
                let dist = sqrt(dist_sq);
                let normalized_dist = dist / brush.radius;

                // Smooth falloff using smoothstep
                let force_magnitude = 1.0 - smoothstep(0.0, 1.0, normalized_dist);

                // Radial force - positive brush.force = attract (toward brush)
                // Negate dist_vec so positive force pulls particles toward brush
                let radial_force = brush.force * force_magnitude * BRUSH_FORCE_MULTIPLIER;
                let radial_dir = -dist_vec / dist;

                // Directional force from brush movement
                let directional_force = force_magnitude * brush.directional_force * params.friction;
                let brush_vel = vec2<f32>(brush.vel_x, brush.vel_y);

                // Apply forces (multiply by dt for frame-independent movement)
                let total_force = (radial_dir * radial_force) + (brush_vel * directional_force);
                particle_vel.x = particle_vel.x + total_force.x * params.dt;
                particle_vel.y = particle_vel.y + total_force.y * params.dt;
            }
        }
    }

    // Apply friction
    let friction_factor = 1.0 - params.friction;
    particle_vel = particle_vel * friction_factor;

    // Apply temperature (Brownian noise)
    if (params.temperature > 0.0) {
        let seed1 = pcg_hash(i + params.frame_counter * 74839u);
        let seed2 = pcg_hash(seed1);
        let noise_x = f32(seed1) / 4294967295.0 - 0.5;
        let noise_y = f32(seed2) / 4294967295.0 - 0.5;
        particle_vel = particle_vel + vec2<f32>(noise_x, noise_y) * params.temperature;
    }

    // Clamp velocity
    let speed = length(particle_vel);
    if (speed > params.max_velocity) {
        let scale = params.max_velocity / speed;
        particle_vel = particle_vel * scale;
    }

    // Update position
    particle_pos.x = particle_pos.x + particle_vel.x * params.dt;
    particle_pos.y = particle_pos.y + particle_vel.y * params.dt;

    // Handle obstacle collisions
    for (var obs_idx = 0u; obs_idx < params.num_obstacles; obs_idx = obs_idx + 1u) {
        let obs = obstacles[obs_idx];
        let obs_pos = vec2<f32>(obs.pos_x, obs.pos_y);
        var delta = particle_pos - obs_pos;

        if (obs.shape == 0u) {
            // Circle collision
            let dist = length(delta);
            let radius = obs.size_a;
            if (dist < radius) {
                let normal = delta / max(dist, 0.001);
                particle_pos = obs_pos + normal * radius;
                let vel_dot_normal = dot(particle_vel, normal);
                if (vel_dot_normal < 0.0) {
                    particle_vel = particle_vel - (1.0 + obs.bounce) * vel_dot_normal * normal;
                }
            }
        } else {
            // Rectangle collision
            let half_w = obs.size_a;
            let half_h = obs.size_b;

            if (abs(delta.x) < half_w && abs(delta.y) < half_h) {
                let dx = half_w - abs(delta.x);
                let dy = half_h - abs(delta.y);

                if (dx < dy) {
                    let sign_x = select(1.0, -1.0, delta.x < 0.0);
                    particle_pos.x = obs_pos.x + sign_x * half_w;
                    particle_vel.x = -particle_vel.x * obs.bounce;
                } else {
                    let sign_y = select(1.0, -1.0, delta.y < 0.0);
                    particle_pos.y = obs_pos.y + sign_y * half_h;
                    particle_vel.y = -particle_vel.y * obs.bounce;
                }
            }
        }
    }

    let margin = params.particle_size;

    // Handle boundaries
    if (params.boundary_mode == 0u) {
        // Repel mode - bounce off walls
        if (particle_pos.x < margin) {
            particle_pos.x = margin;
            particle_vel.x = abs(particle_vel.x);
        } else if (particle_pos.x > width - margin) {
            particle_pos.x = width - margin;
            particle_vel.x = -abs(particle_vel.x);
        }

        if (particle_pos.y < margin) {
            particle_pos.y = margin;
            particle_vel.y = abs(particle_vel.y);
        } else if (particle_pos.y > height - margin) {
            particle_pos.y = height - margin;
            particle_vel.y = -abs(particle_vel.y);
        }
    } else {
        // Wrap mode (modes 1, 2, 3) - teleport to opposite side
        if (particle_pos.x < 0.0) {
            particle_pos.x = particle_pos.x + width;
        } else if (particle_pos.x >= width) {
            particle_pos.x = particle_pos.x - width;
        }

        if (particle_pos.y < 0.0) {
            particle_pos.y = particle_pos.y + height;
        } else if (particle_pos.y >= height) {
            particle_pos.y = particle_pos.y - height;
        }
    }

    // Write back to buffers
    particle_pos_data.x = POS_FLOAT(particle_pos.x);
    particle_pos_data.y = POS_FLOAT(particle_pos.y);
    pos[i] = particle_pos_data;
    vel[i] = vec2<VEL_FLOAT>(particle_vel);
}

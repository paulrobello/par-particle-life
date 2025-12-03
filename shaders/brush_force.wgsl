// Brush force compute shader.
// Applies attraction/repulsion forces from user brush interaction.
// Supports attract and repel modes with smooth falloff.

struct PosType {
    x: POS_FLOAT,
    y: POS_FLOAT,
    particle_type: u32,
    _padding: u32,
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
    // Number of particles
    num_particles: u32,
    // Target particle type (-1 for all)
    target_type: i32,
    // Padding
    _padding: vec2<u32>,
}

// Force scaling constants
// Multiplied by UI force value (0-100) to get actual velocity change per frame
const BRUSH_FORCE_MULTIPLIER: f32 = 50.0;
const BRUSH_DIRECTIONAL_STRENGTH: f32 = 0.5;

@group(0) @binding(0) var<storage, read> pos_type: array<PosType>;
@group(0) @binding(1) var<storage, read_write> velocities: array<vec2<VEL_FLOAT>>;
@group(0) @binding(2) var<uniform> brush: BrushParams;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= brush.num_particles) {
        return;
    }

    // Skip if brush is not active
    if (brush.is_active == 0u) {
        return;
    }

    let particle_pos_data = pos_type[i];

    // Check target type filter
    if (brush.target_type >= 0 && i32(particle_pos_data.particle_type) != brush.target_type) {
        return;
    }

    let brush_pos = vec2<f32>(brush.pos_x, brush.pos_y);
    let particle_pos = vec2<f32>(f32(particle_pos_data.x), f32(particle_pos_data.y));

    // Calculate distance to brush
    let delta = particle_pos - brush_pos;
    let dist = length(delta);

    // Skip if outside brush radius
    if (dist >= brush.radius) {
        return;
    }

    // Avoid division by zero at center
    if (dist < 0.1) {
        return;
    }

    // Calculate normalized distance (0 at center, 1 at edge)
    let normalized_dist = dist / brush.radius;

    // Smooth falloff using smoothstep (stronger at center, weaker at edge)
    let force_magnitude = 1.0 - smoothstep(0.0, 1.0, normalized_dist);

    // Calculate radial force (toward or away from brush)
    let direction = delta / dist;  // Unit vector from brush to particle
    let radial_force = brush.force * force_magnitude * BRUSH_FORCE_MULTIPLIER;

    // Calculate directional force from brush movement
    let brush_vel = vec2<f32>(brush.vel_x, brush.vel_y);
    let directional_strength = force_magnitude * brush.directional_force * BRUSH_DIRECTIONAL_STRENGTH;

    // Apply forces to particle velocity
    var vel = vec2<f32>(velocities[i]);
    // Positive force = attract = move toward brush = negative direction
    // Negative force = repel = move away from brush = positive direction
    vel.x = vel.x - direction.x * radial_force + brush_vel.x * directional_strength;
    vel.y = vel.y - direction.y * radial_force + brush_vel.y * directional_strength;

    velocities[i] = vec2<VEL_FLOAT>(vel);
}

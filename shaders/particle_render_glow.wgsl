// Particle glow rendering shader.
// Draws particles as larger quads with radial falloff for glow effect.
// Uses additive blending for bright, overlapping glow.

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

struct Camera {
    center_x: f32,
    center_y: f32,
    scale_x: f32,
    scale_y: f32,
}

struct GlowParams {
    /// Multiplier for glow quad size (typically 2.0-8.0).
    glow_size: f32,
    /// Intensity of the glow (0.0-2.0).
    glow_intensity: f32,
    /// Steepness of falloff (higher = sharper edge, 1.0-4.0).
    glow_steepness: f32,
    _padding: f32,
}

// Quad vertices for instanced rendering
const QUAD_VERTICES = array<vec2<f32>, 4>(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>( 1.0, -1.0),
    vec2<f32>(-1.0,  1.0),
    vec2<f32>( 1.0,  1.0)
);

@group(0) @binding(0) var<storage, read> particles: array<PosType>;
@group(0) @binding(1) var<storage, read> colors: array<vec4<f32>>;
@group(0) @binding(2) var<uniform> params: SimParams;
@group(0) @binding(3) var<uniform> camera: Camera;
@group(0) @binding(4) var<uniform> glow: GlowParams;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) offset: vec2<f32>,
    @location(1) color: vec4<f32>,
}

@vertex
fn vs_main(
    @builtin(instance_index) instance_index: u32,
    @builtin(vertex_index) vertex_index: u32
) -> VertexOutput {
    let particle = particles[instance_index];
    let color = colors[particle.particle_type];
    let particle_pos = vec2<f32>(f32(particle.x), f32(particle.y));

    // Transform particle position to clip space
    let camera_scale = vec2<f32>(camera.scale_x, -camera.scale_y);
    let camera_center = vec2<f32>(camera.center_x, camera.center_y);
    let transformed_pos = (particle_pos - camera_center) * camera_scale;

    // Get quad vertex offset - scaled by glow_size for larger glow effect
    let quad_offset = QUAD_VERTICES[vertex_index];
    let glow_particle_size = params.particle_size * glow.glow_size;
    let vertex_offset = quad_offset * glow_particle_size * camera_scale;
    let final_pos = transformed_pos + vertex_offset;

    var output: VertexOutput;
    output.position = vec4<f32>(final_pos, 0.0, 1.0);
    output.offset = quad_offset;
    output.color = color;
    return output;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Calculate distance from center of quad
    let dist_sq = dot(in.offset, in.offset);

    // Discard pixels outside the circle
    if (dist_sq > 1.0) {
        discard;
    }

    // Radial falloff from center
    let falloff = saturate(1.0 - dist_sq);

    // Apply steepness and intensity
    let alpha = pow(falloff, glow.glow_steepness) * glow.glow_intensity;

    // Discard very dim pixels
    if (alpha < 0.001) {
        discard;
    }

    // Pre-multiplied alpha for additive blending
    return vec4<f32>(in.color.rgb * alpha, alpha);
}

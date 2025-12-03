// Particle rendering shader.
// Draws particles as point sprites (quads) with smooth circular appearance.

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
    // Center of viewport in world coordinates
    center_x: f32,
    center_y: f32,
    // Scale factors (pixels per world unit)
    scale_x: f32,
    scale_y: f32,
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

    // Get quad vertex offset
    let quad_offset = QUAD_VERTICES[vertex_index];
    let vertex_offset = quad_offset * params.particle_size * camera_scale;
    let final_pos = transformed_pos + vertex_offset;

    var output: VertexOutput;
    output.position = vec4<f32>(final_pos, 0.0, 1.0);
    output.offset = quad_offset;
    output.color = color;
    return output;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Calculate distance from center of quad (0,0)
    let dist_sq = dot(in.offset, in.offset);

    // Smooth edge using derivative-based anti-aliasing
    let edge_width = fwidth(dist_sq);
    let alpha = 1.0 - smoothstep(max(0.0, 1.0 - edge_width), 1.0, dist_sq);

    // Discard pixels outside the circle
    if (alpha < 0.01) {
        discard;
    }

    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}

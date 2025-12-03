// Particle mirror wrap rendering shader.
// Renders particles with N copies at mirror positions for wrap visualization.
// Mirror copies are rendered at reduced opacity to distinguish from originals.

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

struct MirrorParams {
    // Number of copies to render (5 or 9)
    num_copies: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
}

// Mirror offsets for 5-copy mode (center + 4 cardinal directions)
const MIRROR_OFFSETS_5 = array<vec2<f32>, 5>(
    vec2<f32>(0.0, 0.0),    // Original
    vec2<f32>(-1.0, 0.0),   // Left
    vec2<f32>(1.0, 0.0),    // Right
    vec2<f32>(0.0, -1.0),   // Top
    vec2<f32>(0.0, 1.0)     // Bottom
);

// Mirror offsets for 9-copy mode (center + all 8 directions)
const MIRROR_OFFSETS_9 = array<vec2<f32>, 9>(
    vec2<f32>(0.0, 0.0),    // Original
    vec2<f32>(-1.0, 0.0),   // Left
    vec2<f32>(1.0, 0.0),    // Right
    vec2<f32>(0.0, -1.0),   // Top
    vec2<f32>(0.0, 1.0),    // Bottom
    vec2<f32>(-1.0, -1.0),  // Top-left
    vec2<f32>(1.0, -1.0),   // Top-right
    vec2<f32>(-1.0, 1.0),   // Bottom-left
    vec2<f32>(1.0, 1.0)     // Bottom-right
);

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
@group(0) @binding(4) var<uniform> mirror: MirrorParams;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) offset: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) is_mirror: f32,
}

@vertex
fn vs_main(
    @builtin(instance_index) instance_index: u32,
    @builtin(vertex_index) vertex_index: u32
) -> VertexOutput {
    let num_copies = mirror.num_copies;

    // Decompose instance_index into particle_index and mirror_index
    let particle_index = instance_index / num_copies;
    let mirror_index = instance_index % num_copies;

    let particle = particles[particle_index];
    let base_color = colors[particle.particle_type];
    let particle_pos = vec2<f32>(f32(particle.x), f32(particle.y));

    // Get mirror offset based on copy count
    var mirror_offset: vec2<f32>;
    if (num_copies == 5u) {
        mirror_offset = MIRROR_OFFSETS_5[mirror_index];
    } else {
        mirror_offset = MIRROR_OFFSETS_9[mirror_index];
    }

    // Apply mirror offset (in world units)
    let world_size = vec2<f32>(params.world_width, params.world_height);
    let world_pos = particle_pos + mirror_offset * world_size;

    // Transform to clip space
    let camera_scale = vec2<f32>(camera.scale_x, -camera.scale_y);
    let camera_center = vec2<f32>(camera.center_x, camera.center_y);
    let transformed_pos = (world_pos - camera_center) * camera_scale;

    // Get quad vertex offset
    let quad_offset = QUAD_VERTICES[vertex_index];
    let vertex_offset = quad_offset * params.particle_size * camera_scale;
    let final_pos = transformed_pos + vertex_offset;

    // Determine if this is a mirror copy (not the original)
    let is_mirror = f32(mirror_index != 0u);

    // Reduce opacity for mirror copies (75% of original) to distinguish from originals
    let final_alpha = mix(base_color.a, base_color.a * 0.75, is_mirror);

    var output: VertexOutput;
    output.position = vec4<f32>(final_pos, 0.0, 1.0);
    output.offset = quad_offset;
    output.color = vec4<f32>(base_color.rgb, final_alpha);
    output.is_mirror = is_mirror;
    return output;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let dist_sq = dot(in.offset, in.offset);

    let edge_width = fwidth(dist_sq);
    let alpha = 1.0 - smoothstep(max(0.0, 1.0 - edge_width), 1.0, dist_sq);

    if (alpha < 0.01) {
        discard;
    }

    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}

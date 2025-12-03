// Particle infinite wrap rendering shader.
// Renders particles as a seamless tiled grid based on camera viewport.
// All tiles are rendered with full color and opacity (no grayscale effect).

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

struct InfiniteParams {
    // Start offset for tile grid (can be negative)
    start_x: i32,
    start_y: i32,
    // Number of tile copies in each direction
    num_copies_x: u32,
    num_copies_y: u32,
    // Padding to match Rust struct alignment (not strictly needed but good practice)
}

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
@group(0) @binding(4) var<uniform> infinite: InfiniteParams;

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
    let total_copies = infinite.num_copies_x * infinite.num_copies_y;

    // Decompose instance_index into particle_index and copy_index
    let particle_index = instance_index / total_copies;
    let copy_index = instance_index % total_copies;

    // Decompose copy_index into tile x and y
    let copy_x = copy_index % infinite.num_copies_x;
    let copy_y = copy_index / infinite.num_copies_x;

    let particle = particles[particle_index];
    let base_color = colors[particle.particle_type];
    let particle_pos = vec2<f32>(f32(particle.x), f32(particle.y));

    // Calculate tile offset (including start offset for centering on camera)
    let tile_offset_x = f32(i32(copy_x) + infinite.start_x);
    let tile_offset_y = f32(i32(copy_y) + infinite.start_y);

    // Apply tile offset (in world units)
    let world_size = vec2<f32>(params.world_width, params.world_height);
    let world_pos = particle_pos + vec2<f32>(tile_offset_x, tile_offset_y) * world_size;

    // Transform to clip space
    let camera_scale = vec2<f32>(camera.scale_x, -camera.scale_y);
    let camera_center = vec2<f32>(camera.center_x, camera.center_y);
    let transformed_pos = (world_pos - camera_center) * camera_scale;

    // Get quad vertex offset
    let quad_offset = QUAD_VERTICES[vertex_index];
    let vertex_offset = quad_offset * params.particle_size * camera_scale;
    let final_pos = transformed_pos + vertex_offset;

    var output: VertexOutput;
    output.position = vec4<f32>(final_pos, 0.0, 1.0);
    output.offset = quad_offset;
    output.color = base_color;
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

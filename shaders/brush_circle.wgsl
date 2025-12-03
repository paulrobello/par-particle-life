// Brush circle rendering shader.
// Renders a circular indicator showing brush position and radius.

struct BrushRenderParams {
    // Brush position in world coordinates
    pos_x: f32,
    pos_y: f32,
    // Brush radius
    radius: f32,
    // Circle color (RGBA)
    color_r: f32,
    color_g: f32,
    color_b: f32,
    color_a: f32,
    // Is brush visible
    is_visible: u32,
    // Camera transform
    world_width: f32,
    world_height: f32,
    camera_zoom: f32,
    camera_offset_x: f32,
    camera_offset_y: f32,
    // Padding
    _padding: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@group(0) @binding(0) var<uniform> brush: BrushRenderParams;

// Circle line thickness in world units
const LINE_THICKNESS: f32 = 2.0;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    // Generate a quad that covers the brush circle with some padding
    let padding = LINE_THICKNESS * 2.0;
    let size = (brush.radius + padding) * 2.0;

    // Quad vertices (0,1,2,3 -> triangle strip)
    let x = f32(vertex_index & 1u);
    let y = f32((vertex_index >> 1u) & 1u);

    // Local position relative to brush center (-1 to 1 range)
    let local_pos = vec2<f32>(x * 2.0 - 1.0, y * 2.0 - 1.0);

    // World position
    let world_pos = vec2<f32>(
        brush.pos_x + local_pos.x * (brush.radius + padding),
        brush.pos_y + local_pos.y * (brush.radius + padding)
    );

    // Apply camera transform
    let world_center = vec2<f32>(brush.world_width * 0.5, brush.world_height * 0.5);
    let centered = world_pos - world_center - vec2<f32>(brush.camera_offset_x, brush.camera_offset_y);
    let scaled = centered * brush.camera_zoom;

    // Convert to clip space (-1 to 1)
    let clip_pos = scaled / vec2<f32>(brush.world_width * 0.5, brush.world_height * 0.5);

    var output: VertexOutput;
    output.position = vec4<f32>(clip_pos.x, -clip_pos.y, 0.0, 1.0);
    output.uv = local_pos;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Skip if not visible
    if (brush.is_visible == 0u) {
        discard;
    }

    // Calculate distance from center in UV space (-1 to 1)
    let dist = length(input.uv);

    // Normalized distance relative to radius
    let radius_normalized = brush.radius / (brush.radius + LINE_THICKNESS * 2.0);

    // Line thickness in normalized space
    let thickness = LINE_THICKNESS / (brush.radius + LINE_THICKNESS * 2.0);

    // Check if we're on the circle edge
    let inner = radius_normalized - thickness;
    let outer = radius_normalized + thickness;

    if (dist < inner || dist > outer) {
        discard;
    }

    // Smooth the circle edge
    let edge_dist = abs(dist - radius_normalized) / thickness;
    let alpha = 1.0 - smoothstep(0.5, 1.0, edge_dist);

    return vec4<f32>(
        brush.color_r,
        brush.color_g,
        brush.color_b,
        brush.color_a * alpha
    );
}

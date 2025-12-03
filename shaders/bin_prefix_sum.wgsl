// Prefix sum step for computing bin offsets.
// Performs one step of the Hillis-Steele parallel prefix sum algorithm.
// Multiple passes are needed: step_size = 1, 2, 4, 8, ... until step_size >= total_bins

@group(0) @binding(0) var<storage, read> source: array<u32>;
@group(0) @binding(1) var<storage, read_write> destination: array<u32>;
@group(0) @binding(2) var<uniform> step_size: u32;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let idx = id.x;
    if (idx >= arrayLength(&source)) {
        return;
    }

    if (idx < step_size) {
        // Elements before step_size just copy their value
        destination[idx] = source[idx];
    } else {
        // Add the element step_size positions back
        destination[idx] = source[idx - step_size] + source[idx];
    }
}

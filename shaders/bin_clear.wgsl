// Clear bin counters to zero.
// Run before counting particles per bin.

@group(0) @binding(0) var<storage, read_write> bin_counts: array<atomic<u32>>;
@group(0) @binding(1) var<uniform> total_bins: u32;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= total_bins) {
        return;
    }
    atomicStore(&bin_counts[id.x], 0u);
}

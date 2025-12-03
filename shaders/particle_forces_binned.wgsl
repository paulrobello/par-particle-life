// Particle force calculation using spatial hashing.
// Only checks particles in neighboring bins (3x3 neighborhood).
// Complexity: O(n * k) where k is average particles per neighborhood.

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

struct SpatialParams {
    num_particles: u32,
    cell_size: f32,
    grid_width: u32,
    grid_height: u32,
}

@group(0) @binding(1) var<storage, read_write> velocities: array<vec2<VEL_FLOAT>>;
@group(0) @binding(2) var<uniform> params: SimParams;
@group(0) @binding(3) var<storage, read> interaction_matrix: array<f32>;
@group(0) @binding(4) var<storage, read> min_radius: array<f32>;
@group(0) @binding(5) var<storage, read> max_radius: array<f32>;
@group(0) @binding(6) var<storage, read> bin_offsets: array<u32>;
@group(0) @binding(7) var<uniform> spatial: SpatialParams;
@group(0) @binding(8) var<storage, read> sorted_pos_type: array<PosType>;

fn get_bin_coords(pos: vec2<f32>) -> vec2<i32> {
    return vec2<i32>(
        i32(floor(pos.x / spatial.cell_size)),
        i32(floor(pos.y / spatial.cell_size))
    );
}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    // Iterate over SORTED particles to ensure memory coherence for position/bin lookup
    let sorted_idx = id.x;
    if (sorted_idx >= params.num_particles) {
        return;
    }

    let particle = sorted_pos_type[sorted_idx];
    let my_type = particle.particle_type;
    let my_pos = vec2<f32>(f32(particle.x), f32(particle.y));

    // Velocity buffer is now sorted, so we can access it directly
    let my_vel = vec2<f32>(velocities[sorted_idx]);

    let half_width = params.world_width * 0.5;
    let half_height = params.world_height * 0.5;
    // Modes 1, 2, 3 all use wrapped distance calculation (Wrap, MirrorWrap, InfiniteWrap)
    let is_wrap = params.boundary_mode != 0u;

    // Get this particle's bin coordinates
    let my_bin = get_bin_coords(my_pos);
    let grid_w = i32(spatial.grid_width);
    let grid_h = i32(spatial.grid_height);

    var total_force = vec2<f32>(0.0, 0.0);
    var total_particles_in_neighborhood = 0u;
    var neighbors_checked = 0u;
    let budget = params.neighbor_budget;

    // Calculate per-bin budget to ensure symmetric sampling across all 9 bins
    // This prevents asymmetric forces that cause particles to band horizontally
    let per_bin_budget = select(0u, (budget + 8u) / 9u, budget > 0u);

    // Check 3x3 neighborhood of bins - always visit all 9 bins for symmetric physics
    for (var dy = -1; dy <= 1; dy = dy + 1) {
        for (var dx = -1; dx <= 1; dx = dx + 1) {
            var bin_x = my_bin.x + dx;
            var bin_y = my_bin.y + dy;

            // Handle boundary conditions for bin lookup
            if (is_wrap) {
                // Wrap bin coordinates
                if (bin_x < 0) { bin_x = bin_x + grid_w; }
                else if (bin_x >= grid_w) { bin_x = bin_x - grid_w; }
                if (bin_y < 0) { bin_y = bin_y + grid_h; }
                else if (bin_y >= grid_h) { bin_y = bin_y - grid_h; }
            } else {
                // Skip out-of-bounds bins for non-wrapping modes
                if (bin_x < 0 || bin_x >= grid_w || bin_y < 0 || bin_y >= grid_h) {
                    continue;
                }
            }

            let bin_index = u32(bin_y * grid_w + bin_x);
            // bin_offsets[i] is the start offset for bin i (due to +1 shift in counting)
            // bin_offsets[i+1] is the end offset for bin i
            let bin_start = bin_offsets[bin_index];
            let bin_end = bin_offsets[bin_index + 1u];
            
            total_particles_in_neighborhood = total_particles_in_neighborhood + (bin_end - bin_start);

            // Track neighbors checked for this bin (reset per bin for fair sampling)
            var bin_neighbors_checked = 0u;

            // Iterate over particles in this bin
            for (var j = bin_start; j < bin_end; j = j + 1u) {
                // Early exit if we've hit the per-bin budget (ensures symmetric sampling)
                if (per_bin_budget > 0u && bin_neighbors_checked >= per_bin_budget) {
                    break;
                }

                // Skip self-interaction.
                // Since we are iterating sorted buffer, j is a sorted index.
                // If j == sorted_idx, it's definitely us.
                if (j == sorted_idx) {
                    continue;
                }

                bin_neighbors_checked = bin_neighbors_checked + 1u;
                neighbors_checked = neighbors_checked + 1u;
                let other = sorted_pos_type[j];
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
                // This effectively skips self-interaction too if position is identical.
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
        }
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
    var final_force = total_force * params.force_factor;

    // Apply density-based force scaling if max_bin_density is set (non-zero)
    // This reduces forces in very dense clusters to prevent explosions and stabilize performance
    if (params.max_bin_density > 0.0) {
        let neighborhood_particle_count = f32(total_particles_in_neighborhood);
        if (neighborhood_particle_count > params.max_bin_density) {
            let scale_factor = params.max_bin_density / neighborhood_particle_count;
            final_force = final_force * scale_factor;
        }
    }

    var updated_vel = my_vel;
    updated_vel.x = updated_vel.x + final_force.x;
    updated_vel.y = updated_vel.y + final_force.y;

    velocities[sorted_idx] = vec2<VEL_FLOAT>(updated_vel);
}

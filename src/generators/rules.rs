//! Rule generators for creating interaction matrices.
//!
//! This module contains 31 different algorithms for generating
//! particle interaction matrices, ranging from simple random
//! patterns to complex mathematical constructs.

use rand::Rng;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

use crate::simulation::InteractionMatrix;

/// Types of rule generators available.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum RuleType {
    #[default]
    Random = 0,
    Symmetric = 1,
    Snake = 2,
    Chains1 = 3,
    Chains2 = 4,
    Chains3 = 5,
    RockPaperScissors = 6,
    BipartiteAlliances = 7,
    HubAndSpokes = 8,
    ConcentricShells = 9,
    AntiSymmetricSwirl = 10,
    CheckerOffsets = 11,
    DimersAndChains = 12,
    TriadFlocks = 13,
    SpiralConveyor = 14,
    Patchwork = 15,
    Wavefield = 16,
    ChiralBandpass = 17,
    RotatingConveyor = 18,
    PrimeHop = 19,
    ParityVortex = 20,
    HelicalLadder = 21,
    BiasedWave = 22,
    ModularTriads = 23,
    SkippedPursuit = 24,
    BlueNoiseConveyor = 25,
    OffsetPhasefield = 26,
    RingRoad = 27,
    TriSpiral = 28,
    VortexAntivortex = 29,
    DriftedPatchwork = 30,
}

impl RuleType {
    /// Get all available rule types.
    pub fn all() -> &'static [RuleType] {
        &[
            RuleType::Random,
            RuleType::Symmetric,
            RuleType::Snake,
            RuleType::Chains1,
            RuleType::Chains2,
            RuleType::Chains3,
            RuleType::RockPaperScissors,
            RuleType::BipartiteAlliances,
            RuleType::HubAndSpokes,
            RuleType::ConcentricShells,
            RuleType::AntiSymmetricSwirl,
            RuleType::CheckerOffsets,
            RuleType::DimersAndChains,
            RuleType::TriadFlocks,
            RuleType::SpiralConveyor,
            RuleType::Patchwork,
            RuleType::Wavefield,
            RuleType::ChiralBandpass,
            RuleType::RotatingConveyor,
            RuleType::PrimeHop,
            RuleType::ParityVortex,
            RuleType::HelicalLadder,
            RuleType::BiasedWave,
            RuleType::ModularTriads,
            RuleType::SkippedPursuit,
            RuleType::BlueNoiseConveyor,
            RuleType::OffsetPhasefield,
            RuleType::RingRoad,
            RuleType::TriSpiral,
            RuleType::VortexAntivortex,
            RuleType::DriftedPatchwork,
        ]
    }

    /// Get the display name for this rule type.
    pub fn display_name(&self) -> &'static str {
        match self {
            RuleType::Random => "Random",
            RuleType::Symmetric => "Symmetric",
            RuleType::Snake => "Snake",
            RuleType::Chains1 => "Chains 1",
            RuleType::Chains2 => "Chains 2",
            RuleType::Chains3 => "Chains 3",
            RuleType::RockPaperScissors => "Rock–Paper–Scissors",
            RuleType::BipartiteAlliances => "Bipartite Alliances",
            RuleType::HubAndSpokes => "Hub and Spokes",
            RuleType::ConcentricShells => "Concentric Shells",
            RuleType::AntiSymmetricSwirl => "Anti-symmetric Swirl",
            RuleType::CheckerOffsets => "Checker Offsets",
            RuleType::DimersAndChains => "Dimers & Chains",
            RuleType::TriadFlocks => "Triad Flocks",
            RuleType::SpiralConveyor => "Spiral Conveyor",
            RuleType::Patchwork => "Patchwork",
            RuleType::Wavefield => "Wavefield",
            RuleType::ChiralBandpass => "Chiral Bandpass",
            RuleType::RotatingConveyor => "Rotating Conveyor",
            RuleType::PrimeHop => "Prime Hop",
            RuleType::ParityVortex => "Parity Vortex",
            RuleType::HelicalLadder => "Helical Ladder",
            RuleType::BiasedWave => "Biased Wave",
            RuleType::ModularTriads => "Modular Triads",
            RuleType::SkippedPursuit => "Skipped Pursuit",
            RuleType::BlueNoiseConveyor => "Blue-Noise Conveyor",
            RuleType::OffsetPhasefield => "Offset Phasefield",
            RuleType::RingRoad => "Ring Road",
            RuleType::TriSpiral => "Tri-Spiral",
            RuleType::VortexAntivortex => "Vortex–Antivortex Lattice",
            RuleType::DriftedPatchwork => "Drifted Patchwork",
        }
    }

    /// Get the category for this rule type.
    pub fn category(&self) -> &'static str {
        match self {
            RuleType::Random => "Default",
            _ => "Experimental",
        }
    }
}

/// Rule generator trait for creating interaction matrices.
pub trait RuleGenerator {
    /// Generate an interaction matrix of the given size.
    fn generate(&self, num_types: usize) -> InteractionMatrix;
}

impl RuleGenerator for RuleType {
    fn generate(&self, num_types: usize) -> InteractionMatrix {
        generate_rules(*self, num_types)
    }
}

/// Generate an interaction matrix using the specified rule type.
pub fn generate_rules(rule_type: RuleType, num_types: usize) -> InteractionMatrix {
    if num_types == 0 {
        return InteractionMatrix::new(0);
    }

    let mut matrix = match rule_type {
        RuleType::Random => random_generator(num_types),
        RuleType::Symmetric => symmetric_generator(num_types),
        RuleType::Snake => snake_generator(num_types),
        RuleType::Chains1 => chains1_generator(num_types),
        RuleType::Chains2 => chains2_generator(num_types),
        RuleType::Chains3 => chains3_generator(num_types),
        RuleType::RockPaperScissors => rock_paper_scissors_generator(num_types),
        RuleType::BipartiteAlliances => bipartite_generator(num_types),
        RuleType::HubAndSpokes => hub_and_spokes_generator(num_types),
        RuleType::ConcentricShells => concentric_shells_generator(num_types),
        RuleType::AntiSymmetricSwirl => anti_symmetric_swirl_generator(num_types),
        RuleType::CheckerOffsets => checker_offsets_generator(num_types),
        RuleType::DimersAndChains => dimers_and_chains_generator(num_types),
        RuleType::TriadFlocks => triad_flocks_generator(num_types),
        RuleType::SpiralConveyor => spiral_conveyor_generator(num_types),
        RuleType::Patchwork => patchwork_generator(num_types),
        RuleType::Wavefield => wavefield_generator(num_types),
        RuleType::ChiralBandpass => chiral_bandpass_generator(num_types),
        RuleType::RotatingConveyor => rotating_conveyor_generator(num_types),
        RuleType::PrimeHop => prime_hop_generator(num_types),
        RuleType::ParityVortex => parity_vortex_generator(num_types),
        RuleType::HelicalLadder => helical_ladder_generator(num_types),
        RuleType::BiasedWave => biased_wave_generator(num_types),
        RuleType::ModularTriads => modular_triads_generator(num_types),
        RuleType::SkippedPursuit => skipped_pursuit_generator(num_types),
        RuleType::BlueNoiseConveyor => blue_noise_conveyor_generator(num_types),
        RuleType::OffsetPhasefield => offset_phasefield_generator(num_types),
        RuleType::RingRoad => ring_road_generator(num_types),
        RuleType::TriSpiral => tri_spiral_generator(num_types),
        RuleType::VortexAntivortex => vortex_antivortex_generator(num_types),
        RuleType::DriftedPatchwork => drifted_patchwork_generator(num_types),
    };

    // Round all values to 2 decimal places for consistency
    for val in &mut matrix.data {
        *val = (*val * 100.0).round() / 100.0;
    }

    matrix
}

// === Generator Implementations ===

/// Random matrix with values in [-1, 1).
fn random_generator(n: usize) -> InteractionMatrix {
    let mut rng = rand::rng();
    let mut matrix = InteractionMatrix::new(n);
    for val in &mut matrix.data {
        *val = rng.random::<f32>() * 2.0 - 1.0;
    }
    matrix
}

/// Symmetric matrix (m[i][j] = m[j][i]).
fn symmetric_generator(n: usize) -> InteractionMatrix {
    let mut matrix = random_generator(n);
    matrix.symmetrize();
    matrix
}

/// Snake pattern: each type follows the next.
fn snake_generator(n: usize) -> InteractionMatrix {
    let mut matrix = InteractionMatrix::new(n);
    for i in 0..n {
        for j in 0..n {
            let val = if j == i {
                1.0
            } else if j == (i + 1) % n {
                0.2
            } else {
                0.0
            };
            matrix.set(i, j, val);
        }
    }
    matrix
}

/// Chains pattern 1: strong attraction to self and neighbors, repel others.
fn chains1_generator(n: usize) -> InteractionMatrix {
    let mut matrix = InteractionMatrix::new(n);
    for i in 0..n {
        for j in 0..n {
            let val = if j == i || j == (i + 1) % n || j == (i + n - 1) % n {
                1.0
            } else {
                -1.0
            };
            matrix.set(i, j, val);
        }
    }
    matrix
}

/// Chains pattern 2: self-attraction, weak neighbor attraction, repel others.
fn chains2_generator(n: usize) -> InteractionMatrix {
    let mut matrix = InteractionMatrix::new(n);
    for i in 0..n {
        for j in 0..n {
            let val = if j == i {
                1.0
            } else if j == (i + 1) % n || j == (i + n - 1) % n {
                0.2
            } else {
                -1.0
            };
            matrix.set(i, j, val);
        }
    }
    matrix
}

/// Chains pattern 3: like chains2 but neutral to distant types.
fn chains3_generator(n: usize) -> InteractionMatrix {
    let mut matrix = InteractionMatrix::new(n);
    for i in 0..n {
        for j in 0..n {
            let val = if j == i {
                1.0
            } else if j == (i + 1) % n || j == (i + n - 1) % n {
                0.2
            } else {
                0.0
            };
            matrix.set(i, j, val);
        }
    }
    matrix
}

/// Rock-Paper-Scissors: cyclic dominance pattern.
fn rock_paper_scissors_generator(n: usize) -> InteractionMatrix {
    const A: f32 = 0.9; // Attraction to prey
    const R: f32 = -0.7; // Repulsion from predator
    const S: f32 = -0.1; // Self interaction

    let mut matrix = InteractionMatrix::new(n);
    for i in 0..n {
        for j in 0..n {
            let val = if j == i {
                S
            } else if j == (i + 1) % n {
                A
            } else if j == (i + n - 1) % n {
                R
            } else {
                0.0
            };
            matrix.set(i, j, val);
        }
    }
    matrix
}

/// Bipartite: two groups with intra-attraction and inter-repulsion.
fn bipartite_generator(n: usize) -> InteractionMatrix {
    const INTRA: f32 = 0.8;
    const INTER: f32 = -0.8;
    const SELF: f32 = 0.2;

    let mut matrix = InteractionMatrix::new(n);
    for i in 0..n {
        for j in 0..n {
            let same_bloc = (i % 2) == (j % 2);
            let val = if j == i {
                SELF
            } else if same_bloc {
                INTRA
            } else {
                INTER
            };
            matrix.set(i, j, val);
        }
    }
    matrix
}

/// Hub and Spokes: type 0 is central hub.
fn hub_and_spokes_generator(n: usize) -> InteractionMatrix {
    const CORE: f32 = 0.0;
    const INTRA: f32 = 0.0;
    const TO_CORE: f32 = 1.0;
    const FROM_CORE: f32 = 0.6;

    let mut matrix = InteractionMatrix::new(n);
    for i in 0..n {
        for j in 0..n {
            let val = if i == j {
                CORE
            } else if i == 0 {
                TO_CORE
            } else if j == 0 {
                FROM_CORE
            } else {
                INTRA
            };
            matrix.set(i, j, val);
        }
    }
    matrix
}

/// Concentric Shells: nested ring structure.
fn concentric_shells_generator(n: usize) -> InteractionMatrix {
    const SELF: f32 = 0.9;
    const NEXT: f32 = 0.3;
    const FAR: f32 = -0.6;

    let mut matrix = InteractionMatrix::new(n);
    for i in 0..n {
        for j in 0..n {
            let val = if j == i {
                SELF
            } else if j == (i + 1) % n {
                NEXT
            } else {
                FAR
            };
            matrix.set(i, j, val);
        }
    }
    matrix
}

/// Anti-symmetric Swirl: creates rotational patterns.
fn anti_symmetric_swirl_generator(n: usize) -> InteractionMatrix {
    const BASE: f32 = 0.7;

    let mut matrix = InteractionMatrix::new(n);
    for i in 0..n {
        for j in (i + 1)..n {
            let dist = (j - i + n) % n;
            let val = if dist <= n / 2 { BASE } else { -BASE };
            matrix.set(i, j, val);
            matrix.set(j, i, -val);
        }
        matrix.set(i, i, -0.05);
    }
    matrix
}

/// Checker Offsets: alternating pattern.
fn checker_offsets_generator(n: usize) -> InteractionMatrix {
    const NEI: f32 = -0.8;
    const SKIP: f32 = 0.8;
    const SELF: f32 = 0.2;

    let mut matrix = InteractionMatrix::new(n);
    for i in 0..n {
        for j in 0..n {
            let val = if j == i {
                SELF
            } else if j == (i + 1) % n || j == (i + n - 1) % n {
                NEI
            } else if j == (i + 2) % n || j == (i + n - 2) % n {
                SKIP
            } else {
                0.0
            };
            matrix.set(i, j, val);
        }
    }
    matrix
}

/// Dimers and Chains: paired types.
fn dimers_and_chains_generator(n: usize) -> InteractionMatrix {
    const STRONG: f32 = 1.0;
    const REP: f32 = -0.9;
    const SELF: f32 = 0.0;

    let partner = |t: usize| t ^ 1; // 0↔1, 2↔3, ...

    let mut matrix = InteractionMatrix::new(n);
    for i in 0..n {
        for j in 0..n {
            let val = if j == i {
                SELF
            } else if j == partner(i) {
                STRONG
            } else {
                REP
            };
            matrix.set(i, j, val);
        }
    }
    matrix
}

/// Triad Flocks: groups of 3 attract each other.
fn triad_flocks_generator(n: usize) -> InteractionMatrix {
    const IN: f32 = 0.9;
    const OUT: f32 = -0.7;
    const SELF: f32 = 0.1;

    let group_id = |t: usize| t / 3;

    let mut matrix = InteractionMatrix::new(n);
    for i in 0..n {
        for j in 0..n {
            let val = if i == j {
                SELF
            } else if group_id(i) == group_id(j) {
                IN
            } else {
                OUT
            };
            matrix.set(i, j, val);
        }
    }
    matrix
}

/// Spiral Conveyor: creates spiral formations.
fn spiral_conveyor_generator(n: usize) -> InteractionMatrix {
    const A1: f32 = 0.7;
    const A2: f32 = 0.3;
    const R: f32 = -0.6;
    const SELF: f32 = -0.1;

    let mut matrix = InteractionMatrix::new(n);
    for i in 0..n {
        for j in 0..n {
            let val = if j == i {
                SELF
            } else if j == (i + 1) % n {
                A1
            } else if j == (i + 2) % n {
                A2
            } else {
                R
            };
            matrix.set(i, j, val);
        }
    }
    matrix
}

/// Patchwork: deterministic random-looking pattern.
fn patchwork_generator(n: usize) -> InteractionMatrix {
    const P: f32 = 0.35;
    const POS: f32 = 0.9;
    const NEG: f32 = -0.9;
    const SELF: f32 = 0.0;

    let rnd = |i: usize, j: usize| -> f32 {
        let x = ((i as i64 * 73856093) ^ (j as i64 * 19349663)) as f32;
        (x.sin() * 43_758.547).fract().abs()
    };

    let mut matrix = InteractionMatrix::new(n);
    for i in 0..n {
        for j in 0..n {
            let val = if i == j {
                SELF
            } else {
                let r = rnd(i, j);
                if r < P {
                    POS
                } else if r < 2.0 * P {
                    NEG
                } else {
                    0.0
                }
            };
            matrix.set(i, j, val);
        }
    }
    matrix
}

/// Wavefield: sinusoidal pattern.
fn wavefield_generator(n: usize) -> InteractionMatrix {
    const AMP: f32 = 0.9;
    const SELF: f32 = -0.05;

    let mut matrix = InteractionMatrix::new(n);
    for i in 0..n {
        for j in 0..n {
            let val = if i == j {
                SELF
            } else {
                let phase = 2.0 * PI * (j as f32 - i as f32) / n as f32;
                phase.sin() * AMP
            };
            matrix.set(i, j, val);
        }
    }
    matrix
}

/// Chiral Bandpass: complex wave pattern.
fn chiral_bandpass_generator(n: usize) -> InteractionMatrix {
    const SELF: f32 = -0.05;
    const A1: f32 = 0.7;
    const A2: f32 = 0.35;
    const EPS: f32 = 0.15;

    let mut matrix = InteractionMatrix::new(n);
    for i in 0..n {
        for j in 0..n {
            if i == j {
                matrix.set(i, j, SELF);
                continue;
            }
            let d = (j + n - i) % n;
            let ph = 2.0 * PI * d as f32 / n as f32;
            let val = A1 * ph.sin() + A2 * (2.0 * ph).sin() + EPS * if d > 0 { 1.0 } else { -1.0 };
            matrix.set(i, j, val);
        }
    }
    matrix
}

/// Rotating Conveyor: twisted spiral.
fn rotating_conveyor_generator(n: usize) -> InteractionMatrix {
    const SELF: f32 = -0.05;
    const AMP: f32 = 0.8;
    let twist = 2.0 * PI / (n as f32 * 1.5);

    let mut matrix = InteractionMatrix::new(n);
    for i in 0..n {
        for j in 0..n {
            if i == j {
                matrix.set(i, j, SELF);
                continue;
            }
            let ph = 2.0 * PI * (j as f32 - i as f32) / n as f32 + twist * i as f32;
            matrix.set(i, j, AMP * ph.sin());
        }
    }
    matrix
}

/// Prime Hop: pattern based on prime numbers.
fn prime_hop_generator(n: usize) -> InteractionMatrix {
    const SELF: f32 = -0.05;
    const A: f32 = 0.9;
    const R: f32 = -0.7;

    let primes = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29];
    let p = primes.iter().copied().find(|&x| x < n).unwrap_or(2);

    let circ_dist = |a: usize, b: usize| -> usize {
        let d1 = (a + n - b) % n;
        let d2 = (b + n - a) % n;
        d1.min(d2)
    };

    let mut matrix = InteractionMatrix::new(n);
    for i in 0..n {
        for j in 0..n {
            if i == j {
                matrix.set(i, j, SELF);
                continue;
            }
            let d = (j + n - i) % n;
            let val = if d.is_multiple_of(p) {
                A
            } else {
                R * (-0.7 * circ_dist(i, j) as f32).exp()
            };
            matrix.set(i, j, val);
        }
    }
    matrix
}

/// Parity Vortex: even/odd based interactions.
fn parity_vortex_generator(n: usize) -> InteractionMatrix {
    const SELF: f32 = -0.05;
    const AEO: f32 = 0.8; // Even to odd
    const AOE: f32 = 0.6; // Odd to even
    const REE: f32 = -0.4; // Even to even
    const ROO: f32 = -0.6; // Odd to odd

    let is_even = |x: usize| x.is_multiple_of(2);

    let mut matrix = InteractionMatrix::new(n);
    for i in 0..n {
        for j in 0..n {
            if i == j {
                matrix.set(i, j, SELF);
                continue;
            }
            let val = if is_even(i) && !is_even(j) {
                AEO
            } else if !is_even(i) && is_even(j) {
                AOE
            } else if is_even(i) {
                REE
            } else {
                ROO
            };
            matrix.set(i, j, val);
        }
    }
    matrix
}

/// Helical Ladder: double helix pattern.
fn helical_ladder_generator(n: usize) -> InteractionMatrix {
    const SELF: f32 = -0.05;
    const A: f32 = 0.7;
    const B: f32 = 0.4;
    let k = 2.0 * PI / n as f32;

    let mut matrix = InteractionMatrix::new(n);
    for i in 0..n {
        for j in 0..n {
            if i == j {
                matrix.set(i, j, SELF);
                continue;
            }
            let val = A * (k * (j as f32 - i as f32)).sin() + B * (k * (i as f32 + j as f32)).cos();
            matrix.set(i, j, val);
        }
    }
    matrix
}

/// Biased Wave: wave with DC offset.
fn biased_wave_generator(n: usize) -> InteractionMatrix {
    const SELF: f32 = -0.05;
    const AMP: f32 = 0.75;
    const BIAS: f32 = 0.15;

    let mut matrix = InteractionMatrix::new(n);
    for i in 0..n {
        for j in 0..n {
            if i == j {
                matrix.set(i, j, SELF);
                continue;
            }
            let ph = 2.0 * PI * (j as f32 - i as f32) / n as f32;
            matrix.set(i, j, AMP * ph.sin() + BIAS);
        }
    }
    matrix
}

/// Modular Triads: mod-3 based pattern.
fn modular_triads_generator(n: usize) -> InteractionMatrix {
    const SELF: f32 = -0.05;
    const IN: f32 = -0.2;
    const NEXT: f32 = 0.85;
    const PREV: f32 = -0.75;

    let gid = |t: usize| t % 3;

    let mut matrix = InteractionMatrix::new(n);
    for i in 0..n {
        for j in 0..n {
            if i == j {
                matrix.set(i, j, SELF);
                continue;
            }
            let di = gid(i);
            let dj = gid(j);
            let val = if di == dj {
                IN
            } else if dj == (di + 1) % 3 {
                NEXT
            } else {
                PREV
            };
            matrix.set(i, j, val);
        }
    }
    matrix
}

/// Skipped Pursuit: attraction to types K steps ahead.
fn skipped_pursuit_generator(n: usize) -> InteractionMatrix {
    const SELF: f32 = -0.05;
    const K: usize = 2;
    const A: f32 = 0.9;
    const R: f32 = -0.6;

    let circ = |d: i32| -> usize {
        let d_pos = ((d % n as i32) + n as i32) as usize % n;
        let d_neg = ((-(d % n as i32) % n as i32) + n as i32) as usize % n;
        d_pos.min(d_neg)
    };

    let mut matrix = InteractionMatrix::new(n);
    for i in 0..n {
        for j in 0..n {
            if i == j {
                matrix.set(i, j, SELF);
                continue;
            }
            let d = (j + n - i) % n;
            let near_k = circ((d as i32) - (K as i32)).min(circ((d as i32) + (K as i32)));
            let val = if d == K {
                A
            } else {
                R * (-0.8 * near_k as f32).exp()
            };
            matrix.set(i, j, val);
        }
    }
    matrix
}

/// Deterministic hash for reproducible patterns.
fn deterministic_hash32(a: usize, b: usize) -> f32 {
    let mut x = ((a as i64 * 73856093) ^ (b as i64 * 19349663)) as u32;
    x = (x ^ (x << 13)) ^ (x >> 7) ^ (x << 17);
    (x as f32) / (u32::MAX as f32)
}

/// Blue Noise Conveyor: noise-based asymmetric pattern.
fn blue_noise_conveyor_generator(n: usize) -> InteractionMatrix {
    const SELF: f32 = -0.05;
    const AMP: f32 = 0.9;
    const SKEW: f32 = 0.8;

    let mut matrix = InteractionMatrix::new(n);

    // Initial random fill
    for i in 0..n {
        for j in 0..n {
            if i == j {
                matrix.set(i, j, SELF);
            } else {
                let r = deterministic_hash32(i, j) - 0.5;
                matrix.set(i, j, AMP * r * 2.0);
            }
        }
    }

    // Apply biased anti-symmetry for drift
    for i in 0..n {
        for j in (i + 1)..n {
            let val = matrix.get(i, j);
            matrix.set(j, i, -SKEW * val);
        }
    }

    matrix
}

/// Offset Phasefield: twisted phase pattern.
fn offset_phasefield_generator(n: usize) -> InteractionMatrix {
    const SELF: f32 = -0.05;
    const AMP: f32 = 0.8;
    let k = 2.0 * PI / n as f32;
    const TW: f32 = 3.0;

    let theta = |i: usize| k * TW * i as f32;

    let mut matrix = InteractionMatrix::new(n);
    for i in 0..n {
        for j in 0..n {
            if i == j {
                matrix.set(i, j, SELF);
                continue;
            }
            let val = AMP * (theta(j) - theta(i)).sin() + 0.12 * theta(i).sin();
            matrix.set(i, j, val);
        }
    }
    matrix
}

/// Ring Road: circular distance-based pattern.
fn ring_road_generator(n: usize) -> InteractionMatrix {
    const SELF: f32 = -0.05;
    const KA: usize = 1;
    const A: f32 = 0.85;
    const R: f32 = -0.8;

    let kr = (n / 3).max(1);

    let circ = |d: i32| -> usize {
        let d_pos = ((d % n as i32) + n as i32) as usize % n;
        let d_neg = ((-(d % n as i32) % n as i32) + n as i32) as usize % n;
        d_pos.min(d_neg)
    };

    let mut matrix = InteractionMatrix::new(n);
    for i in 0..n {
        for j in 0..n {
            if i == j {
                matrix.set(i, j, SELF);
                continue;
            }
            let d = circ((j as i32) - (i as i32));
            let val = if d <= KA {
                A
            } else if d <= kr {
                0.2
            } else {
                R
            };
            matrix.set(i, j, val);
        }
    }
    matrix
}

/// Tri-Spiral: triple harmonic spiral.
fn tri_spiral_generator(n: usize) -> InteractionMatrix {
    const SELF: f32 = -0.05;
    const A1: f32 = 0.7;
    const A3: f32 = 0.55;

    let mut matrix = InteractionMatrix::new(n);
    for i in 0..n {
        for j in 0..n {
            if i == j {
                matrix.set(i, j, SELF);
                continue;
            }
            let ph = 2.0 * PI * (j as f32 - i as f32) / n as f32;
            matrix.set(i, j, A1 * ph.sin() + A3 * (3.0 * ph).sin());
        }
    }
    matrix
}

/// Vortex-Antivortex: alternating rotation pattern.
fn vortex_antivortex_generator(n: usize) -> InteractionMatrix {
    const SELF: f32 = -0.05;
    const AMP: f32 = 0.8;

    let mut matrix = InteractionMatrix::new(n);
    for i in 0..n {
        for j in 0..n {
            if i == j {
                matrix.set(i, j, SELF);
                continue;
            }
            let ph = 2.0 * PI * (j as f32 - i as f32) / n as f32;
            let parity = if (i + j) % 2 == 0 { 1.0 } else { -1.0 };
            matrix.set(i, j, parity * AMP * ph.sin());
        }
    }
    matrix
}

/// Drifted Patchwork: wave + noise masking.
fn drifted_patchwork_generator(n: usize) -> InteractionMatrix {
    const SELF: f32 = -0.05;
    const AMP: f32 = 0.75;
    const BIAS: f32 = 0.12;
    const DENS: f32 = 0.6;

    let hash = |a: usize, b: usize| -> f32 {
        let mut x = ((a as i64 * 2654435761) ^ (b as i64 * 1597334677)) as u32;
        x = (x ^ (x << 13)) ^ (x >> 7) ^ (x << 17);
        (x as f32) / (u32::MAX as f32)
    };

    let mut matrix = InteractionMatrix::new(n);
    for i in 0..n {
        for j in 0..n {
            if i == j {
                matrix.set(i, j, SELF);
                continue;
            }
            let ph = 2.0 * PI * (j as f32 - i as f32) / n as f32;
            let base = AMP * ph.sin() + BIAS;
            let mask = if hash(i, j) < DENS { 1.0 } else { 0.0 };
            matrix.set(i, j, base * mask);
        }
    }
    matrix
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_generators_produce_valid_matrices() {
        for rule_type in RuleType::all() {
            let matrix = generate_rules(*rule_type, 8);
            assert_eq!(matrix.size, 8);
            assert_eq!(matrix.data.len(), 64);
            assert!(
                matrix.validate().is_ok(),
                "Generator {:?} produced invalid matrix",
                rule_type
            );
        }
    }

    #[test]
    fn test_symmetric_generator() {
        let matrix = symmetric_generator(4);
        for i in 0..4 {
            for j in 0..4 {
                assert!(
                    (matrix.get(i, j) - matrix.get(j, i)).abs() < 0.001,
                    "Matrix not symmetric at ({i}, {j})"
                );
            }
        }
    }

    #[test]
    fn test_snake_generator() {
        let matrix = snake_generator(4);
        // Check that each type has self-attraction
        for i in 0..4 {
            assert!((matrix.get(i, i) - 1.0).abs() < 0.001);
            // And weak attraction to the next type
            assert!((matrix.get(i, (i + 1) % 4) - 0.2).abs() < 0.001);
        }
    }

    #[test]
    fn test_empty_matrix() {
        let matrix = generate_rules(RuleType::Random, 0);
        assert_eq!(matrix.size, 0);
        assert!(matrix.data.is_empty());
    }
}

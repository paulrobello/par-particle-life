//! Position generators for spawning particles.
//!
//! This module provides 27 different spawn patterns for particles,
//! from simple random distributions to complex geometric arrangements.

use rand::Rng;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

use crate::simulation::Particle;

/// Configuration for spawning particles.
#[derive(Debug, Clone)]
pub struct SpawnConfig {
    pub num_particles: usize,
    pub num_types: usize,
    pub width: f32,
    pub height: f32,
}

/// Types of position patterns available.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum PositionPattern {
    #[default]
    Random = 0,
    Disk = 1,
    Ring = 2,
    Rings = 3,
    Spiral = 4,
    Line = 5,
    RainbowDisk = 6,
    RainbowRing = 7,
    RainbowRings = 8,
    RainbowSpiral = 9,
    RainbowLine = 10,
    Stripes = 11,
    Border = 12,
    Grid = 13,
    WavyBands = 14,
    SimpleFlower = 15,
    ChromaticFlower = 16,
    YinYang = 17,
    TwinCrescents = 18,
    TwinSpirals = 19,
    SpiralArms = 20,
    PolarMaze = 21,
    ChaoticBands = 22,
    RadiantFans = 23,
    SoftClusters = 24,
    LinkedClusters = 25,
    OrbitalBelts = 26,
    BraidedBelts = 27,
}

impl PositionPattern {
    /// Get all available patterns.
    pub fn all() -> &'static [PositionPattern] {
        use PositionPattern::*;
        &[
            Random,
            Disk,
            Ring,
            Rings,
            Spiral,
            Line,
            RainbowDisk,
            RainbowRing,
            RainbowRings,
            RainbowSpiral,
            RainbowLine,
            Stripes,
            Border,
            Grid,
            WavyBands,
            SimpleFlower,
            ChromaticFlower,
            YinYang,
            TwinCrescents,
            TwinSpirals,
            SpiralArms,
            PolarMaze,
            ChaoticBands,
            RadiantFans,
            SoftClusters,
            LinkedClusters,
            OrbitalBelts,
            BraidedBelts,
        ]
    }

    /// Get the display name for this pattern.
    pub fn display_name(&self) -> &'static str {
        match self {
            PositionPattern::Random => "Random",
            PositionPattern::Disk => "Disk",
            PositionPattern::Ring => "Ring",
            PositionPattern::Rings => "Rings",
            PositionPattern::Spiral => "Spiral",
            PositionPattern::Line => "Line",
            PositionPattern::RainbowDisk => "Rainbow Disk",
            PositionPattern::RainbowRing => "Rainbow Ring",
            PositionPattern::RainbowRings => "Rainbow Rings",
            PositionPattern::RainbowSpiral => "Rainbow Spiral",
            PositionPattern::RainbowLine => "Rainbow Line",
            PositionPattern::Stripes => "Stripes",
            PositionPattern::Border => "Border",
            PositionPattern::Grid => "Grid",
            PositionPattern::WavyBands => "Wavy Bands",
            PositionPattern::SimpleFlower => "Simple Flower",
            PositionPattern::ChromaticFlower => "Chromatic Flower",
            PositionPattern::YinYang => "Yin–Yang",
            PositionPattern::TwinCrescents => "Twin Crescents",
            PositionPattern::TwinSpirals => "Twin Spirals",
            PositionPattern::SpiralArms => "Spiral Arms",
            PositionPattern::PolarMaze => "Polar Maze",
            PositionPattern::ChaoticBands => "Chaotic Bands",
            PositionPattern::RadiantFans => "Radiant Fans",
            PositionPattern::SoftClusters => "Soft Clusters",
            PositionPattern::LinkedClusters => "Linked Clusters",
            PositionPattern::OrbitalBelts => "Orbital Belts",
            PositionPattern::BraidedBelts => "Braided Belts",
        }
    }

    /// Get the category for this pattern.
    pub fn category(&self) -> &'static str {
        match self {
            PositionPattern::Random => "Default",
            PositionPattern::Disk
            | PositionPattern::Ring
            | PositionPattern::Rings
            | PositionPattern::Spiral
            | PositionPattern::Line => "Classic",
            PositionPattern::RainbowDisk
            | PositionPattern::RainbowRing
            | PositionPattern::RainbowRings
            | PositionPattern::RainbowSpiral
            | PositionPattern::RainbowLine => "Chromatic",
            PositionPattern::Stripes
            | PositionPattern::Border
            | PositionPattern::Grid
            | PositionPattern::WavyBands
            | PositionPattern::SimpleFlower
            | PositionPattern::ChromaticFlower
            | PositionPattern::YinYang
            | PositionPattern::TwinCrescents
            | PositionPattern::TwinSpirals
            | PositionPattern::SpiralArms
            | PositionPattern::PolarMaze => "Geometric",
            _ => "Dynamic",
        }
    }

    /// Get the required number of types for this pattern, if fixed.
    /// Returns `Some(n)` if the pattern requires exactly n types to look correct,
    /// or `None` if the pattern adapts to any number of types.
    pub fn required_types(&self) -> Option<usize> {
        match self {
            // These patterns use exactly 2 types (0 and 1)
            PositionPattern::YinYang
            | PositionPattern::TwinCrescents
            | PositionPattern::TwinSpirals => Some(2),
            // All other patterns adapt to the configured num_types
            _ => None,
        }
    }
}

/// Generate particles using the specified pattern.
pub fn generate_positions(pattern: PositionPattern, config: &SpawnConfig) -> Vec<Particle> {
    if config.num_particles == 0 || config.num_types == 0 {
        return Vec::new();
    }

    match pattern {
        PositionPattern::Random => random_generator(config),
        PositionPattern::Disk => disk_generator(config),
        PositionPattern::Ring => ring_generator(config),
        PositionPattern::Rings => rings_generator(config),
        PositionPattern::Spiral => spiral_generator(config),
        PositionPattern::Line => line_generator(config),
        PositionPattern::RainbowDisk => rainbow_disk_generator(config),
        PositionPattern::RainbowRing => rainbow_ring_generator(config),
        PositionPattern::RainbowRings => rainbow_rings_generator(config),
        PositionPattern::RainbowSpiral => rainbow_spiral_generator(config),
        PositionPattern::RainbowLine => rainbow_line_generator(config),
        PositionPattern::Stripes => stripes_generator(config),
        PositionPattern::Border => border_generator(config),
        PositionPattern::Grid => grid_generator(config),
        PositionPattern::WavyBands => wavy_bands_generator(config),
        PositionPattern::SimpleFlower => simple_flower_generator(config),
        PositionPattern::ChromaticFlower => chromatic_flower_generator(config),
        PositionPattern::YinYang => yin_yang_generator(config),
        PositionPattern::TwinCrescents => twin_crescents_generator(config),
        PositionPattern::TwinSpirals => twin_spirals_generator(config),
        PositionPattern::SpiralArms => spiral_arms_generator(config),
        PositionPattern::PolarMaze => polar_maze_generator(config),
        PositionPattern::ChaoticBands => chaotic_bands_generator(config),
        PositionPattern::RadiantFans => radiant_fans_generator(config),
        PositionPattern::SoftClusters => soft_clusters_generator(config),
        PositionPattern::LinkedClusters => linked_clusters_generator(config),
        PositionPattern::OrbitalBelts => orbital_belts_generator(config),
        PositionPattern::BraidedBelts => braided_belts_generator(config),
    }
}

// === Helper Macros and Functions ===

const TAU: f32 = 2.0 * PI;

fn create_particle(x: f32, y: f32, particle_type: u32) -> Particle {
    Particle::new(x, y, particle_type)
}

// === Generator Implementations ===

fn random_generator(config: &SpawnConfig) -> Vec<Particle> {
    let mut rng = rand::rng();
    let mut particles = Vec::with_capacity(config.num_particles);
    let mut t = 0u32;

    for _ in 0..config.num_particles {
        let x = rng.random::<f32>() * config.width;
        let y = rng.random::<f32>() * config.height;
        particles.push(create_particle(x, y, t));
        t = (t + 1) % config.num_types as u32;
    }

    particles
}

fn disk_generator(config: &SpawnConfig) -> Vec<Particle> {
    let mut rng = rand::rng();
    let mut particles = Vec::with_capacity(config.num_particles);
    let cx = config.width * 0.5;
    let cy = config.height * 0.5;
    let r = 0.46 * config.width.min(config.height);
    let mut t = 0u32;

    for _ in 0..config.num_particles {
        let th = rng.random::<f32>() * TAU;
        let rr = r * rng.random::<f32>().sqrt();
        let x = cx + rr * th.cos();
        let y = cy + rr * th.sin();
        particles.push(create_particle(x, y, t));
        t = (t + 1) % config.num_types as u32;
    }

    particles
}

fn ring_generator(config: &SpawnConfig) -> Vec<Particle> {
    let mut rng = rand::rng();
    let mut particles = Vec::with_capacity(config.num_particles);
    let cx = config.width * 0.5;
    let cy = config.height * 0.5;
    let r = 0.46 * config.width.min(config.height);
    let thick = r * 0.2;
    let rot = rng.random::<f32>() * TAU;
    let dth = TAU / config.num_particles.max(1) as f32;
    let mut t = 0u32;

    for i in 0..config.num_particles {
        let th = rot + i as f32 * dth;
        let rr = r - rng.random::<f32>() * thick;
        let x = cx + rr * th.cos();
        let y = cy + rr * th.sin();
        particles.push(create_particle(x, y, t));
        t = (t + 1) % config.num_types as u32;
    }

    particles
}

fn rings_generator(config: &SpawnConfig) -> Vec<Particle> {
    let mut rng = rand::rng();
    let mut particles = Vec::with_capacity(config.num_particles);
    let cx = config.width * 0.5;
    let cy = config.height * 0.5;
    let max_r = 0.46 * config.width.min(config.height);
    let num_rings = rng.random_range(2..=8);
    let particles_per_ring = config.num_particles / num_rings;
    let mut t = 0u32;

    for ring in 0..num_rings {
        let f = if num_rings == 1 {
            0.5
        } else {
            0.23 + 0.69 * ring as f32 / (num_rings - 1) as f32
        };
        let r = f * max_r;
        let count = if ring < config.num_particles % num_rings {
            particles_per_ring + 1
        } else {
            particles_per_ring
        };

        for j in 0..count {
            let th = TAU * j as f32 / count as f32 + rng.random::<f32>() * 0.1;
            let rr = r + (rng.random::<f32>() - 0.5) * 0.04 * max_r;
            let x = cx + rr * th.cos();
            let y = cy + rr * th.sin();
            particles.push(create_particle(x, y, t));
            t = (t + 1) % config.num_types as u32;
        }
    }

    particles
}

fn spiral_generator(config: &SpawnConfig) -> Vec<Particle> {
    let mut rng = rand::rng();
    let mut particles = Vec::with_capacity(config.num_particles);
    let cx = config.width * 0.5;
    let cy = config.height * 0.5;
    let r = 0.46 * config.width.min(config.height);
    let thick = 0.0175 * config.width.min(config.height);
    let turns = 1.2 + rng.random::<f32>() * 2.4;
    let rot = rng.random::<f32>() * TAU;
    let n1 = (config.num_particles - 1).max(1) as f32;
    let mut t = 0u32;

    for i in 0..config.num_particles {
        let u = i as f32 / n1;
        let th = rot + u * turns * TAU;
        let rr = (u * r + (rng.random::<f32>() - 0.5) * 2.0 * thick).max(0.0);
        let x = cx + rr * th.cos();
        let y = cy + rr * th.sin();
        particles.push(create_particle(x, y, t));
        t = (t + 1) % config.num_types as u32;
    }

    particles
}

fn line_generator(config: &SpawnConfig) -> Vec<Particle> {
    let mut rng = rand::rng();
    let mut particles = Vec::with_capacity(config.num_particles);
    let l = config.width * 0.92;
    let thick = config.height * 0.10;
    let cx = config.width * 0.5;
    let cy = config.height * 0.5;
    let x_start = cx - l * 0.5;
    let step = if config.num_particles > 1 {
        l / (config.num_particles - 1) as f32
    } else {
        0.0
    };
    let mut t = 0u32;

    for i in 0..config.num_particles {
        let x = x_start + step * i as f32;
        let y = cy + (rng.random::<f32>() - 0.5) * thick;
        particles.push(create_particle(x, y, t));
        t = (t + 1) % config.num_types as u32;
    }

    particles
}

fn rainbow_disk_generator(config: &SpawnConfig) -> Vec<Particle> {
    let mut rng = rand::rng();
    let mut particles = Vec::with_capacity(config.num_particles);
    let cx = config.width * 0.5;
    let cy = config.height * 0.5;
    let r = 0.46 * config.width.min(config.height);
    let rot = rng.random::<f32>() * TAU;
    let sector = TAU / config.num_types.max(1) as f32;
    let per_type = config.num_particles / config.num_types;
    let mut remainder = config.num_particles % config.num_types;

    for t in 0..config.num_types {
        let count = per_type
            + if remainder > 0 {
                remainder -= 1;
                1
            } else {
                0
            };
        let th0 = rot + t as f32 * sector;

        for j in 0..count {
            let th = th0 + sector * j as f32 / count as f32 + rng.random::<f32>() * 0.1;
            let rr = r * rng.random::<f32>().sqrt();
            let x = cx + rr * th.cos();
            let y = cy + rr * th.sin();
            particles.push(create_particle(x, y, t as u32));
        }
    }

    particles
}

fn rainbow_ring_generator(config: &SpawnConfig) -> Vec<Particle> {
    let mut rng = rand::rng();
    let mut particles = Vec::with_capacity(config.num_particles);
    let cx = config.width * 0.5;
    let cy = config.height * 0.5;
    let r = 0.46 * config.width.min(config.height);
    let thick = r * 0.2;
    let rot = rng.random::<f32>() * TAU;
    let sector = TAU / config.num_types.max(1) as f32;
    let per_type = config.num_particles / config.num_types;
    let mut remainder = config.num_particles % config.num_types;

    for t in 0..config.num_types {
        let count = per_type
            + if remainder > 0 {
                remainder -= 1;
                1
            } else {
                0
            };
        let th0 = rot + t as f32 * sector;

        for j in 0..count {
            let th = th0 + sector * j as f32 / count as f32;
            let rr = r - rng.random::<f32>() * thick;
            let x = cx + rr * th.cos();
            let y = cy + rr * th.sin();
            particles.push(create_particle(x, y, t as u32));
        }
    }

    particles
}

fn rainbow_rings_generator(config: &SpawnConfig) -> Vec<Particle> {
    let mut rng = rand::rng();
    let mut particles = Vec::with_capacity(config.num_particles);
    let cx = config.width * 0.5;
    let cy = config.height * 0.5;
    let max_r = 0.46 * config.width.min(config.height);
    let thick = 0.02 * max_r;
    let per_ring = config.num_particles / config.num_types;
    let mut remainder = config.num_particles % config.num_types;

    for ring in 0..config.num_types {
        let f = if config.num_types == 1 {
            0.5
        } else {
            0.23 + 0.69 * ring as f32 / (config.num_types - 1) as f32
        };
        let r = f * max_r;
        let count = per_ring
            + if remainder > 0 {
                remainder -= 1;
                1
            } else {
                0
            };

        for j in 0..count {
            let th = TAU * j as f32 / count as f32 + rng.random::<f32>() * 0.1;
            let rr = r + (rng.random::<f32>() - 0.5) * 2.0 * thick;
            let x = cx + rr * th.cos();
            let y = cy + rr * th.sin();
            particles.push(create_particle(x, y, ring as u32));
        }
    }

    particles
}

fn rainbow_spiral_generator(config: &SpawnConfig) -> Vec<Particle> {
    let mut rng = rand::rng();
    let mut particles = Vec::with_capacity(config.num_particles);
    let cx = config.width * 0.5;
    let cy = config.height * 0.5;
    let r = 0.46 * config.width.min(config.height);
    let thick = 0.0175 * config.width.min(config.height);
    let turns = 1.2 + rng.random::<f32>() * 2.4;
    let rot = rng.random::<f32>() * TAU;
    let n1 = (config.num_particles - 1).max(1) as f32;

    for i in 0..config.num_particles {
        let u = i as f32 / n1;
        let th = rot + u * turns * TAU;
        let rr = (u * r + (rng.random::<f32>() - 0.5) * 2.0 * thick).max(0.0);
        let x = cx + rr * th.cos();
        let y = cy + rr * th.sin();
        let t = ((u * config.num_types as f32) as usize).min(config.num_types - 1) as u32;
        particles.push(create_particle(x, y, t));
    }

    particles
}

fn rainbow_line_generator(config: &SpawnConfig) -> Vec<Particle> {
    let mut rng = rand::rng();
    let mut particles = Vec::with_capacity(config.num_particles);
    let l = config.width * 0.92;
    let thick = config.height * 0.10;
    let cx = config.width * 0.5;
    let cy = config.height * 0.5;
    let x_start = cx - l * 0.5;
    let seg_w = l / config.num_types as f32;
    let per_seg = config.num_particles / config.num_types;
    let mut remainder = config.num_particles % config.num_types;

    for t in 0..config.num_types {
        let count = per_seg
            + if remainder > 0 {
                remainder -= 1;
                1
            } else {
                0
            };
        let x0 = x_start + t as f32 * seg_w;

        for j in 0..count {
            let x = x0 + seg_w * (j as f32 + rng.random::<f32>()) / count as f32;
            let y = cy + (rng.random::<f32>() - 0.5) * thick;
            particles.push(create_particle(x, y, t as u32));
        }
    }

    particles
}

fn stripes_generator(config: &SpawnConfig) -> Vec<Particle> {
    let mut rng = rand::rng();
    let mut particles = Vec::with_capacity(config.num_particles);
    let vertical = rng.random::<bool>();
    let per_type = config.num_particles / config.num_types;
    let mut remainder = config.num_particles % config.num_types;

    for t in 0..config.num_types {
        let count = per_type
            + if remainder > 0 {
                remainder -= 1;
                1
            } else {
                0
            };

        for _ in 0..count {
            let (x, y) = if vertical {
                let seg = config.width / config.num_types as f32;
                let x = t as f32 * seg + rng.random::<f32>() * seg;
                let y = rng.random::<f32>() * config.height;
                (x, y)
            } else {
                let seg = config.height / config.num_types as f32;
                let x = rng.random::<f32>() * config.width;
                let y = t as f32 * seg + rng.random::<f32>() * seg;
                (x, y)
            };
            particles.push(create_particle(x, y, t as u32));
        }
    }

    particles
}

fn border_generator(config: &SpawnConfig) -> Vec<Particle> {
    let mut particles = Vec::with_capacity(config.num_particles);
    let inset = 1.0;
    let w = (config.width - 2.0 * inset).max(0.0);
    let h = (config.height - 2.0 * inset).max(0.0);
    let p = (w + h) * 2.0;
    if p == 0.0 {
        return particles;
    }

    let step = p / config.num_particles as f32;
    let mut s = 0.0;
    let mut t = 0u32;

    for _ in 0..config.num_particles {
        let (x, y) = if s < w {
            (inset + s, inset)
        } else if s < w + h {
            (inset + w, inset + (s - w))
        } else if s < 2.0 * w + h {
            (inset + (2.0 * w + h - s), inset + h)
        } else {
            (inset, inset + (p - s))
        };
        particles.push(create_particle(x, y, t));
        t = (t + 1) % config.num_types as u32;
        s += step;
    }

    particles
}

fn grid_generator(config: &SpawnConfig) -> Vec<Particle> {
    let mut particles = Vec::with_capacity(config.num_particles);
    let cols = (config.num_particles as f32).sqrt().ceil() as usize;
    let rows = config.num_particles.div_ceil(cols);
    let dx = config.width / cols as f32;
    let dy = config.height / rows as f32;
    let mut t = 0u32;
    let mut i = 0;

    'outer: for r in 0..rows {
        let y = (r as f32 + 0.5) * dy;
        for c in 0..cols {
            if i >= config.num_particles {
                break 'outer;
            }
            let x = (c as f32 + 0.5) * dx;
            particles.push(create_particle(x, y, t));
            t = (t + 1) % config.num_types as u32;
            i += 1;
        }
    }

    particles
}

fn wavy_bands_generator(config: &SpawnConfig) -> Vec<Particle> {
    let mut rng = rand::rng();
    let mut particles = Vec::with_capacity(config.num_particles);
    let seg_h = config.height / config.num_types as f32;
    let amp = 0.06 * config.height;
    let kx = (TAU / config.width) * (1.0 + (config.num_types % 3) as f32);
    let base_phase = rng.random::<f32>() * TAU;
    let per_type = config.num_particles / config.num_types;
    let mut remainder = config.num_particles % config.num_types;

    for t in 0..config.num_types {
        let count = per_type
            + if remainder > 0 {
                remainder -= 1;
                1
            } else {
                0
            };
        let y0 = (t as f32 + 0.5) * seg_h;
        let phase = base_phase + t as f32 * 0.7;

        for _ in 0..count {
            let x = rng.random::<f32>() * config.width;
            let y = (y0
                + amp * (kx * x + phase).sin()
                + (rng.random::<f32>() - 0.5) * 0.04 * config.height)
                .clamp(0.0, config.height);
            particles.push(create_particle(x, y, t as u32));
        }
    }

    particles
}

fn simple_flower_generator(config: &SpawnConfig) -> Vec<Particle> {
    let mut rng = rand::rng();
    let mut particles = Vec::with_capacity(config.num_particles);
    let petals = rng.random_range(2..=8);
    let phase = rng.random::<f32>() * TAU;
    let cx = config.width * 0.5;
    let cy = config.height * 0.5;
    let r = 0.46 * config.width.min(config.height);
    let jitter = 0.02 * r;
    let mut t = 0u32;

    for i in 0..config.num_particles {
        let u = (i as f32 + rng.random::<f32>()) / config.num_particles as f32;
        let th = TAU * u + phase;
        let r_base = (petals as f32 * th).cos().abs() * r;
        let rr = r_base + (rng.random::<f32>() - 0.5) * 2.0 * jitter;
        let x = cx + rr * th.cos();
        let y = cy + rr * th.sin();
        particles.push(create_particle(x, y, t));
        t = (t + 1) % config.num_types as u32;
    }

    particles
}

fn chromatic_flower_generator(config: &SpawnConfig) -> Vec<Particle> {
    // Simplified version - similar to simple_flower but with chromatic assignment
    let mut rng = rand::rng();
    let mut particles = Vec::with_capacity(config.num_particles);
    let petals = rng.random_range(2..=7);
    let phase = rng.random::<f32>() * TAU;
    let cx = config.width * 0.5;
    let cy = config.height * 0.5;
    let r = 0.46 * config.width.min(config.height);
    let jitter = 0.006 * config.width.min(config.height);
    let per_type = config.num_particles / config.num_types;
    let mut remainder = config.num_particles % config.num_types;

    for t in 0..config.num_types {
        let count = per_type
            + if remainder > 0 {
                remainder -= 1;
                1
            } else {
                0
            };
        let scale = 0.62 + rng.random::<f32>() * 0.38;
        let r_scaled = r * scale;
        let type_phase = rng.random::<f32>() * TAU;

        for j in 0..count {
            let th = type_phase + TAU * j as f32 / count as f32;
            let g = (petals as f32 * th + phase).cos().abs();
            let rr = r_scaled * g.powf(1.35);
            let x = cx + rr * th.cos() + (rng.random::<f32>() - 0.5) * 2.0 * jitter;
            let y = cy + rr * th.sin() + (rng.random::<f32>() - 0.5) * 2.0 * jitter;
            particles.push(create_particle(x, y, t as u32));
        }
    }

    particles
}

fn yin_yang_generator(config: &SpawnConfig) -> Vec<Particle> {
    let mut rng = rand::rng();
    let mut particles = Vec::with_capacity(config.num_particles);
    let cx = config.width * 0.5;
    let cy = config.height * 0.5;
    let r = 0.46 * config.width.min(config.height);
    let small_r = r / 2.0;
    let eye_r = r / 6.0;

    let half = config.num_particles / 2;

    // Top half (one type group)
    for _ in 0..half {
        loop {
            let th = rng.random::<f32>() * TAU;
            let rr = r * rng.random::<f32>().sqrt();
            let x = cx + rr * th.cos();
            let y = cy + rr * th.sin();

            // Check if in top half or bottom eye
            let in_top_main = y <= cy;
            let in_bottom_eye = ((x - cx).powi(2) + (y - cy - small_r).powi(2)) <= eye_r.powi(2);

            if in_top_main || in_bottom_eye {
                particles.push(create_particle(x, y, 0));
                break;
            }
        }
    }

    // Bottom half
    for _ in half..config.num_particles {
        loop {
            let th = rng.random::<f32>() * TAU;
            let rr = r * rng.random::<f32>().sqrt();
            let x = cx + rr * th.cos();
            let y = cy + rr * th.sin();

            let in_bottom_main = y > cy;
            let in_top_eye = ((x - cx).powi(2) + (y - cy + small_r).powi(2)) <= eye_r.powi(2);

            if in_bottom_main || in_top_eye {
                let t = if config.num_types > 1 { 1 } else { 0 };
                particles.push(create_particle(x, y, t));
                break;
            }
        }
    }

    particles
}

fn twin_crescents_generator(config: &SpawnConfig) -> Vec<Particle> {
    let mut rng = rand::rng();
    let mut particles = Vec::with_capacity(config.num_particles);
    let m = config.width.min(config.height);
    let cx = config.width * 0.5;
    let cy = config.height * 0.5;
    let r = 0.36 * m;
    let sep = 0.28 * m;
    let c1x = cx - sep * 0.5;
    let c2x = cx + sep * 0.5;
    let r2 = r * r;

    let half = config.num_particles / 2;

    // Left crescent
    for _ in 0..half {
        loop {
            let th = rng.random::<f32>() * TAU;
            let rr = r * rng.random::<f32>().sqrt();
            let x = c1x + rr * th.cos();
            let y = cy + rr * th.sin();

            // Not in right disk
            if (x - c2x).powi(2) + (y - cy).powi(2) >= r2 {
                particles.push(create_particle(x, y, 0));
                break;
            }
        }
    }

    // Right crescent
    for _ in half..config.num_particles {
        loop {
            let th = rng.random::<f32>() * TAU;
            let rr = r * rng.random::<f32>().sqrt();
            let x = c2x + rr * th.cos();
            let y = cy + rr * th.sin();

            // Not in left disk
            if (x - c1x).powi(2) + (y - cy).powi(2) >= r2 {
                let t = if config.num_types > 1 { 1 } else { 0 };
                particles.push(create_particle(x, y, t));
                break;
            }
        }
    }

    particles
}

fn twin_spirals_generator(config: &SpawnConfig) -> Vec<Particle> {
    let mut rng = rand::rng();
    let mut particles = Vec::with_capacity(config.num_particles);
    let cx = config.width * 0.5;
    let cy = config.height * 0.5;
    let r = 0.46 * config.width.min(config.height);
    let thick = 0.015 * config.width.min(config.height);
    let turns = 1.5 + rng.random::<f32>() * 1.5;
    let rot = rng.random::<f32>() * TAU;

    let half = config.num_particles / 2;

    for arm in 0..2 {
        let arm_rot = rot + arm as f32 * PI;
        let count = if arm == 0 {
            half
        } else {
            config.num_particles - half
        };
        let n1 = (count - 1).max(1) as f32;
        let arm_type = if config.num_types > 1 { arm as u32 } else { 0 };

        for i in 0..count {
            let u = i as f32 / n1;
            let th = arm_rot + u * turns * TAU;
            let rr = (u * r + (rng.random::<f32>() - 0.5) * 2.0 * thick).max(0.0);
            let x = cx + rr * th.cos();
            let y = cy + rr * th.sin();
            particles.push(create_particle(x, y, arm_type % config.num_types as u32));
        }
    }

    particles
}

fn spiral_arms_generator(config: &SpawnConfig) -> Vec<Particle> {
    let mut rng = rand::rng();
    let mut particles = Vec::with_capacity(config.num_particles);
    let cx = config.width * 0.5;
    let cy = config.height * 0.5;
    let max_r = 0.46 * config.width.min(config.height);
    let turns = 2.5;
    let thick = (0.07 / config.num_types as f32).min(0.02) * config.width.min(config.height);
    let per_arm = config.num_particles / config.num_types;
    let mut remainder = config.num_particles % config.num_types;

    for arm in 0..config.num_types {
        let count = per_arm
            + if remainder > 0 {
                remainder -= 1;
                1
            } else {
                0
            };
        let arm_rot = arm as f32 * TAU / config.num_types as f32;
        let n1 = (count - 1).max(1) as f32;

        for i in 0..count {
            let u = i as f32 / n1;
            let th = arm_rot + u * turns * TAU;
            let rr = (u * max_r + (rng.random::<f32>() - 0.5) * 2.0 * thick).max(0.0);
            let x = cx + rr * th.cos();
            let y = cy + rr * th.sin();
            particles.push(create_particle(x, y, arm as u32));
        }
    }

    particles
}

fn polar_maze_generator(config: &SpawnConfig) -> Vec<Particle> {
    let mut rng = rand::rng();
    let mut particles = Vec::with_capacity(config.num_particles);
    let cx = config.width * 0.5;
    let cy = config.height * 0.5;
    let m = config.width.min(config.height);
    let r_min = 0.12 * m;
    let r_max = 0.46 * m;
    let layers = 6;
    let sectors = 18;
    let dr = (r_max - r_min) / layers as f32;
    let dth = TAU / sectors as f32;
    let thick = 0.012 * m;
    let per_layer = config.num_particles / layers;
    let mut remainder = config.num_particles % layers;

    for l in 0..layers {
        let count = per_layer
            + if remainder > 0 {
                remainder -= 1;
                1
            } else {
                0
            };
        let r = r_min + (l as f32 + 0.5) * dr;
        let t = l % config.num_types;

        for _ in 0..count {
            let s = rng.random_range(0..sectors);
            let ang = (s as f32 + rng.random::<f32>()) * dth;
            let rr = r + (rng.random::<f32>() - 0.5) * 2.0 * thick;
            let x = cx + rr * ang.cos();
            let y = cy + rr * ang.sin();
            particles.push(create_particle(x, y, t as u32));
        }
    }

    particles
}

fn chaotic_bands_generator(config: &SpawnConfig) -> Vec<Particle> {
    // Simplified version using random bands
    let mut rng = rand::rng();
    let mut particles = Vec::with_capacity(config.num_particles);
    let lanes = rng.random_range(3..=10).min(config.num_types);
    let per_lane = config.num_particles / lanes;
    let mut remainder = config.num_particles % lanes;

    for lane in 0..lanes {
        let count = per_lane
            + if remainder > 0 {
                remainder -= 1;
                1
            } else {
                0
            };
        let theta = rng.random::<f32>() * TAU;
        let ux = theta.cos();
        let uy = theta.sin();
        let start_x = rng.random::<f32>() * config.width;
        let start_y = rng.random::<f32>() * config.height;
        let amp = rng.random::<f32>() * 0.06 * config.height;
        let t = lane % config.num_types;

        for i in 0..count {
            let u = i as f32 / count.max(1) as f32;
            let along = config.width.max(config.height) * u;
            let offset = amp * (10.0 * u).sin();
            let x = (start_x + along * ux - offset * uy).clamp(0.0, config.width);
            let y = (start_y + along * uy + offset * ux).clamp(0.0, config.height);
            particles.push(create_particle(x, y, t as u32));
        }
    }

    particles
}

fn radiant_fans_generator(config: &SpawnConfig) -> Vec<Particle> {
    let mut rng = rand::rng();
    let mut particles = Vec::with_capacity(config.num_particles);
    let fans = config.num_types.clamp(3, 10);
    let cx = config.width * 0.5;
    let cy = config.height * 0.5;
    let r = 0.46 * config.width.min(config.height);
    let spread = 0.22 * PI;
    let per_fan = config.num_particles / fans;
    let mut remainder = config.num_particles % fans;

    for f in 0..fans {
        let count = per_fan
            + if remainder > 0 {
                remainder -= 1;
                1
            } else {
                0
            };
        let th0 = (f as f32 / fans as f32) * TAU + rng.random::<f32>() * 0.2;
        let t = f % config.num_types;

        for _ in 0..count {
            let rr = r * rng.random::<f32>().powf(0.56);
            let th = th0 + (rng.random::<f32>() - 0.5) * 2.0 * spread;
            let x = cx + rr * th.cos();
            let y = cy + rr * th.sin();
            particles.push(create_particle(x, y, t as u32));
        }
    }

    particles
}

fn soft_clusters_generator(config: &SpawnConfig) -> Vec<Particle> {
    let mut rng = rand::rng();
    let mut particles = Vec::with_capacity(config.num_particles);
    let clusters = rng.random_range(2..=6).min(config.num_types).max(2);
    let m = config.width.min(config.height);
    let margin = 0.18;
    let r_min = 0.14 * m;
    let r_max = 0.20 * m;
    let per_cluster = config.num_particles / clusters;
    let mut remainder = config.num_particles % clusters;

    for c in 0..clusters {
        let count = per_cluster
            + if remainder > 0 {
                remainder -= 1;
                1
            } else {
                0
            };
        let r = r_min + rng.random::<f32>() * (r_max - r_min);
        let cx = margin * config.width + rng.random::<f32>() * config.width * (1.0 - 2.0 * margin);
        let cy =
            margin * config.height + rng.random::<f32>() * config.height * (1.0 - 2.0 * margin);
        let t = c % config.num_types;

        for _ in 0..count {
            let th = rng.random::<f32>() * TAU;
            let rr = r * rng.random::<f32>().sqrt();
            let x = cx + rr * th.cos();
            let y = cy + rr * th.sin();
            particles.push(create_particle(x, y, t as u32));
        }
    }

    particles
}

fn linked_clusters_generator(config: &SpawnConfig) -> Vec<Particle> {
    // Simplified - just clusters for now
    soft_clusters_generator(config)
}

fn orbital_belts_generator(config: &SpawnConfig) -> Vec<Particle> {
    let mut rng = rand::rng();
    let mut particles = Vec::with_capacity(config.num_particles);
    let cx = config.width * 0.5;
    let cy = config.height * 0.5;
    let r = 0.46 * config.width.min(config.height);
    let ecc = 0.35;
    let thick = 0.02 * r;
    let belts = rng.random_range(4..=8).min(config.num_types);
    let per_belt = config.num_particles / belts;
    let mut remainder = config.num_particles % belts;

    for b in 0..belts {
        let count = per_belt
            + if remainder > 0 {
                remainder -= 1;
                1
            } else {
                0
            };
        let a = r * (0.25 + 0.7 * b as f32 / (belts - 1).max(1) as f32);
        let e = ecc * (0.6 + 0.8 * rng.random::<f32>());
        let c_offset = a * e;
        let rot = rng.random::<f32>() * TAU;
        let b_axis = a * (1.0 - e * e).sqrt();
        let t = b % config.num_types;

        for i in 0..count {
            let th = TAU * i as f32 / count as f32 + rng.random::<f32>() * 0.1;
            let rr = (rng.random::<f32>() - 0.5) * 2.0 * thick;
            let ex = (a + rr) * th.cos();
            let ey = (b_axis + rr) * th.sin();
            let x = (cx + c_offset * rot.cos() + ex * rot.cos() - ey * rot.sin())
                .clamp(0.0, config.width);
            let y = (cy + c_offset * rot.sin() + ex * rot.sin() + ey * rot.cos())
                .clamp(0.0, config.height);
            particles.push(create_particle(x, y, t as u32));
        }
    }

    particles
}

fn braided_belts_generator(config: &SpawnConfig) -> Vec<Particle> {
    let mut rng = rand::rng();
    let mut particles = Vec::with_capacity(config.num_particles);
    let cx = config.width * 0.5;
    let cy = config.height * 0.5;
    let r = 0.46 * config.width.min(config.height);
    let ecc = 0.36;
    let wav_amp = 0.06 * r;
    let wav_freq = 4.0;
    let thick = 0.018 * r;
    let belts = rng.random_range(2..=5);
    let per_belt = config.num_particles / belts;
    let mut remainder = config.num_particles % belts;

    for b in 0..belts {
        let count = per_belt
            + if remainder > 0 {
                remainder -= 1;
                1
            } else {
                0
            };
        let a = r * (0.55 + 0.35 * b as f32 / (belts - 1).max(1) as f32);
        let c_offset = a * ecc;
        let rot = rng.random::<f32>() * TAU;
        let b_axis = a * (1.0 - ecc * ecc).sqrt();
        let t = b % config.num_types;

        for i in 0..count {
            let th = TAU * i as f32 / count as f32;
            let w = wav_amp * (wav_freq * th + b as f32 * PI).sin();
            let rr = (rng.random::<f32>() - 0.5) * 2.0 * thick;
            let ex = (a + w + rr) * th.cos();
            let ey = (b_axis + rr) * th.sin();
            let x = (cx + c_offset * rot.cos() + ex * rot.cos() - ey * rot.sin())
                .clamp(0.0, config.width);
            let y = (cy + c_offset * rot.sin() + ex * rot.sin() + ey * rot.cos())
                .clamp(0.0, config.height);
            particles.push(create_particle(x, y, t as u32));
        }
    }

    particles
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> SpawnConfig {
        SpawnConfig {
            num_particles: 100,
            num_types: 4,
            width: 800.0,
            height: 600.0,
        }
    }

    #[test]
    fn test_all_patterns_produce_correct_count() {
        let config = test_config();
        for pattern in PositionPattern::all() {
            let particles = generate_positions(*pattern, &config);
            assert_eq!(
                particles.len(),
                config.num_particles,
                "Pattern {:?} produced wrong count",
                pattern
            );
        }
    }

    #[test]
    fn test_particles_in_bounds() {
        let config = test_config();
        for pattern in PositionPattern::all() {
            let particles = generate_positions(*pattern, &config);
            for (i, p) in particles.iter().enumerate() {
                assert!(
                    p.x >= 0.0 && p.x <= config.width,
                    "Pattern {:?} particle {i} x={} out of bounds",
                    pattern,
                    p.x
                );
                assert!(
                    p.y >= 0.0 && p.y <= config.height,
                    "Pattern {:?} particle {i} y={} out of bounds",
                    pattern,
                    p.y
                );
            }
        }
    }

    #[test]
    fn test_empty_config() {
        let config = SpawnConfig {
            num_particles: 0,
            num_types: 4,
            width: 800.0,
            height: 600.0,
        };
        let particles = generate_positions(PositionPattern::Random, &config);
        assert!(particles.is_empty());
    }
}

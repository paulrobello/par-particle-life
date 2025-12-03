//! Color palette generators for particle types.
//!
//! This module provides 37 different color palette generators,
//! from simple rainbow gradients to complex procedural palettes.

use rand::Rng;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

/// A color in RGBA format with f32 components [0.0, 1.0].
pub type Color = [f32; 4];

/// Types of color palettes available.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum PaletteType {
    #[default]
    Random = 0,
    Rainbow = 1,
    NeonWarm = 2,
    HeatmapClassic = 3,
    HeatmapCool = 4,
    HeatmapWarm = 5,
    Pastel = 6,
    ColdBlue = 7,
    SciFiSpectrum = 8,
    ThermalGlow = 9,
    CrimsonFlame = 10,
    Fire = 11,
    VioletFade = 12,
    Grayscale = 13,
    DesertWarm = 14,
    DualGradient = 15,
    Candy = 16,
    OrganicFlow = 17,
    EarthFlow = 18,
    GameBoyDMG = 19,
    PaperAndInk = 20,
    FluoroSport = 21,
    MidnightCircuit = 22,
    BioluminescentAbyss = 23,
    Blueprint = 24,
    CyberDark = 25,
    HolographicFoil = 26,
    MineralGemstones = 27,
    VaporwavePastel = 28,
    SolarizedDrift = 29,
    Aurora = 30,
    CyberNeon = 31,
    GoldenAngleJitter = 32,
    CMYKMisregister = 33,
    AnodizedMetal = 34,
    InkBleedWatercolor = 35,
    HolographicFoil2 = 36,
}

impl PaletteType {
    /// Get all available palette types.
    pub fn all() -> &'static [PaletteType] {
        use PaletteType::*;
        &[
            Random,
            Rainbow,
            NeonWarm,
            HeatmapClassic,
            HeatmapCool,
            HeatmapWarm,
            Pastel,
            ColdBlue,
            SciFiSpectrum,
            ThermalGlow,
            CrimsonFlame,
            Fire,
            VioletFade,
            Grayscale,
            DesertWarm,
            DualGradient,
            Candy,
            OrganicFlow,
            EarthFlow,
            GameBoyDMG,
            PaperAndInk,
            FluoroSport,
            MidnightCircuit,
            BioluminescentAbyss,
            Blueprint,
            CyberDark,
            HolographicFoil,
            MineralGemstones,
            VaporwavePastel,
            SolarizedDrift,
            Aurora,
            CyberNeon,
            GoldenAngleJitter,
            CMYKMisregister,
            AnodizedMetal,
            InkBleedWatercolor,
            HolographicFoil2,
        ]
    }

    /// Get the display name for this palette.
    pub fn display_name(&self) -> &'static str {
        match self {
            PaletteType::Random => "Random",
            PaletteType::Rainbow => "Rainbow",
            PaletteType::NeonWarm => "Neon Warm",
            PaletteType::HeatmapClassic => "Heatmap Classic",
            PaletteType::HeatmapCool => "Heatmap Cool",
            PaletteType::HeatmapWarm => "Heatmap Warm",
            PaletteType::Pastel => "Pastel",
            PaletteType::ColdBlue => "Cold Blue",
            PaletteType::SciFiSpectrum => "Sci-Fi Spectrum",
            PaletteType::ThermalGlow => "Thermal Glow",
            PaletteType::CrimsonFlame => "Crimson Flame",
            PaletteType::Fire => "Fire",
            PaletteType::VioletFade => "Violet Fade",
            PaletteType::Grayscale => "Grayscale",
            PaletteType::DesertWarm => "Desert Warm",
            PaletteType::DualGradient => "Dual Gradient",
            PaletteType::Candy => "Candy",
            PaletteType::OrganicFlow => "Organic Flow",
            PaletteType::EarthFlow => "Earth Flow",
            PaletteType::GameBoyDMG => "Game Boy DMG",
            PaletteType::PaperAndInk => "Paper & Ink",
            PaletteType::FluoroSport => "Fluoro Sport",
            PaletteType::MidnightCircuit => "Midnight Circuit",
            PaletteType::BioluminescentAbyss => "BioLuminescent Abyss",
            PaletteType::Blueprint => "Blueprint",
            PaletteType::CyberDark => "Cyber Dark",
            PaletteType::HolographicFoil => "Holographic Foil",
            PaletteType::MineralGemstones => "Mineral Gemstones",
            PaletteType::VaporwavePastel => "Vaporwave Pastel",
            PaletteType::SolarizedDrift => "Solarized Drift",
            PaletteType::Aurora => "Aurora",
            PaletteType::CyberNeon => "Cyber Neon",
            PaletteType::GoldenAngleJitter => "Golden Angle Jitter",
            PaletteType::CMYKMisregister => "CMYK Misregister",
            PaletteType::AnodizedMetal => "Anodized Metal",
            PaletteType::InkBleedWatercolor => "Ink Bleed Watercolor",
            PaletteType::HolographicFoil2 => "Holographic Foil 2",
        }
    }

    /// Get the category for this palette.
    pub fn category(&self) -> &'static str {
        match self {
            PaletteType::Random => "Default",
            PaletteType::Rainbow
            | PaletteType::NeonWarm
            | PaletteType::HeatmapClassic
            | PaletteType::HeatmapCool
            | PaletteType::HeatmapWarm
            | PaletteType::Pastel
            | PaletteType::ColdBlue
            | PaletteType::SciFiSpectrum
            | PaletteType::ThermalGlow
            | PaletteType::CrimsonFlame
            | PaletteType::Fire
            | PaletteType::VioletFade
            | PaletteType::Grayscale
            | PaletteType::DesertWarm => "Static",
            PaletteType::DualGradient
            | PaletteType::Candy
            | PaletteType::OrganicFlow
            | PaletteType::EarthFlow
            | PaletteType::GameBoyDMG
            | PaletteType::PaperAndInk
            | PaletteType::FluoroSport
            | PaletteType::MidnightCircuit
            | PaletteType::BioluminescentAbyss
            | PaletteType::Blueprint
            | PaletteType::CyberDark => "Generative",
            _ => "Experimental",
        }
    }
}

/// Trait for color palette generation.
pub trait ColorPalette {
    /// Generate colors for the given number of particle types.
    fn generate(&self, num_types: usize) -> Vec<Color>;
}

impl ColorPalette for PaletteType {
    fn generate(&self, num_types: usize) -> Vec<Color> {
        generate_colors(*self, num_types)
    }
}

/// Generate colors using the specified palette type.
pub fn generate_colors(palette: PaletteType, num_types: usize) -> Vec<Color> {
    if num_types == 0 {
        return Vec::new();
    }

    match palette {
        PaletteType::Random => random_generator(num_types),
        PaletteType::Rainbow => rainbow_generator(num_types),
        PaletteType::NeonWarm => neon_warm_generator(num_types),
        PaletteType::HeatmapClassic => gradient_palette(num_types, &HEATMAP_CLASSIC),
        PaletteType::HeatmapCool => gradient_palette(num_types, &HEATMAP_COOL),
        PaletteType::HeatmapWarm => gradient_palette(num_types, &HEATMAP_WARM),
        PaletteType::Pastel => pastel_generator(num_types),
        PaletteType::ColdBlue => gradient_palette(num_types, &COLD_BLUE),
        PaletteType::SciFiSpectrum => gradient_palette(num_types, &SCIFI_SPECTRUM),
        PaletteType::ThermalGlow => gradient_palette(num_types, &THERMAL_GLOW),
        PaletteType::CrimsonFlame => crimson_flame_generator(num_types),
        PaletteType::Fire => fire_generator(num_types),
        PaletteType::VioletFade => violet_fade_generator(num_types),
        PaletteType::Grayscale => gradient_palette(num_types, &GRAYSCALE),
        PaletteType::DesertWarm => gradient_palette(num_types, &DESERT_WARM),
        PaletteType::DualGradient => dual_gradient_generator(num_types),
        PaletteType::Candy => candy_generator(num_types),
        PaletteType::OrganicFlow => organic_flow_generator(num_types),
        PaletteType::EarthFlow => earth_flow_generator(num_types),
        PaletteType::GameBoyDMG => gameboy_dmg_generator(num_types),
        PaletteType::PaperAndInk => paper_ink_generator(num_types),
        PaletteType::FluoroSport => fluoro_sport_generator(num_types),
        PaletteType::MidnightCircuit => midnight_circuit_generator(num_types),
        PaletteType::BioluminescentAbyss => biolum_abyss_generator(num_types),
        PaletteType::Blueprint => blueprint_generator(num_types),
        PaletteType::CyberDark => cyber_dark_generator(num_types),
        PaletteType::HolographicFoil | PaletteType::HolographicFoil2 => {
            holo_foil_generator(num_types)
        }
        PaletteType::MineralGemstones => gemstones_generator(num_types),
        PaletteType::VaporwavePastel => vaporwave_pastel_generator(num_types),
        PaletteType::SolarizedDrift => solarized_drift_generator(num_types),
        PaletteType::Aurora => aurora_generator(num_types),
        PaletteType::CyberNeon => cyber_neon_generator(num_types),
        PaletteType::GoldenAngleJitter => golden_angle_jitter_generator(num_types),
        PaletteType::CMYKMisregister => cmyk_misregister_generator(num_types),
        PaletteType::AnodizedMetal => anodized_metal_generator(num_types),
        PaletteType::InkBleedWatercolor => ink_bleed_watercolor_generator(num_types),
    }
}

// === Helper Functions ===

fn clamp(x: f32, min: f32, max: f32) -> f32 {
    x.clamp(min, max)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Convert HSV to RGB.
/// h: 0-360, s: 0-1, v: 0-1
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [f32; 3] {
    let c = v * s;
    let hp = h / 60.0;
    let x = c * (1.0 - ((hp % 2.0) - 1.0).abs());

    let (r, g, b) = if hp < 1.0 {
        (c, x, 0.0)
    } else if hp < 2.0 {
        (x, c, 0.0)
    } else if hp < 3.0 {
        (0.0, c, x)
    } else if hp < 4.0 {
        (0.0, x, c)
    } else if hp < 5.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    let m = v - c;
    [r + m, g + m, b + m]
}

/// Key color for gradient interpolation.
struct KeyColor {
    t: f32,
    r: f32,
    g: f32,
    b: f32,
}

/// Generate a gradient palette from key colors.
fn gradient_palette(num_types: usize, keys: &[KeyColor]) -> Vec<Color> {
    let mut colors = Vec::with_capacity(num_types);

    let mut k = 0;
    for i in 0..num_types {
        let u = if num_types <= 1 {
            0.0
        } else {
            i as f32 / (num_types - 1) as f32
        };

        while k < keys.len() - 2 && u > keys[k + 1].t {
            k += 1;
        }

        let a = &keys[k];
        let b = &keys[k + 1];
        let span = (b.t - a.t).max(1e-6);
        let v = (u - a.t) / span;

        let r = clamp(lerp(a.r, b.r, v), 0.0, 1.0);
        let g = clamp(lerp(a.g, b.g, v), 0.0, 1.0);
        let bl = clamp(lerp(a.b, b.b, v), 0.0, 1.0);

        colors.push([r, g, bl, 1.0]);
    }

    colors
}

// === Static Gradient Definitions ===

const COLD_BLUE: [KeyColor; 5] = [
    KeyColor {
        t: 0.00,
        r: 0.05,
        g: 0.10,
        b: 0.35,
    },
    KeyColor {
        t: 0.25,
        r: 0.10,
        g: 0.25,
        b: 0.70,
    },
    KeyColor {
        t: 0.50,
        r: 0.20,
        g: 0.55,
        b: 0.95,
    },
    KeyColor {
        t: 0.75,
        r: 0.55,
        g: 0.80,
        b: 1.00,
    },
    KeyColor {
        t: 1.00,
        r: 0.85,
        g: 0.95,
        b: 1.00,
    },
];

const SCIFI_SPECTRUM: [KeyColor; 5] = [
    KeyColor {
        t: 0.00,
        r: 0.00,
        g: 0.00,
        b: 0.30,
    },
    KeyColor {
        t: 0.25,
        r: 0.00,
        g: 0.20,
        b: 1.00,
    },
    KeyColor {
        t: 0.50,
        r: 0.00,
        g: 1.00,
        b: 0.40,
    },
    KeyColor {
        t: 0.75,
        r: 1.00,
        g: 1.00,
        b: 0.40,
    },
    KeyColor {
        t: 1.00,
        r: 1.00,
        g: 0.40,
        b: 1.00,
    },
];

const THERMAL_GLOW: [KeyColor; 6] = [
    KeyColor {
        t: 0.00,
        r: 0.00,
        g: 0.00,
        b: 0.25,
    },
    KeyColor {
        t: 0.20,
        r: 0.00,
        g: 0.25,
        b: 0.80,
    },
    KeyColor {
        t: 0.40,
        r: 0.00,
        g: 0.85,
        b: 0.40,
    },
    KeyColor {
        t: 0.60,
        r: 0.95,
        g: 0.85,
        b: 0.00,
    },
    KeyColor {
        t: 0.80,
        r: 1.00,
        g: 0.40,
        b: 0.00,
    },
    KeyColor {
        t: 1.00,
        r: 0.90,
        g: 0.00,
        b: 0.65,
    },
];

const HEATMAP_CLASSIC: [KeyColor; 5] = [
    KeyColor {
        t: 0.00,
        r: 0.00,
        g: 0.00,
        b: 0.25,
    },
    KeyColor {
        t: 0.25,
        r: 0.00,
        g: 0.80,
        b: 1.00,
    },
    KeyColor {
        t: 0.50,
        r: 1.00,
        g: 1.00,
        b: 1.00,
    },
    KeyColor {
        t: 0.75,
        r: 1.00,
        g: 1.00,
        b: 0.00,
    },
    KeyColor {
        t: 1.00,
        r: 0.80,
        g: 0.00,
        b: 0.00,
    },
];

const HEATMAP_COOL: [KeyColor; 5] = [
    KeyColor {
        t: 0.00,
        r: 0.05,
        g: 0.10,
        b: 0.35,
    },
    KeyColor {
        t: 0.25,
        r: 0.10,
        g: 0.25,
        b: 0.85,
    },
    KeyColor {
        t: 0.50,
        r: 0.00,
        g: 0.80,
        b: 0.80,
    },
    KeyColor {
        t: 0.75,
        r: 1.00,
        g: 0.90,
        b: 0.10,
    },
    KeyColor {
        t: 1.00,
        r: 1.00,
        g: 1.00,
        b: 1.00,
    },
];

const HEATMAP_WARM: [KeyColor; 5] = [
    KeyColor {
        t: 0.00,
        r: 0.45,
        g: 0.00,
        b: 0.00,
    },
    KeyColor {
        t: 0.25,
        r: 0.90,
        g: 0.20,
        b: 0.00,
    },
    KeyColor {
        t: 0.55,
        r: 1.00,
        g: 0.65,
        b: 0.00,
    },
    KeyColor {
        t: 0.80,
        r: 1.00,
        g: 0.90,
        b: 0.40,
    },
    KeyColor {
        t: 1.00,
        r: 1.00,
        g: 1.00,
        b: 1.00,
    },
];

const GRAYSCALE: [KeyColor; 3] = [
    KeyColor {
        t: 0.00,
        r: 0.05,
        g: 0.05,
        b: 0.05,
    },
    KeyColor {
        t: 0.75,
        r: 0.65,
        g: 0.65,
        b: 0.65,
    },
    KeyColor {
        t: 1.00,
        r: 1.00,
        g: 1.00,
        b: 1.00,
    },
];

const DESERT_WARM: [KeyColor; 5] = [
    KeyColor {
        t: 0.00,
        r: 0.9647,
        g: 0.8863,
        b: 0.7020,
    },
    KeyColor {
        t: 0.25,
        r: 0.9098,
        g: 0.7529,
        b: 0.4471,
    },
    KeyColor {
        t: 0.50,
        r: 0.8471,
        g: 0.5373,
        b: 0.2275,
    },
    KeyColor {
        t: 0.75,
        r: 0.7216,
        g: 0.3608,
        b: 0.1843,
    },
    KeyColor {
        t: 1.00,
        r: 0.3686,
        g: 0.2275,
        b: 0.1804,
    },
];

// === Generator Implementations ===

fn random_generator(n: usize) -> Vec<Color> {
    let mut rng = rand::rng();
    (0..n)
        .map(|_| [rng.random(), rng.random(), rng.random(), 1.0])
        .collect()
}

fn rainbow_generator(n: usize) -> Vec<Color> {
    (0..n)
        .map(|i| {
            let hue = (i as f32 / n as f32) * 360.0;
            let [r, g, b] = hsv_to_rgb(hue, 1.0, 1.0);
            [r, g, b, 1.0]
        })
        .collect()
}

fn pastel_generator(n: usize) -> Vec<Color> {
    (0..n)
        .map(|i| {
            let hue = (i as f32 / n as f32) * 360.0;
            let [r, g, b] = hsv_to_rgb(hue, 0.5, 1.0);
            [r, g, b, 1.0]
        })
        .collect()
}

fn neon_warm_generator(n: usize) -> Vec<Color> {
    (0..n)
        .map(|i| {
            let t = if n <= 1 {
                0.0
            } else {
                i as f32 / (n - 1) as f32
            };
            let h = (lerp(20.0, 300.0, t) % 360.0 + 360.0) % 360.0;
            let s = lerp(1.0, 0.7, t);
            let v = lerp(1.0, 0.8, t);
            let [r, g, b] = hsv_to_rgb(h, s, v);
            [r, g, b, 1.0]
        })
        .collect()
}

fn fire_generator(n: usize) -> Vec<Color> {
    (0..n)
        .map(|i| {
            let t = if n <= 1 {
                0.0
            } else {
                i as f32 / (n - 1) as f32
            };
            let h = lerp(5.0, 45.0, t);
            let s = lerp(1.0, 0.9, t);
            let v = lerp(0.9, 1.0, t);
            let [r, g, b] = hsv_to_rgb(h, s, v);
            [r, g, b, 1.0]
        })
        .collect()
}

fn violet_fade_generator(n: usize) -> Vec<Color> {
    (0..n)
        .map(|i| {
            let t = if n <= 1 {
                0.0
            } else {
                i as f32 / (n - 1) as f32
            };
            let h = lerp(345.0, 260.0, t);
            let s = lerp(0.9, 0.55, t);
            let v = lerp(0.95, 0.35, t);
            let [r, g, b] = hsv_to_rgb((h + 360.0) % 360.0, s, v);
            [r, g, b, 1.0]
        })
        .collect()
}

fn crimson_flame_generator(n: usize) -> Vec<Color> {
    (0..n)
        .map(|i| {
            let t = if n <= 1 {
                0.0
            } else {
                i as f32 / (n - 1) as f32
            };
            let h = (2.0 - (2.0 - (-30.0)) * t + 360.0) % 360.0;
            let s = lerp(1.0, 0.7, t);
            let v = lerp(0.95, 0.45, t);
            let [r, g, b] = hsv_to_rgb(h, s, v);
            [r, g, b, 1.0]
        })
        .collect()
}

fn dual_gradient_generator(n: usize) -> Vec<Color> {
    let mut rng = rand::rng();
    let start_h: f32 = rng.random::<f32>() * 360.0;
    let mut end_h: f32 = rng.random::<f32>() * 360.0;

    // Ensure minimum hue difference
    let hue_delta = ((end_h - start_h + 540.0) % 360.0) - 180.0;
    if hue_delta.abs() < 70.0 {
        let sign = if hue_delta >= 0.0 { 1.0 } else { -1.0 };
        end_h = (start_h + sign * 70.0 + 360.0) % 360.0;
    }

    let start_s = 0.70 + rng.random::<f32>() * 0.30;
    let end_s = 0.70 + rng.random::<f32>() * 0.30;
    let start_v = 0.80 + rng.random::<f32>() * 0.20;
    let end_v = 0.70 + rng.random::<f32>() * 0.30;

    let dh = ((end_h - start_h + 540.0) % 360.0) - 180.0;

    (0..n)
        .map(|i| {
            let t = if n <= 1 {
                0.0
            } else {
                i as f32 / (n - 1) as f32
            };
            let h = (start_h + dh * t + 360.0) % 360.0;
            let s = start_s + (end_s - start_s) * t;
            let v = start_v + (end_v - start_v) * t;
            let [r, g, b] = hsv_to_rgb(h, s, v);
            [r, g, b, 1.0]
        })
        .collect()
}

fn candy_generator(n: usize) -> Vec<Color> {
    let mut rng = rand::rng();
    let phi = 137.507_77_f32;
    let base_h: f32 = rng.random::<f32>() * 360.0;

    (0..n)
        .map(|i| {
            let jitter = (rng.random::<f32>() - 0.5) * 16.0;
            let h = (base_h + i as f32 * phi + jitter) % 360.0;
            let s = clamp(0.8 + (rng.random::<f32>() - 0.5) * 0.16, 0.0, 1.0);
            let v = clamp(0.9 + (rng.random::<f32>() - 0.5) * 0.12, 0.0, 1.0);
            let [r, g, b] = hsv_to_rgb(h, s, v);
            [r, g, b, 1.0]
        })
        .collect()
}

fn organic_flow_generator(n: usize) -> Vec<Color> {
    let mut rng = rand::rng();
    let base_h: f32 = rng.random::<f32>() * 15.0;

    (0..n)
        .map(|i| {
            let jitter = (rng.random::<f32>() - 0.5) * 16.0;
            let extra = if i % 4 == 0 {
                rng.random::<f32>() * 15.0 + 15.0
            } else {
                0.0
            };
            let h = (base_h + jitter + extra) % 360.0;
            let s = clamp(0.7 + (rng.random::<f32>() - 0.5) * 0.2, 0.0, 1.0);
            let v = clamp(0.45 + rng.random::<f32>().abs() * 0.4, 0.0, 1.0);
            let [r, g, b] = hsv_to_rgb((h + 360.0) % 360.0, s, v);
            [r, g, b, 1.0]
        })
        .collect()
}

fn earth_flow_generator(n: usize) -> Vec<Color> {
    let mut rng = rand::rng();
    let h_a: f32 = rng.random::<f32>() * 20.0 + 10.0;
    let h_b = (h_a + rng.random::<f32>() * 80.0 + 140.0) % 360.0;
    let phase = rng.random::<f32>() * PI;

    (0..n)
        .map(|i| {
            let u = if n <= 1 {
                0.0
            } else {
                i as f32 / (n - 1) as f32
            };
            let a = 2.0 * PI * u + phase;
            let mix = 0.5 + 0.5 * a.sin();
            let jitter = (rng.random::<f32>() - 0.5) * 6.0;
            let hue = (h_a * (1.0 - mix) + h_b * mix + jitter + 360.0) % 360.0;
            let s = clamp(0.75 + (rng.random::<f32>() - 0.5) * 0.14, 0.0, 1.0);
            let v = clamp(
                0.60 + 0.35 * (a + 1.1).sin() + (rng.random::<f32>() - 0.5) * 0.12,
                0.0,
                1.0,
            );
            let [r, g, b] = hsv_to_rgb(hue, s, v);
            [r, g, b, 1.0]
        })
        .collect()
}

fn gameboy_dmg_generator(n: usize) -> Vec<Color> {
    let mut rng = rand::rng();
    let steps = [0.2f32, 0.35, 0.55, 0.78];
    let hue: f32 = rng.random::<f32>() * 20.0 + 90.0;

    (0..n)
        .map(|i| {
            let v = clamp(steps[i % 4] + (rng.random::<f32>() - 0.5) * 0.06, 0.0, 1.0);
            let s = clamp(0.25 + (rng.random::<f32>() - 0.5) * 0.1, 0.0, 1.0);
            let [r, g, b] = hsv_to_rgb(hue, s, v);
            [r * 0.9, g, b * 0.9, 1.0]
        })
        .collect()
}

fn paper_ink_generator(n: usize) -> Vec<Color> {
    let mut rng = rand::rng();
    let inks = [210.0f32, 30.0, 220.0];

    (0..n)
        .map(|i| {
            if i % 4 == 0 {
                // Paper
                let v = clamp(0.92 + (rng.random::<f32>() - 0.5) * 0.06, 0.0, 1.0);
                let tint = (rng.random::<f32>() - 0.5) * 12.0;
                let [r, g, b] = hsv_to_rgb((45.0 + tint + 360.0) % 360.0, 0.18, v);
                [r, g, b, 1.0]
            } else {
                // Ink
                let hue = inks[rng.random_range(0..3)];
                let jitter = (rng.random::<f32>() - 0.5) * 12.0;
                let s = clamp(0.6 + (rng.random::<f32>() - 0.5) * 0.2, 0.0, 1.0);
                let v = clamp(0.35 + (rng.random::<f32>() - 0.5) * 0.16, 0.0, 1.0);
                let [r, g, b] = hsv_to_rgb((hue + jitter + 360.0) % 360.0, s, v);
                [r, g, b, 1.0]
            }
        })
        .collect()
}

fn fluoro_sport_generator(n: usize) -> Vec<Color> {
    let mut rng = rand::rng();
    let accents = [95.0f32, 175.0, 310.0];

    (0..n)
        .map(|i| {
            let is_accent = i % 4 == 0;
            let (h, s, v): (f32, f32, f32) = if is_accent {
                let h = accents[rng.random_range(0..3)] + (rng.random::<f32>() - 0.5) * 12.0;
                (h, 0.95, 0.98)
            } else {
                let h = rng.random::<f32>() * 50.0 + 210.0 + (rng.random::<f32>() - 0.5) * 16.0;
                (h, 0.25, 0.18)
            };
            let [r, g, b] = hsv_to_rgb((h + 360.0) % 360.0, s.clamp(0.0, 1.0), v.clamp(0.0, 1.0));
            [r, g, b, 1.0]
        })
        .collect()
}

fn midnight_circuit_generator(n: usize) -> Vec<Color> {
    let mut rng = rand::rng();
    let accent_h: f32 = rng.random::<f32>() * 340.0 + 10.0;
    let accent_period = (n / 3).max(3);

    (0..n)
        .map(|i| {
            if i % accent_period == 0 {
                let h = (accent_h + (rng.random::<f32>() - 0.5) * 16.0 + 360.0) % 360.0;
                let [r, g, b] = hsv_to_rgb(h, 0.9, 0.95);
                [r, g, b, 1.0]
            } else {
                let mut v = clamp(0.25 + rng.random::<f32>() * 0.55, 0.0, 1.0);
                v = v.powf(1.1);
                [v, v, v, 1.0]
            }
        })
        .collect()
}

fn biolum_abyss_generator(n: usize) -> Vec<Color> {
    let mut rng = rand::rng();
    let accent_count = (n / 4).clamp(1, 2);

    (0..n)
        .map(|i| {
            if i < accent_count {
                let h: f32 = rng.random::<f32>() * 15.0 + 185.0;
                let [r, g, b] = hsv_to_rgb(h, 0.95, 0.92);
                [r, g, b, 1.0]
            } else {
                let h: f32 = rng.random::<f32>() * 25.0 + 200.0;
                let v = 0.12 + (i as f32 / n as f32) * 0.08;
                let [r, g, b] = hsv_to_rgb(h, 0.88, v);
                [r, g, b, 1.0]
            }
        })
        .collect()
}

fn blueprint_generator(n: usize) -> Vec<Color> {
    let mut rng = rand::rng();
    let accent_count = (n / 5).clamp(1, 2);

    (0..n)
        .map(|i| {
            if i < accent_count {
                let v = 0.92 + rng.random::<f32>() * 0.06;
                [v, v, v, 1.0]
            } else {
                let h: f32 = rng.random::<f32>() * 20.0 + 215.0;
                let s = 0.65 + rng.random::<f32>() * 0.10;
                let v = 0.35 + rng.random::<f32>() * 0.15;
                let [r, g, b] = hsv_to_rgb(h, s, v);
                [r, g, b, 1.0]
            }
        })
        .collect()
}

fn cyber_dark_generator(n: usize) -> Vec<Color> {
    let mut rng = rand::rng();
    let accent_h: f32 = rng.random::<f32>() * 340.0 + 10.0;
    let accent_period = (n / 3).max(3);

    (0..n)
        .map(|i| {
            if i % accent_period == 0 {
                let h = (accent_h + (rng.random::<f32>() - 0.5) * 16.0 + 360.0) % 360.0;
                let [r, g, b] = hsv_to_rgb(h, 0.9, 0.95);
                [r, g, b, 1.0]
            } else {
                let v = (0.25 + rng.random::<f32>() * 0.55).powf(1.1);
                [v, v, v, 1.0]
            }
        })
        .collect()
}

fn holo_foil_generator(n: usize) -> Vec<Color> {
    let mut rng = rand::rng();
    let k1: f32 = rng.random::<f32>() * 0.6 + 0.8;
    let k2: f32 = rng.random::<f32>() * 1.4 + 2.2;

    (0..n)
        .map(|i| {
            let t = if n <= 1 {
                0.0
            } else {
                i as f32 / (n - 1) as f32
            };
            let jitter = (rng.random::<f32>() - 0.5) * 8.0;
            let h = (360.0
                * (t + 0.05 * (2.0 * PI * k1 * t).sin() + 0.03 * (2.0 * PI * k2 * t).sin())
                + jitter)
                % 360.0;
            let s = clamp(
                0.5 + 0.4 * (2.0 * PI * (k1 + k2) * t).sin() + (rng.random::<f32>() - 0.5) * 0.1,
                0.0,
                1.0,
            );
            let v = clamp(
                0.85 + 0.1 * (2.0 * PI * k2 * t).cos() + (rng.random::<f32>() - 0.5) * 0.06,
                0.0,
                1.0,
            );
            let [r, g, b] = hsv_to_rgb((h + 360.0) % 360.0, s, v);
            [r, g, b, 1.0]
        })
        .collect()
}

fn gemstones_generator(n: usize) -> Vec<Color> {
    let mut rng = rand::rng();
    let hues = [140.0f32, 350.0, 220.0, 45.0, 200.0, 300.0];

    (0..n)
        .map(|i| {
            let h = (hues[i % 6] + (rng.random::<f32>() - 0.5) * 10.0 + 360.0) % 360.0;
            let s = clamp(0.75 + (rng.random::<f32>() - 0.5) * 0.16, 0.0, 1.0);
            let v = clamp(0.7 + (rng.random::<f32>() - 0.5) * 0.2, 0.0, 1.0);
            let [r, g, b] = hsv_to_rgb(h, s, v);
            [r, g, b, 1.0]
        })
        .collect()
}

fn vaporwave_pastel_generator(n: usize) -> Vec<Color> {
    let mut rng = rand::rng();
    let anchors = [320.0f32, 260.0, 170.0];

    (0..n)
        .map(|i| {
            let h0 = anchors[i % 3];
            let h = (h0 + (rng.random::<f32>() - 0.5) * 20.0 + 360.0) % 360.0;
            let s = clamp(0.35 + (rng.random::<f32>() - 0.5) * 0.16, 0.0, 1.0);
            let v = clamp(0.95 + (rng.random::<f32>() - 0.5) * 0.08, 0.0, 1.0);
            let [r, g, b] = hsv_to_rgb(h, s, v);
            [r, g, b, 1.0]
        })
        .collect()
}

fn solarized_drift_generator(n: usize) -> Vec<Color> {
    let mut rng = rand::rng();
    let anchors = [
        (44.0f32, 0.55, 0.92),
        (44.0, 0.25, 0.60),
        (192.0, 0.55, 0.85),
        (220.0, 0.55, 0.80),
        (64.0, 0.55, 0.85),
        (18.0, 0.65, 0.90),
        (350.0, 0.55, 0.85),
        (300.0, 0.40, 0.85),
    ];

    (0..n)
        .map(|i| {
            let (ah, as_, av) = anchors[i % 8];
            let h = (ah + (rng.random::<f32>() - 0.5) * 10.0 + 360.0) % 360.0;
            let s = clamp(as_ + (rng.random::<f32>() - 0.5) * 0.12, 0.0, 1.0);
            let v = clamp(av + (rng.random::<f32>() - 0.5) * 0.1, 0.0, 1.0);
            let [r, g, b] = hsv_to_rgb(h, s, v);
            [r, g, b, 1.0]
        })
        .collect()
}

fn aurora_generator(n: usize) -> Vec<Color> {
    let mut rng = rand::rng();
    let center: f32 = rng.random::<f32>() * 100.0 + 120.0;

    (0..n)
        .map(|_| {
            let spread: f32 = rng.random::<f32>() * 40.0 + 20.0;
            let jitter = (rng.random::<f32>() - 0.5) * 2.0 * spread;
            let h = (center + jitter + 360.0) % 360.0;
            let s = clamp(0.6 + (rng.random::<f32>() - 0.5) * 0.3, 0.0, 1.0);
            let v = clamp(0.75 + (rng.random::<f32>() - 0.5) * 0.24, 0.0, 1.0);
            let [r, g, b] = hsv_to_rgb(h, s, v);
            [r, g, b, 1.0]
        })
        .collect()
}

fn cyber_neon_generator(n: usize) -> Vec<Color> {
    let mut rng = rand::rng();
    let base_h: f32 = rng.random::<f32>() * 60.0 + 280.0;

    (0..n)
        .map(|i| {
            let scale: f32 = rng.random::<f32>() * 0.5 + 0.6;
            let jitter = (rng.random::<f32>() - 0.5) * 12.0;
            let h = (base_h + i as f32 * (360.0 / n as f32) * scale + jitter) % 360.0;
            let s = clamp(0.9 + (rng.random::<f32>() - 0.5) * 0.1, 0.0, 1.0);
            let v = clamp(0.9 + (rng.random::<f32>() - 0.5) * 0.14, 0.0, 1.0);
            let [r, g, b] = hsv_to_rgb((h + 360.0) % 360.0, s, v);
            [r, g, b, 1.0]
        })
        .collect()
}

fn golden_angle_jitter_generator(n: usize) -> Vec<Color> {
    let mut rng = rand::rng();
    let phi = 137.507_77_f32;
    let base_h: f32 = rng.random::<f32>() * 360.0;
    let s_base: f32 = rng.random::<f32>() * 0.35 + 0.6;
    let v_base: f32 = rng.random::<f32>() * 0.2 + 0.8;
    let jitter_h: f32 = rng.random::<f32>() * 8.0 + 2.0;

    (0..n)
        .map(|i| {
            let jitter = (rng.random::<f32>() - 0.5) * 2.0 * jitter_h;
            let h = (base_h + i as f32 * phi + jitter) % 360.0;
            let s = clamp(s_base + (rng.random::<f32>() - 0.5) * 0.14, 0.0, 1.0);
            let v = clamp(v_base + (rng.random::<f32>() - 0.5) * 0.1, 0.0, 1.0);
            let [r, g, b] = hsv_to_rgb((h + 360.0) % 360.0, s, v);
            [r, g, b, 1.0]
        })
        .collect()
}

fn cmyk_misregister_generator(n: usize) -> Vec<Color> {
    let mut rng = rand::rng();
    let inks = [200.0f32, 300.0, 55.0, 220.0];

    (0..n)
        .map(|i| {
            let h = (inks[i % 4] + (rng.random::<f32>() - 0.5) * 10.0 + 360.0) % 360.0;
            let (s, v) = if i % 4 == 3 { (0.1, 0.35) } else { (0.9, 0.95) };
            let s = clamp(s + (rng.random::<f32>() - 0.5) * 0.1, 0.0, 1.0);
            let v = clamp(v + (rng.random::<f32>() - 0.5) * 0.1, 0.0, 1.0);
            let [r, g, b] = hsv_to_rgb(h, s, v);
            [r, g, b, 1.0]
        })
        .collect()
}

fn anodized_metal_generator(n: usize) -> Vec<Color> {
    let mut rng = rand::rng();
    let hue0: f32 = rng.random::<f32>() * 140.0 + 180.0;

    (0..n)
        .map(|i| {
            let t = if n <= 1 {
                0.0
            } else {
                i as f32 / (n - 1) as f32
            };
            let jitter = (rng.random::<f32>() - 0.5) * 6.0;
            let h = (hue0 + 40.0 * (2.0 * PI * t).sin() + jitter) % 360.0;
            let s = clamp(
                0.6 + 0.25 * (4.0 * PI * t + 1.2).sin() + (rng.random::<f32>() - 0.5) * 0.06,
                0.0,
                1.0,
            );
            let v = clamp(
                0.65 + 0.3 * (4.0 * PI * t).cos() + (rng.random::<f32>() - 0.5) * 0.06,
                0.0,
                1.0,
            );
            let [r, g, b] = hsv_to_rgb((h + 360.0) % 360.0, s, v);
            [r * 0.96, g * 0.96, b * 0.96, 1.0]
        })
        .collect()
}

fn ink_bleed_watercolor_generator(n: usize) -> Vec<Color> {
    let mut rng = rand::rng();
    let center: f32 = rng.random::<f32>() * 70.0 + 190.0;

    (0..n)
        .map(|i| {
            let jitter = (rng.random::<f32>() - 0.5) * 36.0;
            let h = (center + jitter + 360.0) % 360.0;
            let s = clamp(0.2 + rng.random::<f32>().abs() * 0.24, 0.0, 1.0);
            let phase = rng.random::<f32>() * PI;
            let v = clamp(
                0.7 + 0.25 * (i as f32 * 0.9 + phase).sin() + (rng.random::<f32>() - 0.5) * 0.12,
                0.0,
                1.0,
            );
            let [r, g, b] = hsv_to_rgb(h, s, v);
            [r, g, b, 1.0]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_palettes_produce_valid_colors() {
        for palette in PaletteType::all() {
            let colors = generate_colors(*palette, 8);
            assert_eq!(colors.len(), 8);
            for color in &colors {
                for component in &color[..3] {
                    assert!(
                        *component >= 0.0 && *component <= 1.0,
                        "Palette {:?} produced invalid color component {component}",
                        palette
                    );
                }
                assert!((color[3] - 1.0).abs() < 0.001, "Alpha should be 1.0");
            }
        }
    }

    #[test]
    fn test_rainbow_hue_distribution() {
        let colors = rainbow_generator(6);
        // Colors should span the hue range
        assert_eq!(colors.len(), 6);
    }

    #[test]
    fn test_empty_palette() {
        let colors = generate_colors(PaletteType::Random, 0);
        assert!(colors.is_empty());
    }

    #[test]
    fn test_hsv_to_rgb() {
        let red = hsv_to_rgb(0.0, 1.0, 1.0);
        assert!((red[0] - 1.0).abs() < 0.01);
        assert!(red[1] < 0.01);
        assert!(red[2] < 0.01);

        let green = hsv_to_rgb(120.0, 1.0, 1.0);
        assert!(green[0] < 0.01);
        assert!((green[1] - 1.0).abs() < 0.01);
        assert!(green[2] < 0.01);
    }
}

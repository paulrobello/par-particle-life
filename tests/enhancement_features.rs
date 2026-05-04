use std::f32::consts::FRAC_PI_2;
use std::fs;

use par_particle_life::app::Preset;
use par_particle_life::generators::colors::{Color, CustomPalette};
use par_particle_life::generators::positions::{PositionPattern, SpawnConfig, generate_positions};
use par_particle_life::simulation::{
    InteractionMatrix, MatrixVariationConfig, MatrixVariationMode, RadiusMatrix, SimulationConfig,
};

#[test]
fn linear_gradient_assigns_particle_type_from_x_position() {
    let config = SpawnConfig {
        num_particles: 128,
        num_types: 4,
        width: 800.0,
        height: 600.0,
    };

    let particles = generate_positions(PositionPattern::LinearGradient, &config);

    assert_eq!(particles.len(), config.num_particles);
    for particle in particles {
        let expected_type = ((particle.x / config.width) * config.num_types as f32)
            .floor()
            .min((config.num_types - 1) as f32) as u32;
        assert_eq!(particle.particle_type, expected_type);
    }
}

#[test]
fn radial_gradient_assigns_particle_type_from_center_distance() {
    let config = SpawnConfig {
        num_particles: 128,
        num_types: 4,
        width: 800.0,
        height: 600.0,
    };
    let center_x = config.width * 0.5;
    let center_y = config.height * 0.5;
    let max_radius = config.width.min(config.height) * 0.48;

    let particles = generate_positions(PositionPattern::RadialGradient, &config);

    assert_eq!(particles.len(), config.num_particles);
    for particle in particles {
        let distance = ((particle.x - center_x).powi(2) + (particle.y - center_y).powi(2)).sqrt();
        let expected_type =
            ((distance / max_radius).min(0.999_999) * config.num_types as f32).floor() as u32;
        assert_eq!(particle.particle_type, expected_type);
    }
}

#[test]
fn angular_gradient_assigns_particle_type_from_angle() {
    let config = SpawnConfig {
        num_particles: 128,
        num_types: 8,
        width: 800.0,
        height: 600.0,
    };
    let center_x = config.width * 0.5;
    let center_y = config.height * 0.5;

    let particles = generate_positions(PositionPattern::AngularGradient, &config);

    assert_eq!(particles.len(), config.num_particles);
    for particle in particles {
        let angle = (particle.y - center_y).atan2(particle.x - center_x);
        let normalized = (angle + std::f32::consts::PI) / std::f32::consts::TAU;
        let expected_type = (normalized.min(0.999_999) * config.num_types as f32).floor() as u32;
        assert_eq!(particle.particle_type, expected_type);
    }
}

#[test]
fn custom_palette_interpolates_saved_anchor_colors() {
    let red: Color = [1.0, 0.0, 0.0, 1.0];
    let blue: Color = [0.0, 0.0, 1.0, 1.0];
    let palette = CustomPalette::new("red-blue", vec![red, blue]).expect("valid palette");

    let colors = palette.colors_for_types(3);

    assert_eq!(colors.len(), 3);
    assert_eq!(colors[0], red);
    assert!((colors[1][0] - 0.5).abs() < 0.001);
    assert!((colors[1][2] - 0.5).abs() < 0.001);
    assert_eq!(colors[2], blue);
}

#[test]
fn custom_palettes_save_load_and_list_from_directory() {
    let dir = std::env::temp_dir().join(format!(
        "par-particle-life-palette-test-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create palette test dir");

    let palette = CustomPalette::new("sunset", vec![[1.0, 0.2, 0.0, 1.0], [0.2, 0.0, 0.8, 1.0]])
        .expect("valid palette");
    palette.save_to_dir(&dir).expect("save palette");

    let names = CustomPalette::list_in_dir(&dir).expect("list palettes");
    let loaded = CustomPalette::load_from_dir(&dir, "sunset").expect("load palette");

    assert_eq!(names, vec!["sunset".to_string()]);
    assert_eq!(loaded.name, palette.name);
    assert_eq!(loaded.colors, palette.colors);

    fs::remove_dir_all(&dir).expect("cleanup palette test dir");
}

#[test]
fn matrix_variation_disabled_returns_base_matrix() {
    let mut base = InteractionMatrix::new(2);
    base.set(0, 0, 0.25);
    base.set(0, 1, -0.5);
    base.set(1, 0, 0.75);
    base.set(1, 1, -0.25);

    let config = MatrixVariationConfig {
        enabled: false,
        mode: MatrixVariationMode::Oscillate,
        amplitude: 0.5,
        speed: 1.0,
    };

    let varied = config.apply(&base, FRAC_PI_2);

    assert_eq!(varied.data, base.data);
}

#[test]
fn presets_preserve_matrix_variation_phase() {
    let matrix = InteractionMatrix::filled(2, 0.25);
    let radii = RadiusMatrix::default_for_size(2);
    let config = SimulationConfig {
        num_types: 2,
        ..SimulationConfig::default()
    };
    let variation = MatrixVariationConfig {
        enabled: true,
        mode: MatrixVariationMode::Drift,
        amplitude: 0.3,
        speed: 0.8,
    };

    let preset = Preset::new(
        "phase-test",
        &config,
        &matrix,
        &radii,
        &matrix,
        variation,
        12.5,
        par_particle_life::generators::rules::RuleType::Random,
        par_particle_life::generators::colors::PaletteType::Rainbow,
        None,
        PositionPattern::Disk,
        &[1.0, 1.0],
        &[1.0, 1.0],
        &[],
    );

    assert_eq!(preset.matrix_variation_time, 12.5);
}

#[test]
fn matrix_variation_oscillates_from_base_without_mutating_it() {
    let mut base = InteractionMatrix::new(2);
    base.set(0, 0, 0.0);
    base.set(0, 1, 0.95);
    base.set(1, 0, -0.95);
    base.set(1, 1, 0.0);

    let config = MatrixVariationConfig {
        enabled: true,
        mode: MatrixVariationMode::Oscillate,
        amplitude: 0.2,
        speed: 1.0,
    };

    let varied = config.apply(&base, FRAC_PI_2);

    assert_ne!(varied.data, base.data);
    assert_eq!(base.get(0, 0), 0.0);
    assert!(varied.data.iter().all(|value| (-1.0..=1.0).contains(value)));
}

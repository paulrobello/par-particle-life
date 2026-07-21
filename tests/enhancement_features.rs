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

use par_particle_life::renderer::gpu::{ParticleView, SimParamsUniform, SimulationBuffers};
use par_particle_life::simulation::Particle;

fn request_headless_wgpu_device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    });

    let adapter = match pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
        compatible_surface: None,
        force_fallback_adapter: false,
    })) {
        Ok(adapter) => adapter,
        Err(err) => {
            eprintln!("skipping production-path GPU params test: no wgpu adapter available: {err}");
            return None;
        }
    };

    match pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("enhancement_features test device"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        memory_hints: wgpu::MemoryHints::Performance,
        ..Default::default()
    })) {
        Ok(device_queue) => Some(device_queue),
        Err(err) => {
            eprintln!("skipping production-path GPU params test: no wgpu device available: {err}");
            None
        }
    }
}

fn read_sim_params_uniform(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    buffer: &wgpu::Buffer,
) -> SimParamsUniform {
    let size = std::mem::size_of::<SimParamsUniform>() as u64;
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Simulation Params Test Readback"),
        size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Simulation Params Test Readback Encoder"),
    });
    encoder.copy_buffer_to_buffer(buffer, 0, &staging, 0, size);
    queue.submit(std::iter::once(encoder.finish()));

    let slice = staging.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = tx.send(result);
    });
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    rx.recv().unwrap().unwrap();

    let mapped = slice.get_mapped_range();
    let params = *bytemuck::from_bytes::<SimParamsUniform>(&mapped);
    drop(mapped);
    staging.unmap();
    params
}

#[test]
fn set_particle_count_preserves_existing_particles_when_growing() {
    let mut app = par_particle_life::app::App::new(true);
    app.sim_config.num_types = 4;
    app.sim_config.world_size = glam::Vec2::new(800.0, 600.0);
    app.particles = vec![
        Particle::with_velocity(10.0, 20.0, 1.0, 2.0, 0),
        Particle::with_velocity(30.0, 40.0, 3.0, 4.0, 1),
    ];
    app.sim_config.num_particles = 2;

    let original = app.particles.clone();
    app.set_particle_count_preserving_existing(5);

    assert_eq!(app.particles.len(), 5);
    assert_eq!(app.sim_config.num_particles, 5);
    assert_eq!(app.particles[0].x, original[0].x);
    assert_eq!(app.particles[0].y, original[0].y);
    assert_eq!(app.particles[0].vx, original[0].vx);
    assert_eq!(app.particles[1].particle_type, original[1].particle_type);
    for particle in &app.particles[2..] {
        assert!((0.0..=800.0).contains(&particle.x));
        assert!((0.0..=600.0).contains(&particle.y));
        assert!(particle.particle_type < 4);
        assert_eq!(particle.vx, 0.0);
        assert_eq!(particle.vy, 0.0);
    }
}

#[test]
fn set_particle_count_truncates_without_regenerating_prefix() {
    let mut app = par_particle_life::app::App::new(true);
    app.particles = vec![
        Particle::with_velocity(10.0, 20.0, 1.0, 2.0, 0),
        Particle::with_velocity(30.0, 40.0, 3.0, 4.0, 1),
        Particle::with_velocity(50.0, 60.0, 5.0, 6.0, 2),
    ];
    app.sim_config.num_particles = 3;

    app.set_particle_count_preserving_existing(2);

    assert_eq!(app.particles.len(), 2);
    assert_eq!(app.sim_config.num_particles, 2);
    assert_eq!(app.particles[0].x, 10.0);
    assert_eq!(app.particles[1].y, 40.0);
    assert_eq!(app.particles[1].vy, 4.0);
}

#[test]
fn sim_params_uniform_uses_explicit_active_particle_count() {
    let config = SimulationConfig {
        num_particles: 128,
        num_types: 3,
        ..SimulationConfig::default()
    };

    let params = par_particle_life::renderer::gpu::SimParamsUniform::from_config_with_num_particles(
        &config, 0.125, 42,
    );

    assert_eq!(params.num_particles, 42);
    assert_eq!(params.num_types, config.num_types);
    assert_eq!(params.dt, 0.125);
}

#[test]
fn simulation_buffers_production_paths_write_active_particle_count_to_uniform() {
    let Some((device, queue)) = request_headless_wgpu_device() else {
        return;
    };

    let particles = vec![
        Particle::with_velocity(10.0, 20.0, 1.0, 2.0, 0),
        Particle::with_velocity(30.0, 40.0, 3.0, 4.0, 1),
        Particle::with_velocity(50.0, 60.0, 5.0, 6.0, 2),
    ];
    let config = SimulationConfig {
        num_particles: 999,
        num_types: 3,
        ..SimulationConfig::default()
    };
    let num_types = config.num_types as usize;
    let interaction_matrix = InteractionMatrix::filled(num_types, 0.0);
    let radius_matrix = RadiusMatrix::default_for_size(num_types);
    let colors = vec![[1.0, 1.0, 1.0, 1.0]; num_types];
    let type_masses = vec![1.0; num_types];
    let type_sizes = vec![1.0; num_types];

    let view = ParticleView {
        particles: &particles,
        interaction_matrix: &interaction_matrix,
        radius_matrix: &radius_matrix,
        colors: &colors,
        type_masses: &type_masses,
        type_sizes: &type_sizes,
    };
    let mut buffers = SimulationBuffers::new(&device, &view, &config);

    let initial_params = read_sim_params_uniform(&device, &queue, &buffers.params);
    assert_eq!(initial_params.num_particles, 3);

    buffers.set_num_particles(2);
    buffers.update_params(&queue, &config, 1.0 / 60.0);
    let updated_params = read_sim_params_uniform(&device, &queue, &buffers.params);
    assert_eq!(updated_params.num_particles, 2);

    assert!(buffers.has_capacity_for(128_000));
    let over_capacity = buffers.capacity_particles + 1;
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        buffers.set_num_particles(over_capacity);
    }));
    std::panic::set_hook(previous_hook);
    assert!(panic_result.is_err());
}

#[test]
fn particle_buffer_capacity_has_ui_hot_swap_headroom() {
    assert_eq!(
        par_particle_life::renderer::gpu::SimulationBuffers::capacity_for_particle_count(1_000),
        128_000
    );
    assert_eq!(
        par_particle_life::renderer::gpu::SimulationBuffers::capacity_for_particle_count(64_000),
        128_000
    );
    assert_eq!(
        par_particle_life::renderer::gpu::SimulationBuffers::capacity_for_particle_count(200_000),
        200_000
    );
}

#[test]
fn particle_range_byte_offsets_match_soa_layout() {
    assert_eq!(
        par_particle_life::renderer::gpu::SimulationBuffers::pos_type_byte_offset(3),
        (3 * std::mem::size_of::<par_particle_life::simulation::ParticlePosType>()) as u64
    );
    assert_eq!(
        par_particle_life::renderer::gpu::SimulationBuffers::velocity_byte_offset_f32(3),
        (3 * std::mem::size_of::<par_particle_life::simulation::ParticleVel>()) as u64
    );
    assert_eq!(
        par_particle_life::renderer::gpu::SimulationBuffers::velocity_byte_offset_f16(3),
        (3 * std::mem::size_of::<par_particle_life::simulation::ParticleVelHalf>()) as u64
    );
}

//! Physics benchmarks.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use par_particle_life::simulation::{
    InteractionMatrix, Particle, RadiusMatrix, SimulationConfig, compute_forces_cpu,
};

fn make_particles(n: usize, num_types: usize) -> Vec<Particle> {
    use rand::Rng;
    let mut rng = rand::rng();
    (0..n)
        .map(|_| {
            Particle::new(
                rng.random::<f32>() * 1000.0,
                rng.random::<f32>() * 1000.0,
                rng.random_range(0..num_types as u32),
            )
        })
        .collect()
}

fn benchmark_force_calculation(c: &mut Criterion) {
    let num_types = 7;
    let particles = make_particles(1000, num_types);
    let matrix = InteractionMatrix::new(num_types);
    let radii = RadiusMatrix::default_for_size(num_types);
    let config = SimulationConfig::default();

    c.bench_function("compute_forces_cpu_1000", |b| {
        b.iter(|| {
            compute_forces_cpu(
                black_box(&particles),
                black_box(&matrix),
                black_box(&radii),
                black_box(&config),
            )
        })
    });
}

criterion_group!(benches, benchmark_force_calculation);
criterion_main!(benches);

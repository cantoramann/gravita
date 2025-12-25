#![allow(missing_docs)]
//! Benchmarks for gravita-physics simulation.
//!
//! Run with: `cargo bench -p gravita-physics`

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use gravita_math::{Circle, Vec2};
use gravita_physics::{
    CollisionShape, PhysicsWorld, RigidBody,
    integrator::{Integrator, SemiImplicitEuler, Verlet},
};
use rand::Rng;

// ============================================================================
// Helper Functions
// ============================================================================

fn create_circle_body(id: usize, position: Vec2, radius: f32) -> RigidBody {
    let shape = CollisionShape::Circle(Circle::new(Vec2::ZERO, radius));
    RigidBody::new(id, shape)
        .with_position(position)
        .with_density(1.0)
}

fn create_random_bodies(count: usize, area_size: f32) -> Vec<RigidBody> {
    let mut rng = rand::thread_rng();
    (0..count)
        .map(|i| {
            let x = rng.gen_range(0.0..area_size);
            let y = rng.gen_range(0.0..area_size);
            let radius = rng.gen_range(5.0..15.0);
            create_circle_body(i, Vec2::new(x, y), radius)
        })
        .collect()
}

fn create_physics_world_with_bodies(count: usize) -> PhysicsWorld {
    let mut world = PhysicsWorld::new();
    let bodies = create_random_bodies(count, 1000.0);
    for body in bodies {
        world.add_body(body);
    }
    world
}

// ============================================================================
// Integrator Benchmarks
// ============================================================================

fn bench_integrators(c: &mut Criterion) {
    let mut group = c.benchmark_group("Integrators");

    let shape = CollisionShape::Circle(Circle::new(Vec2::ZERO, 10.0));

    // Semi-implicit Euler (single body)
    group.bench_function("semi_implicit_euler_single", |b| {
        let mut integrator = SemiImplicitEuler;
        let mut body = RigidBody::new(0, shape).with_velocity(Vec2::new(100.0, 50.0));
        body.apply_force(Vec2::new(0.0, -98.1)); // Gravity

        b.iter(|| {
            let mut body_copy = body.clone();
            integrator.integrate_velocity(&mut body_copy, black_box(0.016));
            integrator.integrate_position(&mut body_copy, black_box(0.016));
            body_copy
        });
    });

    // Verlet (single body)
    group.bench_function("verlet_single", |b| {
        let mut integrator = Verlet::new();
        let mut body = RigidBody::new(0, shape).with_velocity(Vec2::new(100.0, 50.0));
        body.apply_force(Vec2::new(0.0, -98.1));

        b.iter(|| {
            let mut body_copy = body.clone();
            integrator.integrate_velocity(&mut body_copy, black_box(0.016));
            integrator.integrate_position(&mut body_copy, black_box(0.016));
            body_copy
        });
    });

    group.finish();
}

// ============================================================================
// World Step Benchmarks
// ============================================================================

fn bench_world_step(c: &mut Criterion) {
    let mut group = c.benchmark_group("WorldStep");

    for body_count in [10, 50, 100, 200, 500].iter() {
        group.throughput(Throughput::Elements(*body_count as u64));

        group.bench_with_input(
            BenchmarkId::new("step", body_count),
            body_count,
            |b, &count| {
                let mut world = create_physics_world_with_bodies(count);
                b.iter(|| {
                    world.step(black_box(0.016));
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Body Operations Benchmarks
// ============================================================================

fn bench_body_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("RigidBody");

    let shape = CollisionShape::Circle(Circle::new(Vec2::ZERO, 10.0));
    let body = RigidBody::new(0, shape)
        .with_position(Vec2::new(100.0, 100.0))
        .with_velocity(Vec2::new(50.0, -20.0));

    group.bench_function("get_world_aabb", |b| {
        b.iter(|| black_box(&body).get_world_aabb());
    });

    group.bench_function("velocity_at_point", |b| {
        let point = Vec2::new(110.0, 100.0);
        b.iter(|| black_box(&body).get_velocity_at_point(black_box(point)));
    });

    group.bench_function("apply_force", |b| {
        b.iter(|| {
            let mut body_copy = body.clone();
            body_copy.apply_force(black_box(Vec2::new(100.0, -50.0)));
            body_copy
        });
    });

    group.bench_function("apply_impulse", |b| {
        b.iter(|| {
            let mut body_copy = body.clone();
            body_copy.apply_impulse(black_box(Vec2::new(10.0, -5.0)));
            body_copy
        });
    });

    group.bench_function("apply_impulse_at_point", |b| {
        let point = Vec2::new(110.0, 100.0);
        let impulse = Vec2::new(10.0, -5.0);
        b.iter(|| {
            let mut body_copy = body.clone();
            body_copy.apply_impulse_at_point(black_box(impulse), black_box(point));
            body_copy
        });
    });

    group.finish();
}

// ============================================================================
// Scaling Benchmarks
// ============================================================================

fn bench_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("Scaling");
    group.sample_size(50); // Fewer samples for expensive benchmarks

    // Test how simulation time scales with body count
    for body_count in [50, 100, 200, 500, 1000].iter() {
        group.throughput(Throughput::Elements(*body_count as u64));

        group.bench_with_input(
            BenchmarkId::new("10_steps", body_count),
            body_count,
            |b, &count| {
                let mut world = create_physics_world_with_bodies(count);
                b.iter(|| {
                    for _ in 0..10 {
                        world.step(black_box(0.016));
                    }
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_integrators,
    bench_world_step,
    bench_body_operations,
    bench_scaling,
);

criterion_main!(benches);

#![allow(missing_docs)]
//! Benchmarks for `gravita-physics-3d`.
//!
//! Run with: `cargo bench -p gravita-physics-3d`
//!
//! Coverage:
//! - `world.step` scaling 50/100/500 bodies (catches solver regressions).
//! - Simple vs spatial-hash detector at scale (justifies the broad phase).
//! - SAT OBB-OBB narrow-phase cost.

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use gravita_math::{Aabb3, Obb, Quat, Sphere, Vec3};
use gravita_physics_3d::{
    BodyType, CollisionShape, PhysicsWorld, RigidBody, SimpleCollisionDetector,
    SpatialHashDetector, collision::test_obb_obb,
};

fn build_world(body_count: usize) -> PhysicsWorld {
    let mut world = PhysicsWorld::new();
    world.add_body(
        RigidBody::new(
            0,
            CollisionShape::Aabb(Aabb3::from_center_size(
                Vec3::new(0.0, -1.0, 0.0),
                Vec3::new(100.0, 1.0, 100.0),
            )),
        )
        .with_type(BodyType::Static),
    );
    let side = (body_count as f32).sqrt().ceil() as i32;
    for i in 0..body_count as i32 {
        let x = ((i % side) - side / 2) as f32 * 1.2;
        let z = ((i / side) - side / 2) as f32 * 1.2;
        let y = (i as f32).mul_add(0.05, 5.0);
        world.add_body(
            RigidBody::new(0, CollisionShape::Sphere(Sphere::new(Vec3::ZERO, 0.5)))
                .with_position(Vec3::new(x, y, z))
                .with_density(1.0)
                .with_restitution(0.5),
        );
    }
    world
}

fn build_bodies_for_detector(body_count: usize) -> Vec<RigidBody> {
    let mut bodies = Vec::with_capacity(body_count);
    let side = (body_count as f32).sqrt().ceil() as i32;
    for i in 0..body_count as i32 {
        let x = ((i % side) - side / 2) as f32 * 1.2;
        let z = ((i / side) - side / 2) as f32 * 1.2;
        bodies.push(
            RigidBody::new(0, CollisionShape::Sphere(Sphere::new(Vec3::ZERO, 0.4)))
                .with_position(Vec3::new(x, 0.0, z)),
        );
    }
    bodies
}

fn bench_world_step(c: &mut Criterion) {
    let mut group = c.benchmark_group("world.step");
    for &n in &[50usize, 100, 500] {
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            let mut world = build_world(n);
            b.iter(|| world.step(1.0 / 60.0));
        });
    }
    group.finish();
}

fn bench_detectors(c: &mut Criterion) {
    let mut group = c.benchmark_group("detector");
    for &n in &[50usize, 200, 500] {
        let bodies = build_bodies_for_detector(n);

        group.bench_with_input(BenchmarkId::new("simple", n), &bodies, |b, bodies| {
            let mut contacts = Vec::new();
            b.iter(|| {
                contacts.clear();
                SimpleCollisionDetector::detect(black_box(bodies), &mut contacts);
            });
        });

        group.bench_with_input(BenchmarkId::new("spatial_hash", n), &bodies, |b, bodies| {
            let mut detector = SpatialHashDetector::new(2.0);
            let mut contacts = Vec::new();
            b.iter(|| {
                contacts.clear();
                detector.detect(black_box(bodies), &mut contacts);
            });
        });
    }
    group.finish();
}

fn bench_obb_sat(c: &mut Criterion) {
    let mut group = c.benchmark_group("narrow_phase");
    let a = Obb::new(Vec3::ZERO, Vec3::splat(1.0), Quat::IDENTITY);
    let b = Obb::new(
        Vec3::new(1.5, 0.5, 0.0),
        Vec3::splat(1.0),
        Quat::from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_4),
    );
    group.bench_function("obb_obb_overlap", |bench| {
        bench.iter(|| test_obb_obb(black_box(&a), black_box(&b), 0, 1));
    });
    let b_far = Obb::new(Vec3::new(20.0, 0.0, 0.0), Vec3::splat(1.0), Quat::IDENTITY);
    group.bench_function("obb_obb_no_overlap", |bench| {
        bench.iter(|| test_obb_obb(black_box(&a), black_box(&b_far), 0, 1));
    });
    group.finish();
}

criterion_group!(benches, bench_world_step, bench_detectors, bench_obb_sat);
criterion_main!(benches);

#![allow(missing_docs)]
//! Benchmarks for gravita-physics collision detection.
//!
//! Run with: `cargo bench -p gravita-physics -- collision`

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use gravita_math::{Aabb, Circle, Vec2};
use gravita_physics::{
    CollisionShape, RigidBody,
    collision::{
        CollisionDetector, SimpleCollisionDetector, SpatialHashDetector,
        narrow_phase::{test_aabb_aabb, test_circle_aabb, test_circle_circle},
    },
};
use rand::Rng;

// ============================================================================
// Helper Functions
// ============================================================================

fn create_random_circle_bodies(count: usize, area_size: f32) -> Vec<RigidBody> {
    let mut rng = rand::thread_rng();
    (0..count)
        .map(|i| {
            let x = rng.gen_range(0.0..area_size);
            let y = rng.gen_range(0.0..area_size);
            let radius = rng.gen_range(5.0..20.0);
            let shape = CollisionShape::Circle(Circle::new(Vec2::ZERO, radius));
            RigidBody::new(i, shape).with_position(Vec2::new(x, y))
        })
        .collect()
}

fn create_random_aabb_bodies(count: usize, area_size: f32) -> Vec<RigidBody> {
    let mut rng = rand::thread_rng();
    (0..count)
        .map(|i| {
            let x = rng.gen_range(0.0..area_size);
            let y = rng.gen_range(0.0..area_size);
            let w = rng.gen_range(10.0..30.0);
            let h = rng.gen_range(10.0..30.0);
            let shape = CollisionShape::Aabb(Aabb::new(
                Vec2::new(-w / 2.0, -h / 2.0),
                Vec2::new(w / 2.0, h / 2.0),
            ));
            RigidBody::new(i, shape).with_position(Vec2::new(x, y))
        })
        .collect()
}

// ============================================================================
// Narrow Phase Benchmarks
// ============================================================================

fn bench_narrow_phase(c: &mut Criterion) {
    let mut group = c.benchmark_group("NarrowPhase");

    // Circle-Circle collision test
    let c1 = Circle::new(Vec2::new(0.0, 0.0), 10.0);
    let c2 = Circle::new(Vec2::new(15.0, 0.0), 10.0); // Overlapping

    group.bench_function("circle_circle_hit", |b| {
        b.iter(|| test_circle_circle(&black_box(c1), &black_box(c2), 0, 1));
    });

    let c3 = Circle::new(Vec2::new(30.0, 0.0), 10.0); // Not overlapping
    group.bench_function("circle_circle_miss", |b| {
        b.iter(|| test_circle_circle(&black_box(c1), &black_box(c3), 0, 1));
    });

    // Aabb-Aabb collision test
    let a1 = Aabb::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
    let a2 = Aabb::new(Vec2::new(5.0, 5.0), Vec2::new(15.0, 15.0)); // Overlapping

    group.bench_function("aabb_aabb_hit", |b| {
        b.iter(|| test_aabb_aabb(&black_box(a1), &black_box(a2), 0, 1));
    });

    let a3 = Aabb::new(Vec2::new(20.0, 20.0), Vec2::new(30.0, 30.0)); // Not overlapping
    group.bench_function("aabb_aabb_miss", |b| {
        b.iter(|| test_aabb_aabb(&black_box(a1), &black_box(a3), 0, 1));
    });

    // Circle-Aabb collision test
    let circle = Circle::new(Vec2::new(12.0, 5.0), 5.0);
    let aabb = Aabb::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));

    group.bench_function("circle_aabb_hit", |b| {
        b.iter(|| test_circle_aabb(&black_box(circle), &black_box(aabb), 0, 1));
    });

    let circle_far = Circle::new(Vec2::new(30.0, 5.0), 5.0);
    group.bench_function("circle_aabb_miss", |b| {
        b.iter(|| test_circle_aabb(&black_box(circle_far), &black_box(aabb), 0, 1));
    });

    group.finish();
}

// ============================================================================
// Broad Phase Benchmarks
// ============================================================================

fn bench_broad_phase_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("BroadPhase");
    group.sample_size(50);

    for body_count in [50, 100, 200, 500].iter() {
        let bodies = create_random_circle_bodies(*body_count, 500.0);

        group.throughput(Throughput::Elements(*body_count as u64));

        // Simple O(n²) detector
        group.bench_with_input(
            BenchmarkId::new("simple", body_count),
            &bodies,
            |b, bodies| {
                let mut detector = SimpleCollisionDetector;
                let mut contacts = Vec::new();
                b.iter(|| {
                    contacts.clear();
                    detector.detect(black_box(bodies), &mut contacts);
                    contacts.len()
                });
            },
        );

        // Spatial hash detector
        group.bench_with_input(
            BenchmarkId::new("spatial_hash", body_count),
            &bodies,
            |b, bodies| {
                let mut detector = SpatialHashDetector::new(50.0);
                let mut contacts = Vec::new();
                b.iter(|| {
                    contacts.clear();
                    detector.detect(black_box(bodies), &mut contacts);
                    contacts.len()
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Collision Detection Scaling
// ============================================================================

fn bench_collision_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("CollisionScaling");
    group.sample_size(30);

    // Test how collision detection scales with density
    for body_count in [100, 200, 400, 800].iter() {
        let area_size = 200.0; // Fixed area, increasing density
        let bodies = create_random_circle_bodies(*body_count, area_size);

        group.throughput(Throughput::Elements(*body_count as u64));

        group.bench_with_input(
            BenchmarkId::new("dense_simple", body_count),
            &bodies,
            |b, bodies| {
                let mut detector = SimpleCollisionDetector;
                let mut contacts = Vec::new();
                b.iter(|| {
                    contacts.clear();
                    detector.detect(black_box(bodies), &mut contacts);
                    contacts.len()
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("dense_spatial_hash", body_count),
            &bodies,
            |b, bodies| {
                let mut detector = SpatialHashDetector::new(30.0);
                let mut contacts = Vec::new();
                b.iter(|| {
                    contacts.clear();
                    detector.detect(black_box(bodies), &mut contacts);
                    contacts.len()
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Mixed Shape Benchmarks
// ============================================================================

fn bench_mixed_shapes(c: &mut Criterion) {
    let mut group = c.benchmark_group("MixedShapes");
    group.sample_size(50);

    for body_count in [50, 100, 200].iter() {
        let half = *body_count / 2;
        let mut bodies = create_random_circle_bodies(half, 500.0);
        let mut aabb_bodies = create_random_aabb_bodies(*body_count - half, 500.0);

        // Renumber Aabb bodies
        for (i, body) in aabb_bodies.iter_mut().enumerate() {
            body.id = half + i;
        }
        bodies.append(&mut aabb_bodies);

        group.throughput(Throughput::Elements(*body_count as u64));

        group.bench_with_input(
            BenchmarkId::new("mixed_simple", body_count),
            &bodies,
            |b, bodies| {
                let mut detector = SimpleCollisionDetector;
                let mut contacts = Vec::new();
                b.iter(|| {
                    contacts.clear();
                    detector.detect(black_box(bodies), &mut contacts);
                    contacts.len()
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("mixed_spatial_hash", body_count),
            &bodies,
            |b, bodies| {
                let mut detector = SpatialHashDetector::new(50.0);
                let mut contacts = Vec::new();
                b.iter(|| {
                    contacts.clear();
                    detector.detect(black_box(bodies), &mut contacts);
                    contacts.len()
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_narrow_phase,
    bench_broad_phase_comparison,
    bench_collision_scaling,
    bench_mixed_shapes,
);

criterion_main!(benches);

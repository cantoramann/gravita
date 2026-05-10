#![allow(missing_docs)]
//! Benchmarks for gravita-math operations.
//!
//! Run with: `cargo bench -p gravita-math`

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use gravita_math::{
    Aabb, Aabb3, Circle, Obb, PI, Quat, Ray2D, Sphere, Transform2D, Transform3D, Vec2, Vec3,
    lerp, smooth_step,
};

// ============================================================================
// Vector Operations
// ============================================================================

fn bench_vec2_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("Vec2");

    let v1 = Vec2::new(3.0, 4.0);
    let v2 = Vec2::new(1.0, 2.0);

    group.bench_function("add", |b| b.iter(|| black_box(v1) + black_box(v2)));
    group.bench_function("sub", |b| b.iter(|| black_box(v1) - black_box(v2)));
    group.bench_function("scale", |b| b.iter(|| black_box(v1) * black_box(2.5)));
    group.bench_function("dot", |b| b.iter(|| black_box(v1).dot(black_box(v2))));
    group.bench_function("cross", |b| b.iter(|| black_box(v1).cross(black_box(v2))));
    group.bench_function("length", |b| b.iter(|| black_box(v1).length()));
    group.bench_function("length_squared", |b| {
        b.iter(|| black_box(v1).length_squared());
    });
    group.bench_function("normalize", |b| b.iter(|| black_box(v1).normalize()));
    group.bench_function("distance", |b| {
        b.iter(|| black_box(v1).distance(black_box(v2)));
    });
    group.bench_function("rotate", |b| {
        let angle = PI / 4.0;
        b.iter(|| black_box(v1).rotate(black_box(angle)));
    });
    group.bench_function("from_angle", |b| {
        let angle = PI / 4.0;
        b.iter(|| Vec2::from_angle(black_box(angle)));
    });
    group.bench_function("lerp", |b| {
        b.iter(|| black_box(v1).lerp(black_box(v2), black_box(0.5)));
    });
    group.bench_function("reflect", |b| {
        let normal = Vec2::UP;
        b.iter(|| black_box(v1).reflect(black_box(normal)));
    });

    group.finish();
}

// ============================================================================
// Utility Functions
// ============================================================================

fn bench_utility_functions(c: &mut Criterion) {
    let mut group = c.benchmark_group("Utility");

    group.bench_function("lerp_f32", |b| {
        b.iter(|| lerp(black_box(0.0), black_box(100.0), black_box(0.5)));
    });
    group.bench_function("smooth_step", |b| {
        b.iter(|| smooth_step(black_box(0.0), black_box(1.0), black_box(0.5)));
    });

    group.finish();
}

// ============================================================================
// Transform Operations
// ============================================================================

fn bench_transform_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("Transform2D");

    let t1 = Transform2D::new(Vec2::new(10.0, 20.0), PI / 4.0, Vec2::ONE);
    let t2 = Transform2D::new(Vec2::new(5.0, 10.0), PI / 6.0, Vec2::ONE);
    let point = Vec2::new(1.0, 0.0);

    group.bench_function("transform_point", |b| {
        b.iter(|| black_box(t1).transform_point(black_box(point)));
    });
    group.bench_function("combine", |b| {
        b.iter(|| black_box(t1).combine(&black_box(t2)));
    });

    group.finish();
}

// ============================================================================
// Aabb Operations
// ============================================================================

fn bench_aabb_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("Aabb");

    let aabb1 = Aabb::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
    let aabb2 = Aabb::new(Vec2::new(5.0, 5.0), Vec2::new(15.0, 15.0));
    let point = Vec2::new(5.0, 5.0);

    group.bench_function("contains_point", |b| {
        b.iter(|| black_box(aabb1).contains_point(black_box(point)));
    });
    group.bench_function("intersects", |b| {
        b.iter(|| black_box(aabb1).intersects(&black_box(aabb2)));
    });
    group.bench_function("merge", |b| {
        b.iter(|| black_box(aabb1).merge(&black_box(aabb2)));
    });
    group.bench_function("center", |b| b.iter(|| black_box(aabb1).center()));
    group.bench_function("size", |b| b.iter(|| black_box(aabb1).size()));

    group.finish();
}

// ============================================================================
// Circle Operations
// ============================================================================

fn bench_circle_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("Circle");

    let c1 = Circle::new(Vec2::new(0.0, 0.0), 10.0);
    let c2 = Circle::new(Vec2::new(15.0, 0.0), 10.0);
    let point = Vec2::new(5.0, 0.0);

    group.bench_function("contains_point", |b| {
        b.iter(|| black_box(c1).contains_point(black_box(point)));
    });
    group.bench_function("intersects_circle", |b| {
        b.iter(|| black_box(c1).intersects_circle(&black_box(c2)));
    });
    group.bench_function("to_aabb", |b| b.iter(|| black_box(c1).to_aabb()));

    group.finish();
}

// ============================================================================
// Ray Operations
// ============================================================================

fn bench_ray_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("Ray2D");

    let ray = Ray2D::new(Vec2::new(-20.0, 5.0), Vec2::RIGHT);
    let aabb = Aabb::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
    let circle = Circle::new(Vec2::new(0.0, 5.0), 5.0);

    group.bench_function("cast_aabb", |b| {
        b.iter(|| black_box(ray).cast_aabb(&black_box(aabb)));
    });
    group.bench_function("cast_circle", |b| {
        b.iter(|| black_box(ray).cast_circle(&black_box(circle)));
    });
    group.bench_function("point_at", |b| {
        b.iter(|| black_box(ray).point_at(black_box(10.0)));
    });

    group.finish();
}

// ============================================================================
// Batch Operations (Throughput)
// ============================================================================

fn bench_batch_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("Batch");

    for size in [100, 1000, 10000].iter() {
        let vectors: Vec<Vec2> = (0..*size)
            .map(|i| Vec2::new(i as f32, (i * 2) as f32))
            .collect();

        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(
            BenchmarkId::new("normalize_all", size),
            &vectors,
            |b, vecs| {
                b.iter(|| {
                    vecs.iter()
                        .map(gravita_math::Vec2::normalize)
                        .collect::<Vec<_>>()
                });
            },
        );

        group.bench_with_input(BenchmarkId::new("sum_all", size), &vectors, |b, vecs| {
            b.iter(|| vecs.iter().fold(Vec2::ZERO, |acc, v| acc + *v));
        });
    }

    group.finish();
}

// ============================================================================
// 3D — Vec3, Quat, Transform3D, Aabb3, Sphere, Obb
// ============================================================================

fn bench_vec3(c: &mut Criterion) {
    let mut group = c.benchmark_group("Vec3");
    let a = Vec3::new(1.0, 2.0, 3.0);
    let b = Vec3::new(4.0, -2.0, 0.5);
    group.bench_function("dot", |bench| bench.iter(|| black_box(a).dot(black_box(b))));
    group.bench_function("cross", |bench| {
        bench.iter(|| black_box(a).cross(black_box(b)));
    });
    group.bench_function("length", |bench| bench.iter(|| black_box(a).length()));
    group.bench_function("normalize", |bench| {
        bench.iter(|| black_box(a).normalize());
    });
    group.bench_function("distance", |bench| {
        bench.iter(|| black_box(a).distance(black_box(b)));
    });
    group.bench_function("lerp", |bench| {
        bench.iter(|| black_box(a).lerp(black_box(b), black_box(0.4)));
    });
    group.finish();
}

fn bench_quat(c: &mut Criterion) {
    let mut group = c.benchmark_group("Quat");
    let q = Quat::from_axis_angle(Vec3::Y, PI / 4.0);
    let r = Quat::from_axis_angle(Vec3::X, PI / 6.0);
    let v = Vec3::new(1.0, 2.0, 3.0);
    group.bench_function("from_axis_angle", |bench| {
        bench.iter(|| Quat::from_axis_angle(black_box(Vec3::Y), black_box(PI / 4.0)));
    });
    group.bench_function("mul", |bench| bench.iter(|| black_box(q) * black_box(r)));
    group.bench_function("rotate_vec", |bench| {
        bench.iter(|| black_box(q).rotate_vec(black_box(v)));
    });
    group.bench_function("inverse", |bench| bench.iter(|| black_box(q).inverse()));
    group.bench_function("normalize", |bench| bench.iter(|| black_box(q).normalize()));
    group.finish();
}

fn bench_transform3d(c: &mut Criterion) {
    let mut group = c.benchmark_group("Transform3D");
    let t = Transform3D::IDENTITY
        .with_position(Vec3::new(1.0, 2.0, 3.0))
        .with_rotation(Quat::from_axis_angle(Vec3::Y, PI / 4.0))
        .with_scale(Vec3::new(2.0, 1.0, 1.0));
    let p = Vec3::new(0.5, 0.5, 0.5);
    group.bench_function("transform_point", |bench| {
        bench.iter(|| black_box(t).transform_point(black_box(p)));
    });
    group.bench_function("to_matrix", |bench| {
        bench.iter(|| black_box(t).to_matrix());
    });
    group.bench_function("combine", |bench| {
        bench.iter(|| black_box(t).combine(&black_box(t)));
    });
    group.finish();
}

fn bench_aabb3_sphere_obb(c: &mut Criterion) {
    let mut group = c.benchmark_group("Geometry3D");
    let a = Aabb3::new(Vec3::ZERO, Vec3::splat(2.0));
    let b = Aabb3::new(Vec3::splat(1.0), Vec3::splat(3.0));
    let p = Vec3::new(0.5, 0.5, 0.5);
    group.bench_function("aabb3_intersects", |bench| {
        bench.iter(|| black_box(a).intersects(&black_box(b)));
    });
    group.bench_function("aabb3_closest_point", |bench| {
        bench.iter(|| black_box(a).closest_point(black_box(p)));
    });

    let s1 = Sphere::new(Vec3::ZERO, 1.0);
    let s2 = Sphere::new(Vec3::splat(1.5), 1.0);
    group.bench_function("sphere_intersects", |bench| {
        bench.iter(|| black_box(s1).intersects_sphere(&black_box(s2)));
    });
    group.bench_function("sphere_aabb_intersects", |bench| {
        bench.iter(|| black_box(s1).intersects_aabb(&black_box(a)));
    });

    let o = Obb::new(Vec3::ZERO, Vec3::splat(1.0), Quat::IDENTITY);
    group.bench_function("obb_closest_point", |bench| {
        bench.iter(|| black_box(o).closest_point(black_box(p)));
    });
    group.bench_function("obb_corners", |bench| bench.iter(|| black_box(o).corners()));
    group.finish();
}

criterion_group!(
    benches,
    bench_vec2_operations,
    bench_utility_functions,
    bench_transform_operations,
    bench_aabb_operations,
    bench_circle_operations,
    bench_ray_operations,
    bench_batch_operations,
    bench_vec3,
    bench_quat,
    bench_transform3d,
    bench_aabb3_sphere_obb,
);

criterion_main!(benches);

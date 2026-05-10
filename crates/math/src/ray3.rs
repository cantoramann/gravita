// math/src/ray3.rs

use crate::{aabb3::Aabb3, sphere::Sphere, vector3::Vec3};

/// 3D ray with an origin and a (typically unit-length) direction.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Ray3D {
    /// Starting point.
    pub origin: Vec3,
    /// Direction. Callers usually pass a unit vector — `intersect_aabb` and
    /// `intersect_sphere` are parameterised by `t` in the ray's own units, so
    /// non-unit directions still produce meaningful `t` values, just scaled.
    pub direction: Vec3,
}

/// Result of a successful 3D raycast.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct RayHit3D {
    /// Parameter `t` along the ray where the hit occurs (`origin + direction * t`).
    pub t: f32,
    /// World-space hit point.
    pub point: Vec3,
    /// Outward-facing surface normal at the hit point.
    pub normal: Vec3,
}

impl Ray3D {
    /// Build a new ray.
    #[inline]
    pub const fn new(origin: Vec3, direction: Vec3) -> Self {
        Self { origin, direction }
    }

    /// Evaluate `origin + direction * t`.
    #[inline]
    pub fn at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }

    /// Slab method for ray vs AABB. Returns the entry point if any positive
    /// `t` lies inside the box.
    pub fn intersect_aabb(&self, aabb: &Aabb3) -> Option<RayHit3D> {
        // Slab on each axis. Track the latest entry and the earliest exit.
        let mut t_enter = f32::NEG_INFINITY;
        let mut t_exit = f32::INFINITY;
        let mut enter_axis = 0u8;

        for axis in 0..3 {
            let (o, d, lo, hi) = match axis {
                0 => (self.origin.x, self.direction.x, aabb.min.x, aabb.max.x),
                1 => (self.origin.y, self.direction.y, aabb.min.y, aabb.max.y),
                _ => (self.origin.z, self.direction.z, aabb.min.z, aabb.max.z),
            };

            if d.abs() < 1e-8 {
                // Ray parallel to slab — must already be inside.
                if o < lo || o > hi {
                    return None;
                }
                continue;
            }

            let inv = 1.0 / d;
            let mut t1 = (lo - o) * inv;
            let mut t2 = (hi - o) * inv;
            if t1 > t2 {
                std::mem::swap(&mut t1, &mut t2);
            }
            if t1 > t_enter {
                t_enter = t1;
                enter_axis = axis as u8;
            }
            if t2 < t_exit {
                t_exit = t2;
            }
            if t_enter > t_exit {
                return None;
            }
        }

        let t = if t_enter >= 0.0 { t_enter } else { return None };
        let point = self.at(t);
        let normal = match enter_axis {
            0 => Vec3::new(-self.direction.x.signum(), 0.0, 0.0),
            1 => Vec3::new(0.0, -self.direction.y.signum(), 0.0),
            _ => Vec3::new(0.0, 0.0, -self.direction.z.signum()),
        };
        Some(RayHit3D { t, point, normal })
    }

    /// Ray vs sphere using the quadratic formula. Returns the entry point of
    /// the smaller non-negative `t`.
    pub fn intersect_sphere(&self, sphere: &Sphere) -> Option<RayHit3D> {
        let oc = self.origin - sphere.center;
        let a = self.direction.dot(self.direction);
        let b = oc.dot(self.direction);
        let c = sphere.radius.mul_add(-sphere.radius, oc.dot(oc));
        let disc = b.mul_add(b, -(a * c));
        if disc < 0.0 {
            return None;
        }
        let sqrt_d = disc.sqrt();
        let t0 = (-b - sqrt_d) / a;
        let t1 = (-b + sqrt_d) / a;
        let t = if t0 >= 0.0 {
            t0
        } else if t1 >= 0.0 {
            t1
        } else {
            return None;
        };
        let point = self.at(t);
        let normal = (point - sphere.center).normalize();
        Some(RayHit3D { t, point, normal })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1e-5;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    fn vec_approx(a: Vec3, b: Vec3) -> bool {
        approx(a.x, b.x) && approx(a.y, b.y) && approx(a.z, b.z)
    }

    #[test]
    fn at_parameterises_along_direction() {
        let r = Ray3D::new(Vec3::ZERO, Vec3::X);
        assert_eq!(r.at(0.0), Vec3::ZERO);
        assert_eq!(r.at(5.0), Vec3::new(5.0, 0.0, 0.0));
    }

    #[test]
    fn aabb_hit_from_outside_returns_entry_point() {
        let r = Ray3D::new(Vec3::new(-5.0, 0.5, 0.5), Vec3::X);
        let a = Aabb3::new(Vec3::ZERO, Vec3::splat(1.0));
        let hit = r.intersect_aabb(&a).unwrap();
        assert!(approx(hit.t, 5.0));
        assert!(vec_approx(hit.point, Vec3::new(0.0, 0.5, 0.5)));
        assert!(vec_approx(hit.normal, -Vec3::X));
    }

    #[test]
    fn aabb_miss_returns_none() {
        let r = Ray3D::new(Vec3::new(-5.0, 5.0, 5.0), Vec3::X);
        let a = Aabb3::new(Vec3::ZERO, Vec3::splat(1.0));
        assert!(r.intersect_aabb(&a).is_none());
    }

    #[test]
    fn aabb_behind_ray_returns_none() {
        let r = Ray3D::new(Vec3::new(5.0, 0.5, 0.5), Vec3::X);
        let a = Aabb3::new(Vec3::ZERO, Vec3::splat(1.0));
        // Box is behind the ray origin; no positive-t entry.
        assert!(r.intersect_aabb(&a).is_none());
    }

    #[test]
    fn sphere_hit_returns_normal_at_surface() {
        let r = Ray3D::new(Vec3::new(-10.0, 0.0, 0.0), Vec3::X);
        let s = Sphere::new(Vec3::ZERO, 1.0);
        let hit = r.intersect_sphere(&s).unwrap();
        assert!(approx(hit.t, 9.0));
        assert!(vec_approx(hit.point, Vec3::new(-1.0, 0.0, 0.0)));
        assert!(vec_approx(hit.normal, -Vec3::X));
    }

    #[test]
    fn sphere_miss_returns_none() {
        let r = Ray3D::new(Vec3::new(-10.0, 5.0, 0.0), Vec3::X);
        let s = Sphere::new(Vec3::ZERO, 1.0);
        assert!(r.intersect_sphere(&s).is_none());
    }

    #[test]
    fn sphere_origin_inside_returns_exit_hit() {
        let r = Ray3D::new(Vec3::ZERO, Vec3::X);
        let s = Sphere::new(Vec3::ZERO, 1.0);
        let hit = r.intersect_sphere(&s).unwrap();
        assert!(approx(hit.t, 1.0));
        assert!(vec_approx(hit.point, Vec3::X));
    }
}

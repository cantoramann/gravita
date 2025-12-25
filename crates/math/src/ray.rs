// math/src/ray.rs
use crate::{aabb::AABB, circle::Circle, vector2::Vec2};

/// Infinite ray in 2D, defined by origin and direction.
#[derive(Debug, Copy, Clone)]
pub struct Ray2D {
    /// Ray origin point.
    pub origin: Vec2,
    /// Ray direction (normalized on construction).
    pub direction: Vec2,
}

/// Result of a ray cast against a shape.
#[derive(Debug, Copy, Clone)]
pub struct RayHit {
    /// Impact point in world space.
    pub point: Vec2,
    /// Surface normal at the impact point.
    pub normal: Vec2,
    /// Distance along the ray from the origin to `point`.
    pub distance: f32,
}

impl Ray2D {
    /// Create a new ray; `direction` will be normalized.
    pub fn new(origin: Vec2, direction: Vec2) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }

    /// Sample a point `distance` units along the ray.
    pub fn point_at(&self, distance: f32) -> Vec2 {
        self.origin + self.direction * distance
    }

    /// Cast this ray against an AABB, returning the closest hit if any.
    pub fn cast_aabb(&self, aabb: &AABB) -> Option<RayHit> {
        let inv_dir = Vec2::new(1.0 / self.direction.x, 1.0 / self.direction.y);

        let t1 = (aabb.min.x - self.origin.x) * inv_dir.x;
        let t2 = (aabb.max.x - self.origin.x) * inv_dir.x;
        let t3 = (aabb.min.y - self.origin.y) * inv_dir.y;
        let t4 = (aabb.max.y - self.origin.y) * inv_dir.y;

        let tmin = t1.min(t2).max(t3.min(t4));
        let tmax = t1.max(t2).min(t3.max(t4));

        if tmax < 0.0 || tmin > tmax {
            return None;
        }

        let distance = if tmin < 0.0 { tmax } else { tmin };
        let point = self.point_at(distance);

        // Calculate normal based on which face was hit
        let center = aabb.center();
        let half_size = aabb.half_size();
        let local_point = point - center;

        let normal = if local_point.x.abs() / half_size.x > local_point.y.abs() / half_size.y {
            Vec2::new(local_point.x.signum(), 0.0)
        } else {
            Vec2::new(0.0, local_point.y.signum())
        };

        Some(RayHit {
            point,
            normal,
            distance,
        })
    }

    /// Cast this ray against a circle, returning the closest hit if any.
    pub fn cast_circle(&self, circle: &Circle) -> Option<RayHit> {
        let to_circle = circle.center - self.origin;
        let projected = to_circle.dot(self.direction);

        if projected < 0.0 {
            return None;
        }

        let closest_point = self.origin + self.direction * projected;
        let distance_to_center = closest_point.distance_squared(circle.center);

        if distance_to_center > circle.radius * circle.radius {
            return None;
        }

        let half_chord = circle
            .radius
            .mul_add(circle.radius, -distance_to_center)
            .sqrt();
        let distance = projected - half_chord;

        if distance < 0.0 {
            return None;
        }

        let point = self.point_at(distance);
        let normal = (point - circle.center).normalize();

        Some(RayHit {
            point,
            normal,
            distance,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1e-4;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    fn vec_approx_eq(a: Vec2, b: Vec2) -> bool {
        approx_eq(a.x, b.x) && approx_eq(a.y, b.y)
    }

    // =========================================================================
    // Construction
    // =========================================================================

    #[test]
    fn construction_normalizes_direction() {
        let ray = Ray2D::new(Vec2::ZERO, Vec2::new(10.0, 0.0));
        assert!(approx_eq(ray.direction.length(), 1.0));
    }

    #[test]
    fn point_at_samples_along_ray() {
        let ray = Ray2D::new(Vec2::ZERO, Vec2::RIGHT);
        assert_eq!(ray.point_at(5.0), Vec2::new(5.0, 0.0));
        assert_eq!(ray.point_at(10.0), Vec2::new(10.0, 0.0));
    }

    // =========================================================================
    // Ray-AABB Intersection
    // =========================================================================

    #[test]
    fn ray_hits_aabb_from_left() {
        let ray = Ray2D::new(Vec2::new(-10.0, 5.0), Vec2::RIGHT);
        let aabb = AABB::new(Vec2::ZERO, Vec2::new(10.0, 10.0));
        let hit = ray.cast_aabb(&aabb).unwrap();
        assert!(approx_eq(hit.distance, 10.0));
        assert!(vec_approx_eq(hit.point, Vec2::new(0.0, 5.0)));
        assert!(vec_approx_eq(hit.normal, Vec2::LEFT));
    }

    #[test]
    fn ray_hits_aabb_from_bottom() {
        let ray = Ray2D::new(Vec2::new(5.0, -10.0), Vec2::UP);
        let aabb = AABB::new(Vec2::ZERO, Vec2::new(10.0, 10.0));
        let hit = ray.cast_aabb(&aabb).unwrap();
        assert!(approx_eq(hit.distance, 10.0));
        assert!(vec_approx_eq(hit.point, Vec2::new(5.0, 0.0)));
        assert!(vec_approx_eq(hit.normal, Vec2::DOWN));
    }

    #[test]
    fn ray_misses_aabb() {
        let ray = Ray2D::new(Vec2::new(-10.0, 15.0), Vec2::RIGHT);
        let aabb = AABB::new(Vec2::ZERO, Vec2::new(10.0, 10.0));
        assert!(ray.cast_aabb(&aabb).is_none());
    }

    #[test]
    fn ray_pointing_away_from_aabb_misses() {
        let ray = Ray2D::new(Vec2::new(-10.0, 5.0), Vec2::LEFT);
        let aabb = AABB::new(Vec2::ZERO, Vec2::new(10.0, 10.0));
        assert!(ray.cast_aabb(&aabb).is_none());
    }

    // =========================================================================
    // Ray-Circle Intersection
    // =========================================================================

    #[test]
    fn ray_hits_circle_through_center() {
        let ray = Ray2D::new(Vec2::new(-20.0, 0.0), Vec2::RIGHT);
        let circle = Circle::new(Vec2::ZERO, 10.0);
        let hit = ray.cast_circle(&circle).unwrap();
        assert!(approx_eq(hit.distance, 10.0));
        assert!(vec_approx_eq(hit.point, Vec2::new(-10.0, 0.0)));
        assert!(vec_approx_eq(hit.normal, Vec2::LEFT));
    }

    #[test]
    fn ray_hits_circle_tangentially() {
        // Ray just grazes the top of the circle
        let ray = Ray2D::new(Vec2::new(-20.0, 10.0), Vec2::RIGHT);
        let circle = Circle::new(Vec2::ZERO, 10.0);
        let hit = ray.cast_circle(&circle).unwrap();
        assert!(vec_approx_eq(hit.point, Vec2::new(0.0, 10.0)));
    }

    #[test]
    fn ray_misses_circle() {
        let ray = Ray2D::new(Vec2::new(-20.0, 15.0), Vec2::RIGHT);
        let circle = Circle::new(Vec2::ZERO, 10.0);
        assert!(ray.cast_circle(&circle).is_none());
    }

    #[test]
    fn ray_pointing_away_from_circle_misses() {
        let ray = Ray2D::new(Vec2::new(-20.0, 0.0), Vec2::LEFT);
        let circle = Circle::new(Vec2::ZERO, 10.0);
        assert!(ray.cast_circle(&circle).is_none());
    }

    #[test]
    fn ray_origin_behind_circle_misses() {
        // Ray starts past the circle, pointing away
        let ray = Ray2D::new(Vec2::new(20.0, 0.0), Vec2::RIGHT);
        let circle = Circle::new(Vec2::ZERO, 10.0);
        assert!(ray.cast_circle(&circle).is_none());
    }
}

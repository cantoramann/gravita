//! Right-handed Y-up perspective camera.

use gravita_math::Vec3;

/// View + perspective projection camera.
#[derive(Debug, Clone)]
pub struct Camera {
    /// Eye position in world space.
    pub eye: Vec3,
    /// Point being looked at.
    pub target: Vec3,
    /// Up vector. Usually [`Vec3::Y`].
    pub up: Vec3,
    /// Vertical field of view in radians.
    pub fov_y: f32,
    /// Aspect ratio (width / height).
    pub aspect: f32,
    /// Near plane distance (must be positive).
    pub near: f32,
    /// Far plane distance.
    pub far: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            eye: Vec3::new(0.0, 2.0, 5.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            fov_y: std::f32::consts::FRAC_PI_3,
            aspect: 16.0 / 9.0,
            near: 0.1,
            far: 100.0,
        }
    }
}

impl Camera {
    /// Build the view matrix (right-handed look-at).
    pub fn view(&self) -> [[f32; 4]; 4] {
        let f = (self.target - self.eye).normalize();
        let r = f.cross(self.up).normalize();
        let u = r.cross(f);
        // Column-major 4x4 with translation in the last column.
        [
            [r.x, u.x, -f.x, 0.0],
            [r.y, u.y, -f.y, 0.0],
            [r.z, u.z, -f.z, 0.0],
            [-r.dot(self.eye), -u.dot(self.eye), f.dot(self.eye), 1.0],
        ]
    }

    /// Build the perspective projection matrix, mapping clip-space Z to
    /// `[0, 1]` (wgpu / D3D / Metal convention).
    pub fn projection(&self) -> [[f32; 4]; 4] {
        let f = 1.0 / (self.fov_y * 0.5).tan();
        let nf = 1.0 / (self.near - self.far);
        [
            [f / self.aspect, 0.0, 0.0, 0.0],
            [0.0, f, 0.0, 0.0],
            [0.0, 0.0, self.far * nf, -1.0],
            [0.0, 0.0, self.far * self.near * nf, 0.0],
        ]
    }

    /// Combined view-projection.
    pub fn view_projection(&self) -> [[f32; 4]; 4] {
        mat4_mul(self.projection(), self.view())
    }
}

#[inline]
fn mat4_mul(a: [[f32; 4]; 4], b: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let mut out = [[0.0f32; 4]; 4];
    for col in 0..4 {
        for row in 0..4 {
            let mut sum = 0.0;
            for (k, a_col) in a.iter().enumerate() {
                sum = a_col[row].mul_add(b[col][k], sum);
            }
            out[col][row] = sum;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-4
    }

    #[test]
    fn default_camera_has_sensible_values() {
        let c = Camera::default();
        assert!(c.aspect > 0.0);
        assert!(c.near < c.far);
        assert!(c.fov_y > 0.0);
    }

    #[test]
    fn view_matrix_translates_origin_into_camera_space() {
        // Camera looking down -Z from origin: view of (0,0,0) should be (0,0,0).
        let c = Camera {
            eye: Vec3::ZERO,
            target: -Vec3::Z,
            up: Vec3::Y,
            ..Camera::default()
        };
        let v = c.view();
        // Last column is the translation
        assert!(approx(v[3][0], 0.0));
        assert!(approx(v[3][1], 0.0));
        assert!(approx(v[3][2], 0.0));
    }

    #[test]
    fn projection_far_plane_maps_to_one() {
        // A point on the far plane should map z' = 1 in clip space after
        // perspective divide.
        let c = Camera {
            eye: Vec3::ZERO,
            target: -Vec3::Z,
            up: Vec3::Y,
            fov_y: std::f32::consts::FRAC_PI_2,
            aspect: 1.0,
            near: 0.1,
            far: 100.0,
        };
        let p = c.projection();
        // Apply p to view-space point (0, 0, -far, 1) i.e. directly on the far plane.
        let zp = p[2][2].mul_add(-c.far, p[3][2]);
        let wp = p[2][3] * (-c.far);
        let z_ndc = zp / wp;
        assert!(approx(z_ndc, 1.0), "far plane z_ndc = {z_ndc}");
    }

    #[test]
    fn projection_near_plane_maps_to_zero() {
        let c = Camera {
            eye: Vec3::ZERO,
            target: -Vec3::Z,
            up: Vec3::Y,
            fov_y: std::f32::consts::FRAC_PI_2,
            aspect: 1.0,
            near: 0.1,
            far: 100.0,
        };
        let p = c.projection();
        let zp = p[2][2].mul_add(-c.near, p[3][2]);
        let wp = p[2][3] * (-c.near);
        let z_ndc = zp / wp;
        assert!(approx(z_ndc, 0.0), "near plane z_ndc = {z_ndc}");
    }

    #[test]
    fn view_projection_combines_view_and_projection() {
        let c = Camera::default();
        let vp = c.view_projection();
        // Sanity: not all zeros, has reasonable magnitude.
        assert!(vp[0][0].abs() > 0.0);
        assert!(vp[3][3].abs() < 100.0);
    }
}

//! CPU-side meshes built once at startup and uploaded to GPU buffers by
//! `MeshBuffer::upload`.

use wgpu::util::DeviceExt;

use crate::vertex::Vertex;

/// CPU-side mesh: positions, colors, normals and triangle indices.
#[derive(Debug, Clone)]
pub struct Mesh {
    /// Per-vertex data.
    pub vertices: Vec<Vertex>,
    /// Triangle indices into `vertices` (3 per face).
    pub indices: Vec<u32>,
}

impl Mesh {
    /// Number of triangle indices.
    #[must_use]
    pub fn index_count(&self) -> u32 {
        self.indices.len() as u32
    }

    /// Axis-aligned cube of size 2 centered at the origin, with one color per
    /// face (so even an unlit shader produces visibly different sides).
    #[must_use]
    pub fn cube() -> Self {
        // Helper: build a face from 4 corners + a normal + a color, emitting two triangles.
        let mut vertices = Vec::with_capacity(24);
        let mut indices = Vec::with_capacity(36);
        let mut face = |a, b, c, d, normal, color: [f32; 3]| {
            let base = vertices.len() as u32;
            vertices.push(Vertex::new(a, color, normal));
            vertices.push(Vertex::new(b, color, normal));
            vertices.push(Vertex::new(c, color, normal));
            vertices.push(Vertex::new(d, color, normal));
            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        };

        // +X (red)
        face(
            [1.0, -1.0, -1.0],
            [1.0, 1.0, -1.0],
            [1.0, 1.0, 1.0],
            [1.0, -1.0, 1.0],
            [1.0, 0.0, 0.0],
            [0.9, 0.2, 0.2],
        );
        // -X (cyan)
        face(
            [-1.0, -1.0, 1.0],
            [-1.0, 1.0, 1.0],
            [-1.0, 1.0, -1.0],
            [-1.0, -1.0, -1.0],
            [-1.0, 0.0, 0.0],
            [0.2, 0.8, 0.9],
        );
        // +Y (green)
        face(
            [-1.0, 1.0, -1.0],
            [-1.0, 1.0, 1.0],
            [1.0, 1.0, 1.0],
            [1.0, 1.0, -1.0],
            [0.0, 1.0, 0.0],
            [0.2, 0.9, 0.3],
        );
        // -Y (magenta)
        face(
            [-1.0, -1.0, 1.0],
            [-1.0, -1.0, -1.0],
            [1.0, -1.0, -1.0],
            [1.0, -1.0, 1.0],
            [0.0, -1.0, 0.0],
            [0.9, 0.2, 0.8],
        );
        // +Z (blue)
        face(
            [-1.0, -1.0, 1.0],
            [1.0, -1.0, 1.0],
            [1.0, 1.0, 1.0],
            [-1.0, 1.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.2, 0.4, 0.95],
        );
        // -Z (yellow)
        face(
            [1.0, -1.0, -1.0],
            [-1.0, -1.0, -1.0],
            [-1.0, 1.0, -1.0],
            [1.0, 1.0, -1.0],
            [0.0, 0.0, -1.0],
            [0.95, 0.85, 0.2],
        );

        Self { vertices, indices }
    }

    /// Flat XZ plane of size `size`, centered at origin, facing +Y.
    #[must_use]
    pub fn plane(size: f32, color: [f32; 3]) -> Self {
        let h = size * 0.5;
        let vertices = vec![
            Vertex::new([-h, 0.0, -h], color, [0.0, 1.0, 0.0]),
            Vertex::new([h, 0.0, -h], color, [0.0, 1.0, 0.0]),
            Vertex::new([h, 0.0, h], color, [0.0, 1.0, 0.0]),
            Vertex::new([-h, 0.0, h], color, [0.0, 1.0, 0.0]),
        ];
        let indices = vec![0, 1, 2, 0, 2, 3];
        Self { vertices, indices }
    }

    /// UV sphere with `segments` longitudinal slices and `rings` latitudinal
    /// bands, of given `radius` and `color`. A reasonable default is
    /// `(24, 16)`.
    #[must_use]
    pub fn uv_sphere(radius: f32, segments: u32, rings: u32, color: [f32; 3]) -> Self {
        let segments = segments.max(3);
        let rings = rings.max(2);
        let mut vertices = Vec::with_capacity(((rings + 1) * (segments + 1)) as usize);
        let mut indices = Vec::with_capacity((rings * segments * 6) as usize);

        use std::f32::consts::PI;
        for ring in 0..=rings {
            let v = ring as f32 / rings as f32;
            let phi = v * PI;
            let (sphi, cphi) = phi.sin_cos();
            for seg in 0..=segments {
                let u = seg as f32 / segments as f32;
                let theta = u * 2.0 * PI;
                let (stheta, ctheta) = theta.sin_cos();
                let nx = sphi * ctheta;
                let ny = cphi;
                let nz = sphi * stheta;
                vertices.push(Vertex::new(
                    [nx * radius, ny * radius, nz * radius],
                    color,
                    [nx, ny, nz],
                ));
            }
        }

        let stride = segments + 1;
        for ring in 0..rings {
            for seg in 0..segments {
                let a = ring * stride + seg;
                let b = a + stride;
                indices.extend_from_slice(&[a, b, a + 1, a + 1, b, b + 1]);
            }
        }

        Self { vertices, indices }
    }
}

/// GPU-resident vertex + index buffers for a [`Mesh`].
#[derive(Debug)]
pub struct MeshBuffer {
    /// Vertex buffer.
    pub vertex_buffer: wgpu::Buffer,
    /// Index buffer.
    pub index_buffer: wgpu::Buffer,
    /// Number of indices (the count passed to `draw_indexed`).
    pub index_count: u32,
}

impl MeshBuffer {
    /// Upload `mesh` to GPU buffers. The returned buffers reference no other
    /// allocations and can be dropped independently of the source mesh.
    #[must_use]
    pub fn upload(device: &wgpu::Device, label: &str, mesh: &Mesh) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{label} vertices")),
            contents: bytemuck::cast_slice(&mesh.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{label} indices")),
            contents: bytemuck::cast_slice(&mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        Self {
            vertex_buffer,
            index_buffer,
            index_count: mesh.index_count(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cube_has_six_faces_worth_of_indices() {
        let m = Mesh::cube();
        // 6 faces * 2 triangles * 3 verts = 36 indices, and 24 unique vertices.
        assert_eq!(m.indices.len(), 36);
        assert_eq!(m.vertices.len(), 24);
        assert_eq!(m.index_count(), 36);
    }

    #[test]
    fn plane_has_two_triangles() {
        let m = Mesh::plane(10.0, [1.0, 1.0, 1.0]);
        assert_eq!(m.indices.len(), 6);
        assert_eq!(m.vertices.len(), 4);
    }

    #[test]
    fn uv_sphere_index_count_scales_with_resolution() {
        let s = Mesh::uv_sphere(1.0, 8, 4, [1.0, 1.0, 1.0]);
        // 8 segments * 4 rings * 6 indices per quad = 192
        assert_eq!(s.indices.len(), 192);
    }

    #[test]
    fn uv_sphere_radius_normals_unit_length() {
        let s = Mesh::uv_sphere(3.0, 8, 4, [1.0; 3]);
        for v in &s.vertices {
            let n = v.normal;
            let len = n[2].mul_add(n[2], n[0].mul_add(n[0], n[1] * n[1])).sqrt();
            assert!((len - 1.0).abs() < 1e-4, "normal not unit length: {n:?}");
        }
    }
}

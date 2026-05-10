//! Vertex + instance layouts uploaded to the GPU.

use bytemuck::{Pod, Zeroable};

/// Per-vertex data: position, color, normal.
#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct Vertex {
    /// Object-space position.
    pub position: [f32; 3],
    /// RGB color in `[0, 1]`. Multiplied with the per-instance tint.
    pub color: [f32; 3],
    /// Object-space normal (will be transformed by the model matrix in the
    /// vertex shader).
    pub normal: [f32; 3],
}

impl Vertex {
    /// Convenience constructor.
    #[inline]
    pub const fn new(position: [f32; 3], color: [f32; 3], normal: [f32; 3]) -> Self {
        Self { position, color, normal }
    }

    /// `wgpu::VertexBufferLayout` describing this struct.
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 24,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

/// Per-instance data: 4×4 model matrix (as 4 vec4 rows) + RGBA tint.
#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct InstanceRaw {
    /// Column-major model matrix, packed as 4 rows for the shader.
    pub model: [[f32; 4]; 4],
    /// RGBA tint applied to the vertex color.
    pub tint: [f32; 4],
}

impl InstanceRaw {
    /// `wgpu::VertexBufferLayout` describing this struct, stepped per-instance.
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        const ATTRS: &[wgpu::VertexAttribute] = &[
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 3,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: 16,
                shader_location: 4,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: 32,
                shader_location: 5,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: 48,
                shader_location: 6,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: 64,
                shader_location: 7,
                format: wgpu::VertexFormat::Float32x4,
            },
        ];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: ATTRS,
        }
    }
}

/// Global uniform: view-projection + directional light + ambient.
#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct GlobalsRaw {
    /// View-projection matrix.
    pub view_proj: [[f32; 4]; 4],
    /// Light direction (towards the light), padded.
    pub light_dir: [f32; 4],
    /// Ambient color, padded.
    pub ambient: [f32; 4],
}

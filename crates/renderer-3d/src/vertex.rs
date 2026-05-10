//! Vertex + instance layouts uploaded to the GPU.
//!
//! These structs are `Pod`/`Zeroable` via manual `unsafe impl` (rather than
//! the `bytemuck` derive macro) to avoid pulling in the `bytemuck_derive`
//! proc-macro crate. Every field is a `[f32; N]` or `[[f32; N]; M]` of
//! plain floats inside a `#[repr(C)]` struct — there is no padding and any
//! bit pattern is a valid `f32`, so both traits' invariants hold trivially.

// Manual `Pod` / `Zeroable` for GPU upload — see module-level comment above.
#![allow(unsafe_code)]

/// Per-vertex data: position, color, normal.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Vertex {
    /// Object-space position.
    pub position: [f32; 3],
    /// RGB color in `[0, 1]`. Multiplied with the per-instance tint.
    pub color: [f32; 3],
    /// Object-space normal (will be transformed by the model matrix in the
    /// vertex shader).
    pub normal: [f32; 3],
}

// SAFETY: `#[repr(C)]` struct of `[f32; 3]` fields. No padding, no invalid
// bit patterns, no `Drop` impl.
unsafe impl bytemuck::Zeroable for Vertex {}
unsafe impl bytemuck::Pod for Vertex {}

impl Vertex {
    /// Convenience constructor.
    #[inline]
    pub const fn new(position: [f32; 3], color: [f32; 3], normal: [f32; 3]) -> Self {
        Self {
            position,
            color,
            normal,
        }
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
#[derive(Debug, Copy, Clone)]
pub struct InstanceRaw {
    /// Column-major model matrix, packed as 4 rows for the shader.
    pub model: [[f32; 4]; 4],
    /// RGBA tint applied to the vertex color.
    pub tint: [f32; 4],
}

// SAFETY: `#[repr(C)]` struct of `[f32; N]` and `[[f32; N]; M]` fields only.
unsafe impl bytemuck::Zeroable for InstanceRaw {}
unsafe impl bytemuck::Pod for InstanceRaw {}

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
#[derive(Debug, Copy, Clone)]
pub struct GlobalsRaw {
    /// View-projection matrix.
    pub view_proj: [[f32; 4]; 4],
    /// Light direction (towards the light), padded.
    pub light_dir: [f32; 4],
    /// Ambient color, padded.
    pub ambient: [f32; 4],
}

// SAFETY: `#[repr(C)]` struct of `[f32; N]` and `[[f32; N]; M]` fields only.
unsafe impl bytemuck::Zeroable for GlobalsRaw {}
unsafe impl bytemuck::Pod for GlobalsRaw {}

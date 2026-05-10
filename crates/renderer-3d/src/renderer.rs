//! `wgpu` pipeline and per-frame drawing.

use std::sync::Arc;

use gravita_math::Transform3D;
use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::{
    camera::Camera,
    mesh::MeshBuffer,
    vertex::{GlobalsRaw, InstanceRaw, Vertex},
};

/// Identifier for a registered mesh.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct MeshHandle(pub(crate) usize);

/// One mesh instance to draw this frame.
#[derive(Debug, Copy, Clone)]
pub struct Instance {
    /// Which registered mesh to draw.
    pub mesh: MeshHandle,
    /// World-space transform.
    pub transform: Transform3D,
    /// RGBA tint multiplied with the mesh's vertex colors.
    pub tint: [f32; 4],
}

/// Owns the `wgpu` device, surface, depth target, and the colored-mesh pipeline.
pub struct Renderer3D {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    depth_view: wgpu::TextureView,
    pipeline: wgpu::RenderPipeline,
    globals_buffer: wgpu::Buffer,
    globals_bind_group: wgpu::BindGroup,
    meshes: Vec<MeshBuffer>,
    /// Kept alive so the surface's borrowed window handle stays valid.
    _window: Arc<Window>,
    /// Background clear color.
    pub clear_color: [f32; 4],
    /// Light direction (towards the light).
    pub light_direction: [f32; 3],
    /// Ambient color.
    pub ambient: [f32; 3],
}

impl Renderer3D {
    /// Build a renderer attached to `window`. Performs the async `wgpu`
    /// adapter/device setup synchronously via `pollster`.
    ///
    /// # Errors
    ///
    /// Returns an error if a `wgpu` adapter or device could not be acquired
    /// or if the surface can't be created from the window handle.
    pub fn new(window: Arc<Window>) -> Result<Self, Box<dyn std::error::Error>> {
        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });
        // `instance.create_surface` borrows the window unless we hand it an
        // owned target. Arc<Window> satisfies that requirement.
        let surface = instance.create_surface(Arc::clone(&window))?;

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .ok_or("no compatible wgpu adapter")?;

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("gravita-renderer-3d device"),
                required_features: wgpu::Features::empty(),
                required_limits:
                    wgpu::Limits::downlevel_defaults().using_resolution(adapter.limits()),
            },
            None,
        ))?;

        let surface_caps = surface.get_capabilities(&adapter);
        let format = surface_caps
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let depth_view = create_depth_view(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("colored-mesh shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let globals_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("globals uniform"),
            size: std::mem::size_of::<GlobalsRaw>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let globals_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("globals bind group layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let globals_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("globals bind group"),
            layout: &globals_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: globals_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("colored-mesh pipeline layout"),
            bind_group_layouts: &[&globals_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("colored-mesh pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::layout(), InstanceRaw::layout()],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        Ok(Self {
            surface,
            device,
            queue,
            config,
            depth_view,
            pipeline,
            globals_buffer,
            globals_bind_group,
            meshes: Vec::new(),
            _window: window,
            clear_color: [0.05, 0.08, 0.12, 1.0],
            light_direction: [0.4, 1.0, 0.6],
            ambient: [0.18, 0.18, 0.22],
        })
    }

    /// Aspect ratio of the current framebuffer.
    #[must_use]
    pub fn aspect(&self) -> f32 {
        self.config.width as f32 / self.config.height as f32
    }

    /// Resize the surface (call from `WindowEvent::Resized`).
    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.depth_view = create_depth_view(&self.device, &self.config);
    }

    /// Upload a mesh to the GPU and return a handle for use in `Instance`.
    pub fn register_mesh(&mut self, label: &str, mesh: &crate::mesh::Mesh) -> MeshHandle {
        let buffer = MeshBuffer::upload(&self.device, label, mesh);
        let id = self.meshes.len();
        self.meshes.push(buffer);
        MeshHandle(id)
    }

    /// Draw `instances` from the perspective of `camera` and present.
    pub fn render(
        &mut self,
        camera: &Camera,
        instances: &[Instance],
    ) -> Result<(), wgpu::SurfaceError> {
        // Update globals uniform.
        let globals = GlobalsRaw {
            view_proj: camera.view_projection(),
            light_dir: [
                self.light_direction[0],
                self.light_direction[1],
                self.light_direction[2],
                0.0,
            ],
            ambient: [self.ambient[0], self.ambient[1], self.ambient[2], 0.0],
        };
        self.queue
            .write_buffer(&self.globals_buffer, 0, bytemuck::bytes_of(&globals));

        // Group instances by mesh so each mesh draws once with all its instances.
        let mut by_mesh: Vec<Vec<InstanceRaw>> = vec![Vec::new(); self.meshes.len()];
        for inst in instances {
            let raw = InstanceRaw {
                model: inst.transform.to_matrix(),
                tint: inst.tint,
            };
            if let Some(bucket) = by_mesh.get_mut(inst.mesh.0) {
                bucket.push(raw);
            }
        }

        // Allocate one fresh instance buffer per non-empty mesh. They must
        // outlive the render pass, so we collect them before begin_render_pass.
        // (For tight per-frame budgets this should be a persistent ring buffer
        // with `Queue::write_buffer`; one alloc per mesh per frame is fine for
        // demo-scale scenes.)
        let instance_buffers: Vec<(usize, wgpu::Buffer, u32)> = by_mesh
            .iter()
            .enumerate()
            .filter(|(_, raws)| !raws.is_empty())
            .map(|(mesh_id, raws)| {
                let buf = self
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("instance buffer"),
                        contents: bytemuck::cast_slice(raws),
                        usage: wgpu::BufferUsages::VERTEX,
                    });
                (mesh_id, buf, raws.len() as u32)
            })
            .collect();

        let frame = self.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame encoder"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("colored-mesh pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: self.clear_color[0] as f64,
                            g: self.clear_color[1] as f64,
                            b: self.clear_color[2] as f64,
                            a: self.clear_color[3] as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.globals_bind_group, &[]);

            for (mesh_id, buf, count) in &instance_buffers {
                let mesh = &self.meshes[*mesh_id];
                pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                pass.set_vertex_buffer(1, buf.slice(..));
                pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..mesh.index_count, 0, 0..*count);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
        Ok(())
    }
}

fn create_depth_view(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
) -> wgpu::TextureView {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("depth texture"),
        size: wgpu::Extent3d {
            width: config.width.max(1),
            height: config.height.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    texture.create_view(&wgpu::TextureViewDescriptor::default())
}

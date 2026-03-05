//! Loom WebGL - GPU Acceleration and 3D Rendering
//!
//! Phase L17: WebGL/GPU Acceleration
//! - WebGL context creation via wgpu
//! - Shader compilation (WGSL)
//! - Buffer/texture management
//! - Basic 3D rendering pipeline
//! - Canvas 3D context support

use anyhow::{anyhow, Result};
use glam::{Mat4, Vec3, Vec4};
use log::{debug, error, info, warn};
use std::sync::Arc;
use wgpu::util::DeviceExt;

pub mod buffer;
pub mod context;
pub mod pipeline;
pub mod shader;
pub mod texture;

pub use buffer::*;
pub use context::*;
pub use pipeline::*;
pub use shader::*;
pub use texture::*;

/// Version of the WebGL crate
pub const VERSION: &str = "0.1.0-L17";

/// WebGL capabilities
#[derive(Debug, Clone)]
pub struct WebGlCapabilities {
    /// Max texture size
    pub max_texture_size: u32,
    /// Max uniform buffer binding size
    pub max_uniform_buffer_binding_size: u32,
    /// Supports depth textures
    pub supports_depth_texture: bool,
    /// Supports multisampling
    pub supports_multisampling: bool,
    /// Anisotropic filtering level
    pub max_anisotropy: Option<u16>,
}

impl WebGlCapabilities {
    /// Query capabilities from device
    pub fn query(device: &wgpu::Device, adapter: &wgpu::Adapter) -> Self {
        let limits = device.limits();
        let features = adapter.features();
        
        Self {
            max_texture_size: limits.max_texture_dimension_2d,
            max_uniform_buffer_binding_size: limits.max_uniform_buffer_binding_size,
            supports_depth_texture: features.contains(wgpu::Features::DEPTH_CLIP_CONTROL),
            supports_multisampling: true, // Standard in wgpu
            max_anisotropy: None, // Would need extension query
        }
    }
}

/// A 3D mesh for rendering
#[derive(Debug)]
pub struct Mesh {
    /// Vertex buffer
    pub vertex_buffer: wgpu::Buffer,
    /// Index buffer (optional)
    pub index_buffer: Option<wgpu::Buffer>,
    /// Number of vertices
    pub vertex_count: u32,
    /// Number of indices
    pub index_count: u32,
    /// Vertex layout
    pub vertex_layout: wgpu::VertexBufferLayout<'static>,
}

impl Mesh {
    /// Create a cube mesh
    pub fn create_cube(device: &wgpu::Device) -> Self {
        // Cube vertices: position (3), normal (3), uv (2)
        let vertices: &[f32] = &[
            // Front face
            -0.5, -0.5,  0.5,  0.0,  0.0,  1.0,  0.0, 0.0,
             0.5, -0.5,  0.5,  0.0,  0.0,  1.0,  1.0, 0.0,
             0.5,  0.5,  0.5,  0.0,  0.0,  1.0,  1.0, 1.0,
            -0.5,  0.5,  0.5,  0.0,  0.0,  1.0,  0.0, 1.0,
            // Back face
            -0.5, -0.5, -0.5,  0.0,  0.0, -1.0,  1.0, 0.0,
             0.5, -0.5, -0.5,  0.0,  0.0, -1.0,  0.0, 0.0,
             0.5,  0.5, -0.5,  0.0,  0.0, -1.0,  0.0, 1.0,
            -0.5,  0.5, -0.5,  0.0,  0.0, -1.0,  1.0, 1.0,
            // Top face
            -0.5,  0.5, -0.5,  0.0,  1.0,  0.0,  0.0, 0.0,
             0.5,  0.5, -0.5,  0.0,  1.0,  0.0,  1.0, 0.0,
             0.5,  0.5,  0.5,  0.0,  1.0,  0.0,  1.0, 1.0,
            -0.5,  0.5,  0.5,  0.0,  1.0,  0.0,  0.0, 1.0,
            // Bottom face
            -0.5, -0.5, -0.5,  0.0, -1.0,  0.0,  0.0, 1.0,
             0.5, -0.5, -0.5,  0.0, -1.0,  0.0,  1.0, 1.0,
             0.5, -0.5,  0.5,  0.0, -1.0,  0.0,  1.0, 0.0,
            -0.5, -0.5,  0.5,  0.0, -1.0,  0.0,  0.0, 0.0,
            // Right face
             0.5, -0.5, -0.5,  1.0,  0.0,  0.0,  0.0, 0.0,
             0.5,  0.5, -0.5,  1.0,  0.0,  0.0,  1.0, 0.0,
             0.5,  0.5,  0.5,  1.0,  0.0,  0.0,  1.0, 1.0,
             0.5, -0.5,  0.5,  1.0,  0.0,  0.0,  0.0, 1.0,
            // Left face
            -0.5, -0.5, -0.5, -1.0,  0.0,  0.0,  1.0, 0.0,
            -0.5,  0.5, -0.5, -1.0,  0.0,  0.0,  0.0, 0.0,
            -0.5,  0.5,  0.5, -1.0,  0.0,  0.0,  0.0, 1.0,
            -0.5, -0.5,  0.5, -1.0,  0.0,  0.0,  1.0, 1.0,
        ];

        let indices: &[u16] = &[
            0, 1, 2, 0, 2, 3,       // front
            4, 5, 6, 4, 6, 7,       // back
            8, 9, 10, 8, 10, 11,    // top
            12, 13, 14, 12, 14, 15, // bottom
            16, 17, 18, 16, 18, 19, // right
            20, 21, 22, 20, 22, 23, // left
        ];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cube Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cube Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: 32, // 8 floats * 4 bytes
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
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        };

        Self {
            vertex_buffer,
            index_buffer: Some(index_buffer),
            vertex_count: vertices.len() as u32 / 8,
            index_count: indices.len() as u32,
            vertex_layout,
        }
    }
}

/// Transformation matrices for 3D rendering
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TransformUniform {
    pub model: [[f32; 4]; 4],
    pub view: [[f32; 4]; 4],
    pub projection: [[f32; 4]; 4],
}

impl TransformUniform {
    pub fn new() -> Self {
        Self {
            model: Mat4::IDENTITY.to_cols_array_2d(),
            view: Mat4::IDENTITY.to_cols_array_2d(),
            projection: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    pub fn update_model(&mut self, rotation: f32) {
        self.model = Mat4::from_rotation_y(rotation).to_cols_array_2d();
    }

    pub fn update_view(&mut self, eye: Vec3, target: Vec3, up: Vec3) {
        self.view = Mat4::look_at_rh(eye, target, up).to_cols_array_2d();
    }

    pub fn update_projection(&mut self, fov: f32, aspect: f32, near: f32, far: f32) {
        self.projection = Mat4::perspective_rh(fov, aspect, near, far).to_cols_array_2d();
    }
}

/// Basic 3D renderer
pub struct Renderer3D {
    /// WebGL context
    context: WebGlContext,
    /// Render pipeline
    pipeline: wgpu::RenderPipeline,
    /// Uniform buffer
    uniform_buffer: wgpu::Buffer,
    /// Uniform bind group
    bind_group: wgpu::BindGroup,
    /// Depth texture
    depth_texture: wgpu::Texture,
    /// Depth texture view
    depth_view: wgpu::TextureView,
}

impl Renderer3D {
    /// Create a new 3D renderer
    pub async fn new(window: &impl raw_window_handle::HasWindowHandle) -> Result<Self> {
        let context = WebGlContext::new(window).await?;
        
        let shader = context.device().create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("3D Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl")),
        });

        let uniform_buffer = context.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: std::mem::size_of::<TransformUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = context.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Uniform Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = context.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Uniform Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = context.device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = context.device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("3D Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 32,
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
                    ],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: context.config().format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
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
            multiview: None,
        });

        let (depth_texture, depth_view) = create_depth_texture(
            context.device(),
            context.config().width,
            context.config().height,
        );

        Ok(Self {
            context,
            pipeline,
            uniform_buffer,
            bind_group,
            depth_texture,
            depth_view,
        })
    }

    /// Render a frame
    pub fn render(&mut self, mesh: &Mesh, transform: &TransformUniform) -> Result<()> {
        // Update uniform buffer
        self.context.queue().write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[*transform]),
        );

        let output = self.context.surface().get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.context.device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.15,
                            a: 1.0,
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

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));

            if let Some(ref index_buffer) = mesh.index_buffer {
                render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
            } else {
                render_pass.draw(0..mesh.vertex_count, 0..1);
            }
        }

        self.context.queue().submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    /// Resize the renderer
    pub fn resize(&mut self, width: u32, height: u32) {
        self.context.resize(width, height);
        
        // Recreate depth texture
        let (depth_texture, depth_view) = create_depth_texture(
            self.context.device(),
            width,
            height,
        );
        self.depth_texture = depth_texture;
        self.depth_view = depth_view;
    }
}

/// Create a depth texture
fn create_depth_texture(device: &wgpu::Device, width: u32, height: u32) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Depth Texture"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    
    (texture, view)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_uniform() {
        let mut transform = TransformUniform::new();
        
        transform.update_model(std::f32::consts::PI / 4.0);
        transform.update_view(
            Vec3::new(0.0, 0.0, 3.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        );
        transform.update_projection(
            std::f32::consts::PI / 4.0,
            16.0 / 9.0,
            0.1,
            100.0,
        );
        
        // Just verify it doesn't panic
        assert!(true);
    }

    #[test]
    fn test_webgl_capabilities() {
        // Test default values
        let caps = WebGlCapabilities {
            max_texture_size: 8192,
            max_uniform_buffer_binding_size: 65536,
            supports_depth_texture: true,
            supports_multisampling: true,
            max_anisotropy: Some(16),
        };
        
        assert_eq!(caps.max_texture_size, 8192);
        assert!(caps.supports_depth_texture);
    }
}

//! Render pipeline management
//!
//! Graphics pipeline configuration

use anyhow::{anyhow, Result};

/// Render pipeline descriptor
pub struct PipelineDescriptor {
    /// Pipeline label
    pub label: Option<String>,
    /// Shader module
    pub shader: wgpu::ShaderModuleDescriptor<'static>,
    /// Vertex entry point
    pub vertex_entry: String,
    /// Fragment entry point
    pub fragment_entry: Option<String>,
    /// Vertex buffer layouts
    pub vertex_layouts: Vec<wgpu::VertexBufferLayout<'static>>,
    /// Primitive state
    pub primitive: wgpu::PrimitiveState,
    /// Depth stencil state
    pub depth_stencil: Option<wgpu::DepthStencilState>,
    /// Color target states
    pub color_targets: Vec<Option<wgpu::ColorTargetState>>,
    /// Multisample state
    pub multisample: wgpu::MultisampleState,
}

impl PipelineDescriptor {
    /// Create a default pipeline descriptor
    pub fn new(shader: wgpu::ShaderModuleDescriptor<'static>) -> Self {
        Self {
            label: None,
            shader,
            vertex_entry: "vs_main".to_string(),
            fragment_entry: Some("fs_main".to_string()),
            vertex_layouts: vec![],
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            color_targets: vec![Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            multisample: wgpu::MultisampleState::default(),
        }
    }

    /// Set label
    pub fn with_label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }

    /// Set vertex entry point
    pub fn with_vertex_entry(mut self, entry: &str) -> Self {
        self.vertex_entry = entry.to_string();
        self
    }

    /// Set fragment entry point
    pub fn with_fragment_entry(mut self, entry: Option<&str>) -> Self {
        self.fragment_entry = entry.map(|s| s.to_string());
        self
    }

    /// Add vertex buffer layout
    pub fn with_vertex_layout(mut self, layout: wgpu::VertexBufferLayout<'static>) -> Self {
        self.vertex_layouts.push(layout);
        self
    }

    /// Set primitive state
    pub fn with_primitive(mut self, primitive: wgpu::PrimitiveState) -> Self {
        self.primitive = primitive;
        self
    }

    /// Set depth stencil state
    pub fn with_depth_stencil(mut self, depth_stencil: Option<wgpu::DepthStencilState>) -> Self {
        self.depth_stencil = depth_stencil;
        self
    }

    /// Set color targets
    pub fn with_color_targets(mut self, targets: Vec<Option<wgpu::ColorTargetState>>) -> Self {
        self.color_targets = targets;
        self
    }

    /// Set multisample state
    pub fn with_multisample(mut self, multisample: wgpu::MultisampleState) -> Self {
        self.multisample = multisample;
        self
    }

    /// Build the pipeline
    pub fn build(self, device: &wgpu::Device) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(self.shader);

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: self.label.as_ref().map(|s| s.as_str()),
            entries: &[],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: self.label.as_ref().map(|s| format!("{} Layout", s).as_str()),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: self.label.as_ref().map(|s| s.as_str()),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader.module(),
                entry_point: &self.vertex_entry,
                buffers: &self.vertex_layouts,
            },
            fragment: self.fragment_entry.as_ref().map(|entry| wgpu::FragmentState {
                module: shader.module(),
                entry_point: entry,
                targets: &self.color_targets,
            }),
            primitive: self.primitive,
            depth_stencil: self.depth_stencil,
            multisample: self.multisample,
            multiview: None,
        })
    }
}

/// Default pipeline presets
pub mod presets {
    use super::*;

    /// Standard 3D pipeline with depth testing
    pub fn standard_3d(format: wgpu::TextureFormat) -> PipelineDescriptor {
        PipelineDescriptor::new(wgpu::ShaderModuleDescriptor {
            label: Some("Standard 3D Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        })
        .with_label("Standard 3D")
        .with_primitive(wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        })
        .with_depth_stencil(Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }))
        .with_color_targets(vec![Some(wgpu::ColorTargetState {
            format,
            blend: Some(wgpu::BlendState::REPLACE),
            write_mask: wgpu::ColorWrites::ALL,
        })])
    }

    /// 2D UI pipeline (no depth testing)
    pub fn ui_2d(format: wgpu::TextureFormat) -> PipelineDescriptor {
        PipelineDescriptor::new(wgpu::ShaderModuleDescriptor {
            label: Some("UI 2D Shader"),
            source: wgpu::ShaderSource::Wgsl(
                r#"
                @vertex
                fn vs_main(@location(0) position: vec2<f32>, @location(1) color: vec4<f32>) -> @builtin(position) vec4<f32> {
                    return vec4<f32>(position, 0.0, 1.0);
                }

                @fragment
                fn fs_main() -> @location(0) vec4<f32> {
                    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
                }
                "#.into(),
            ),
        })
        .with_label("UI 2D")
        .with_primitive(wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        })
        .with_depth_stencil(None)
        .with_color_targets(vec![Some(wgpu::ColorTargetState {
            format,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::ALL,
        })])
    }

    /// Wireframe pipeline
    pub fn wireframe(format: wgpu::TextureFormat) -> PipelineDescriptor {
        PipelineDescriptor::new(wgpu::ShaderModuleDescriptor {
            label: Some("Wireframe Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        })
        .with_label("Wireframe")
        .with_fragment_entry(Some("fs_wireframe"))
        .with_primitive(wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::LineList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Line,
            unclipped_depth: false,
            conservative: false,
        })
        .with_depth_stencil(Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }))
        .with_color_targets(vec![Some(wgpu::ColorTargetState {
            format,
            blend: Some(wgpu::BlendState::REPLACE),
            write_mask: wgpu::ColorWrites::ALL,
        })])
    }
}

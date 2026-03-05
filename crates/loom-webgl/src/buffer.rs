//! GPU Buffer management
//!
//! Vertex buffers, index buffers, uniform buffers

use wgpu::util::DeviceExt;

/// GPU buffer wrapper
pub struct GpuBuffer {
    /// wgpu buffer
    buffer: wgpu::Buffer,
    /// Buffer size in bytes
    size: u64,
    /// Usage flags
    usage: wgpu::BufferUsages,
}

impl GpuBuffer {
    /// Create a new GPU buffer with initial data
    pub fn new(device: &wgpu::Device, data: &[u8], usage: wgpu::BufferUsages, label: &str) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: data,
            usage,
        });

        Self {
            buffer,
            size: data.len() as u64,
            usage,
        }
    }

    /// Create an empty GPU buffer
    pub fn empty(device: &wgpu::Device, size: u64, usage: wgpu::BufferUsages, label: &str) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size,
            usage,
            mapped_at_creation: false,
        });

        Self { buffer, size, usage }
    }

    /// Get the wgpu buffer
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    /// Get buffer size
    pub fn size(&self) -> u64 {
        self.size
    }

    /// Write data to buffer
    pub fn write(&self, queue: &wgpu::Queue, offset: u64, data: &[u8]) {
        queue.write_buffer(&self.buffer, offset, data);
    }
}

/// Vertex buffer descriptor
pub struct VertexBufferDesc {
    /// Buffer stride in bytes
    pub stride: u64,
    /// Vertex attributes
    pub attributes: Vec<wgpu::VertexAttribute>,
    /// Step mode
    pub step_mode: wgpu::VertexStepMode,
}

impl VertexBufferDesc {
    /// Convert to wgpu vertex buffer layout
    pub fn to_layout(&self) -> wgpu::VertexBufferLayout {
        wgpu::VertexBufferLayout {
            array_stride: self.stride,
            step_mode: self.step_mode,
            attributes: &self.attributes, // Note: This would need proper lifetime management
        }
    }
}

/// Common vertex formats
pub mod vertex_formats {
    use super::*;

    /// Position (3 floats) + Normal (3 floats) + UV (2 floats)
    pub fn position_normal_uv() -> VertexBufferDesc {
        VertexBufferDesc {
            stride: 32,
            attributes: vec![
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
            step_mode: wgpu::VertexStepMode::Vertex,
        }
    }

    /// Position (3 floats) + Color (4 floats)
    pub fn position_color() -> VertexBufferDesc {
        VertexBufferDesc {
            stride: 28,
            attributes: vec![
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
            step_mode: wgpu::VertexStepMode::Vertex,
        }
    }
}

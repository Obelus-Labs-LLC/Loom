//! Main renderer using wgpu

use wgpu::{Device, Queue, Surface, SurfaceConfiguration};

/// Main renderer
pub struct Renderer {
    device: Device,
    queue: Queue,
}

impl Renderer {
    pub fn new(device: Device, queue: Queue) -> Self {
        Self { device, queue }
    }
}

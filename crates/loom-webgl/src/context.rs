//! WebGL Context management
//!
//! Handles wgpu instance, adapter, device, and surface creation

use anyhow::{anyhow, Result};
use log::{debug, info, warn};
use std::sync::Arc;

/// WebGL context wrapper around wgpu
pub struct WebGlContext {
    /// wgpu instance
    instance: wgpu::Instance,
    /// wgpu adapter
    adapter: wgpu::Adapter,
    /// wgpu device
    device: wgpu::Device,
    /// wgpu queue
    queue: wgpu::Queue,
    /// Surface for rendering
    surface: wgpu::Surface<'static>,
    /// Surface configuration
    config: wgpu::SurfaceConfiguration,
}

impl WebGlContext {
    /// Create a new WebGL context for a window
    pub async fn new(window: &impl raw_window_handle::HasWindowHandle) -> Result<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = unsafe {
            instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::from_window(window)
                .map_err(|e| anyhow!("Failed to create surface: {:?}", e))?)
        };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow!("No suitable GPU adapter found"))?;

        info!("Using GPU adapter: {:?}", adapter.get_info());

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("WebGL Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: 800,
            height: 600,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            surface,
            config,
        })
    }

    /// Create a headless WebGL context (for offscreen rendering)
    pub async fn new_headless() -> Result<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow!("No suitable GPU adapter found"))?;

        info!("Using GPU adapter (headless): {:?}", adapter.get_info());

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("WebGL Headless Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await?;

        // Create a dummy surface (we won't use it for rendering)
        let surface = instance.create_surface(wgpu::SurfaceTarget::from(std::sync::Arc::new(
            wgpu::SurfaceConfiguration::default(),
        )));

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            width: 800,
            height: 600,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            surface,
            config,
        })
    }

    /// Resize the surface
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    /// Get the device
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// Get the queue
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    /// Get the surface
    pub fn surface(&self) -> &wgpu::Surface {
        &self.surface
    }

    /// Get the surface configuration
    pub fn config(&self) -> &wgpu::SurfaceConfiguration {
        &self.config
    }

    /// Get the adapter
    pub fn adapter(&self) -> &wgpu::Adapter {
        &self.adapter
    }

    /// Get the preferred texture format
    pub fn format(&self) -> wgpu::TextureFormat {
        self.config.format
    }
}

/// Canvas 3D context for HTML canvas element
pub struct Canvas3DContext {
    /// WebGL context
    context: WebGlContext,
    /// Canvas width
    width: u32,
    /// Canvas height
    height: u32,
}

impl Canvas3DContext {
    /// Create a new 3D canvas context
    pub async fn new(width: u32, height: u32) -> Result<Self> {
        // In a real implementation, this would bind to an actual canvas
        // For now, we create a headless context
        let context = WebGlContext::new_headless().await?;

        Ok(Self {
            context,
            width,
            height,
        })
    }

    /// Resize the canvas
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.context.resize(width, height);
    }

    /// Get the WebGL context
    pub fn context(&self) -> &WebGlContext {
        &self.context
    }

    /// Get canvas width
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get canvas height
    pub fn height(&self) -> u32 {
        self.height
    }
}

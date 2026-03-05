//! Shader compilation and management
//!
//! WGSL shader support

use anyhow::{anyhow, Result};
use log::{debug, info};

/// Shader module wrapper
pub struct Shader {
    /// wgpu shader module
    module: wgpu::ShaderModule,
    /// Entry point name
    entry_point: String,
    /// Shader stage
    stage: wgpu::ShaderStages,
}

impl Shader {
    /// Create a shader from WGSL source
    pub fn from_wgsl(device: &wgpu::Device, source: &str, label: &str) -> Result<Self> {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(label),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });

        // Check for compilation errors
        if let Some(info) = module.get_compilation_info() {
            for msg in info.messages {
                match msg.message_type {
                    wgpu::CompilationMessageType::Error => {
                        return Err(anyhow!("Shader compilation error: {}", msg.message));
                    }
                    wgpu::CompilationMessageType::Warning => {
                        log::warn!("Shader warning: {}", msg.message);
                    }
                    wgpu::CompilationMessageType::Info => {
                        log::info!("Shader info: {}", msg.message);
                    }
                }
            }
        }

        Ok(Self {
            module,
            entry_point: "main".to_string(),
            stage: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
        })
    }

    /// Create a shader from SPIR-V binary
    pub fn from_spirv(device: &wgpu::Device, data: &[u32], label: &str) -> Result<Self> {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(label),
            source: wgpu::ShaderSource::SpirV(std::borrow::Cow::Borrowed(data)),
        });

        Ok(Self {
            module,
            entry_point: "main".to_string(),
            stage: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
        })
    }

    /// Get the shader module
    pub fn module(&self) -> &wgpu::ShaderModule {
        &self.module
    }

    /// Set entry point
    pub fn with_entry_point(mut self, entry: &str) -> Self {
        self.entry_point = entry.to_string();
        self
    }

    /// Get entry point
    pub fn entry_point(&self) -> &str {
        &self.entry_point
    }

    /// Set shader stage
    pub fn with_stage(mut self, stage: wgpu::ShaderStages) -> Self {
        self.stage = stage;
        self
    }
}

/// Shader library for managing multiple shaders
pub struct ShaderLibrary {
    shaders: std::collections::HashMap<String, wgpu::ShaderModule>,
}

impl ShaderLibrary {
    /// Create a new shader library
    pub fn new() -> Self {
        Self {
            shaders: std::collections::HashMap::new(),
        }
    }

    /// Load a shader from WGSL source
    pub fn load_wgsl(&mut self, device: &wgpu::Device, name: &str, source: &str) -> Result<()> {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(name),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });

        self.shaders.insert(name.to_string(), module);
        info!("Loaded shader: {}", name);
        Ok(())
    }

    /// Get a shader by name
    pub fn get(&self, name: &str) -> Option<&wgpu::ShaderModule> {
        self.shaders.get(name)
    }

    /// Check if shader exists
    pub fn has(&self, name: &str) -> bool {
        self.shaders.contains_key(name)
    }

    /// Remove a shader
    pub fn remove(&mut self, name: &str) -> bool {
        self.shaders.remove(name).is_some()
    }

    /// Clear all shaders
    pub fn clear(&mut self) {
        self.shaders.clear();
    }
}

impl Default for ShaderLibrary {
    fn default() -> Self {
        Self::new()
    }
}

/// Predefined shader templates
pub mod templates {
    /// Basic 3D shader with position, normal, and color
    pub const BASIC_3D: &str = include_str!("shader.wgsl");

    /// Simple 2D texture shader
    pub const TEXTURE_2D: &str = r#"
        struct VertexInput {
            @location(0) position: vec2<f32>,
            @location(1) uv: vec2<f32>,
        };

        struct VertexOutput {
            @builtin(position) clip_position: vec4<f32>,
            @location(0) uv: vec2<f32>,
        };

        @vertex
        fn vs_main(input: VertexInput) -> VertexOutput {
            var output: VertexOutput;
            output.clip_position = vec4<f32>(input.position, 0.0, 1.0);
            output.uv = input.uv;
            return output;
        }

        @group(0) @binding(0)
        var t_diffuse: texture_2d<f32>;
        @group(0) @binding(1)
        var s_diffuse: sampler;

        @fragment
        fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
            return textureSample(t_diffuse, s_diffuse, input.uv);
        }
    "#;

    /// Solid color shader
    pub const SOLID_COLOR: &str = r#"
        struct VertexInput {
            @location(0) position: vec3<f32>,
        };

        struct Uniforms {
            mvp: mat4x4<f32>,
            color: vec4<f32>,
        };

        @group(0) @binding(0)
        var<uniform> uniforms: Uniforms;

        @vertex
        fn vs_main(input: VertexInput) -> @builtin(position) vec4<f32> {
            return uniforms.mvp * vec4<f32>(input.position, 1.0);
        }

        @fragment
        fn fs_main() -> @location(0) vec4<f32> {
            return uniforms.color;
        }
    "#;
}

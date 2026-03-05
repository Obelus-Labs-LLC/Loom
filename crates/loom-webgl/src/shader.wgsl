// 3D Rendering Shader
// Phase L17: WebGL/GPU Acceleration

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
};

struct Uniforms {
    model: mat4x4<f32>,
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    
    let world_position = uniforms.model * vec4<f32>(input.position, 1.0);
    output.world_position = world_position.xyz;
    output.clip_position = uniforms.projection * uniforms.view * world_position;
    
    // Transform normal to world space
    output.world_normal = normalize((uniforms.model * vec4<f32>(input.normal, 0.0)).xyz);
    
    return output;
}

// Fragment shader with basic lighting
const LIGHT_DIR: vec3<f32> = vec3<f32>(0.5, 1.0, 0.3);
const BASE_COLOR: vec3<f32> = vec3<f32>(0.2, 0.5, 0.8);

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Normalize light direction
    let light_dir = normalize(LIGHT_DIR);
    let normal = normalize(input.world_normal);
    
    // Simple Lambertian diffuse lighting
    let diffuse = max(dot(normal, light_dir), 0.0);
    
    // Ambient light
    let ambient = 0.2;
    
    // Final color
    let lighting = ambient + diffuse * 0.8;
    let color = BASE_COLOR * lighting;
    
    return vec4<f32>(color, 1.0);
}

// Alternative: Wireframe shader
@fragment
fn fs_wireframe(input: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 1.0, 0.0, 1.0); // Green wireframe
}

// Alternative: Normal visualization shader
@fragment
fn fs_normals(input: VertexOutput) -> @location(0) vec4<f32> {
    let normal = normalize(input.world_normal);
    // Map [-1, 1] to [0, 1]
    let color = normal * 0.5 + 0.5;
    return vec4<f32>(color, 1.0);
}

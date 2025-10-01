// Shader WGSL pour le rendu des triangles Model 2

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
}

// Matrices de transformation 3D
struct Matrices {
    model: mat4x4<f32>,
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
}

@group(1) @binding(0)
var<uniform> matrices: Matrices;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    
    // Appliquer les transformations 3D: Projection * View * Model * Position
    let model_view = matrices.view * matrices.model;
    let mvp = matrices.projection * model_view;
    output.clip_position = mvp * vec4<f32>(input.position, 1.0);
    
    output.tex_coords = input.tex_coords;
    output.color = input.color;
    
    return output;
}

@group(0) @binding(0)
var texture_diffuse: texture_2d<f32>;

@group(0) @binding(1)
var sampler_diffuse: sampler;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Échantillonner la texture
    let texture_color = textureSample(texture_diffuse, sampler_diffuse, input.tex_coords);
    
    // Combiner avec la couleur du vertex (éclairage Gouraud)
    let final_color = texture_color * input.color;
    
    return final_color;
}
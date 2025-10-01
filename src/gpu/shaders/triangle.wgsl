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

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    
    // Transformation directe pour l'instant (les matrices seront ajoutées plus tard)
    output.clip_position = vec4<f32>(input.position, 1.0);
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
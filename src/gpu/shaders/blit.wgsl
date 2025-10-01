// Shader WGSL pour la copie de framebuffer vers l'écran

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var output: VertexOutput;
    
    // Génération procédurale d'un quad plein écran
    let x = f32(i32(vertex_index) / 2) * 4.0 - 1.0;
    let y = f32(i32(vertex_index) & 1) * 4.0 - 1.0;
    
    output.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    output.tex_coords = vec2<f32>((x + 1.0) * 0.5, 1.0 - (y + 1.0) * 0.5);
    
    return output;
}

@group(0) @binding(0)
var framebuffer_texture: texture_2d<f32>;

@group(0) @binding(1)
var framebuffer_sampler: sampler;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Copie directe du framebuffer
    return textureSample(framebuffer_texture, framebuffer_sampler, input.tex_coords);
}
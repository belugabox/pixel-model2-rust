// Shader WGSL simplifié pour le rendu de triangles sans textures

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    // Transformation directe (sera remplacée par des matrices de projection plus tard)
    output.clip_position = vec4<f32>(input.position, 1.0);
    output.color = input.color;

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Rendu simple de la couleur du vertex
    return input.color;
}
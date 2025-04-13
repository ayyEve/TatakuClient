struct VertexInputs {
    @location(0) pos: vec2<f32>,
    @location(1) tex_uvs: vec2<f32>,
}

struct VertexOutputs {
    // The position of the vertex
    @builtin(position) position: vec4<f32>,
    @location(0) tex_uvs: vec2<f32>,
}

struct FragmentInputs {
    // The position of the fragment
    @builtin(position) position: vec4<f32>,
    @location(0) tex_uvs: vec2<f32>,
}

@group(0) @binding(0) var s: sampler;
@group(0) @binding(1) var texture: texture_2d<f32>;
@group(1) @binding(0) var<uniform> projection_matrix: mat4x4<f32>;

@vertex
fn vs_main(input: VertexInputs) -> VertexOutputs {
    var output: VertexOutputs;

    output.position = projection_matrix * vec4<f32>(input.pos, 0.0, 1.0);
    output.tex_uvs = input.tex_uvs;

    return output;
}

@fragment
fn fs_main(input: FragmentInputs) -> @location(0) vec4<f32> {
    return textureSample(texture, s, input.tex_uvs);
}

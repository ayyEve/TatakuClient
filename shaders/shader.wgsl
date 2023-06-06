struct VertexOutputs {
    //The position of the vertex
    @builtin(position) position: vec4<f32>,
    //The texture cooridnate of the vertex
    @location(0) tex_coord: vec2<f32>,
    //The color of the vertex
    @location(1) tex_index: i32,
    //The color of the vertex
    @location(2) vertex_col: vec4<f32>
}

struct FragmentInputs {
    //Texture coordinate
    @location(0) tex_coord: vec2<f32>,
    // texture index
    @location(1) tex_index: i32,
    //The Vertex color
    @location(2) vertex_col: vec4<f32>
}

@group(0) @binding(0) var<uniform> projection_matrix: mat4x4<f32>;

@vertex
fn vs_main(
    @location(0) pos: vec3<f32>,
    @location(1) tex_coord: vec2<f32>,
    @location(2) tex_index: i32,
    @location(3) vertex_col: vec4<f32>,
) -> VertexOutputs {
    var output: VertexOutputs;

    output.position = projection_matrix * vec4<f32>(pos, 1.0);
    output.tex_coord = tex_coord;
    output.tex_index = tex_index;
    output.vertex_col = vertex_col;

    return output;
}

//TODO: keep an eye on the spec, once we are able to support texture and sampler arrays, PLEASE USE THEM
//The texture we're sampling
@group(1) @binding(0) var t: binding_array<texture_2d<f32>>;
//The sampler we're using to sample the texture
@group(1) @binding(1) var s: sampler;

@fragment
fn fs_main(input: FragmentInputs) -> @location(0) vec4<f32> {
    var i = input.tex_index;
    if (i == -1) { i = 0; }

    let ts = textureSample(t[i], s, input.tex_coord);
    // idk how to make it not use the sampler for non-textures, so we do this instead
    if (input.tex_index == -1) {
        return input.vertex_col;
    } else {
        return ts * input.vertex_col;
    }
}

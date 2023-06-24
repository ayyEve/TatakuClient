struct VertexInputs {
    @location(0) pos: vec3<f32>,
    @location(1) tex_coord: vec2<f32>,
    @location(2) @interpolate(flat) tex_index: i32,
    @location(3) vertex_col: vec4<f32>,
    @location(4) @interpolate(flat) scissor_index: u32,
}

struct VertexOutputs {
    //The position of the vertex
    @builtin(position) position: vec4<f32>,
    //The texture cooridnate of the vertex
    @location(0) tex_coord: vec2<f32>,
    // The index of the texture
    @location(1) @interpolate(flat) tex_index: i32,
    //The color of the vertex
    @location(2) vertex_col: vec4<f32>,
    // index of scissor
    @location(3) @interpolate(flat) scissor_index: u32,
}

struct FragmentInputs {
    //The position of the fragment
    @builtin(position) position: vec4<f32>,
    // Texture coordinate
    @location(0) tex_coord: vec2<f32>,
    // texture index
    @location(1) tex_index: i32,
    // The Vertex color
    @location(2) vertex_col: vec4<f32>,
    // index of scissor
    @location(3) scissor_index: u32,
}

@group(0) @binding(0) var<uniform> projection_matrix: mat4x4<f32>;

@vertex
fn vs_main(input: VertexInputs) -> VertexOutputs {
    var output: VertexOutputs;

    output.position = projection_matrix * vec4<f32>(input.pos, 1.0);
    output.tex_coord = input.tex_coord;
    output.tex_index = input.tex_index;
    output.vertex_col = input.vertex_col;
    output.scissor_index = input.scissor_index;

    return output;
}

// helper struct since apparently vec4<f32> is incompatible with the address space Handle
struct Scissor {
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
}

@group(1) @binding(0) var s: sampler;
@group(1) @binding(1) var texture1: texture_2d<f32>;
@group(1) @binding(2) var texture2: texture_2d<f32>;
@group(1) @binding(3) var texture3: texture_2d<f32>;
@group(1) @binding(4) var texture4: texture_2d<f32>;
@group(2) @binding(0) var<uniform> scissors: array<Scissor, 3000>;

@fragment
fn fs_main(input: FragmentInputs) -> @location(0) vec4<f32> {
    var i = input.tex_index;
    if (i == -1) { i = 0; }

    let ts1 = textureSample(texture1, s, input.tex_coord);
    let ts2 = textureSample(texture2, s, input.tex_coord);
    let ts3 = textureSample(texture3, s, input.tex_coord);
    let ts4 = textureSample(texture4, s, input.tex_coord);
    var ts = ts1;

    switch input.tex_index {
        case 1: { ts = ts2; }
        case 2: { ts = ts3; }
        case 3: { ts = ts4; }
        default: { ts = ts1; }
    }
 
    if (input.scissor_index > 0u) {
        let scissor = scissors[input.scissor_index - 1u];
        if (input.position.x < scissor.x1 
        || input.position.x > scissor.x2
        || input.position.y < scissor.y1 
        || input.position.y > scissor.y2) {
            discard;
        }
    }

    // idk how to make it not use the sampler for non-textures, so we do this instead
    if (input.tex_index == -1) {
        return input.vertex_col;
    } else {
        return ts * input.vertex_col;
    }
}

struct VertexInput {
    @location(0) pos: vec2<f32>,
    
    @location(1) flashlight_index: u32,
}

struct FragmentInput {
    //The position of the fragment
    @builtin(position) position: vec4<f32>,

    @location(0) flashlight_index: u32,
}

struct FlashlightData {
    cursor_pos: array<f32, 2>,

    flashlight_radius: f32,
    fade_radius: f32,

    color: array<f32, 4>,
}


@group(0) @binding(0) var<uniform> projection_matrix: mat4x4<f32>;

@vertex
fn flashlight_vs_main(input: VertexInput) -> FragmentInput {
    var output: FragmentInput;

    output.position = projection_matrix * vec4<f32>(input.pos, 0.0, 1.0);
    output.flashlight_index = input.flashlight_index;

    return output;
}


// Per flashlight data
@group(1) @binding(0) var<storage> flashlight_data: array<FlashlightData>;

@fragment
fn flashlight_fs_main(input: FragmentInput) -> @location(0) vec4<f32> {
    let flashlight = flashlight_data[input.flashlight_index];
    let cursor_pos = vec2(flashlight.cursor_pos[0], flashlight.cursor_pos[1]);

    let r_sq = pow(flashlight.flashlight_radius, 2.0);
    let fr_sq = pow(flashlight.flashlight_radius + flashlight.fade_radius, 2.0);

    let dist = distance_sq(cursor_pos, input.position.xy);

    if dist < r_sq {
        discard;
        // return vec4(0.0, 0.0, 0.0, 0.0);
    } else {
        var color = vec4(
            flashlight.color[0],
            flashlight.color[1],
            flashlight.color[2],
            flashlight.color[3]
        );

        if dist < fr_sq {
            let d = sqrt(dist) - flashlight.flashlight_radius;
            color.a = d / flashlight.fade_radius;
        }

        return color;
    }
}

fn distance_sq(a: vec2<f32>, b: vec2<f32>) -> f32 {
    let c = b-a;
    return dot(c, c);
}

// yoinked from https://www.shadertoy.com/view/llGSz3
// all credit to stoyan3d

struct Settings {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    size: u32,
};
struct Orientation {
    vertical: u32,
};

@group(0) @binding(0) var<uniform> settings: Settings;
@group(1) @binding(0) var input_texture: texture_2d<f32>;
@group(1) @binding(1) var output_texture: texture_storage_2d<bgra8unorm, write>;
@group(1) @binding(2) var<uniform> orientation: Orientation;


@compute
@workgroup_size(16, 16)
fn main( 
    @builtin(global_invocation_id) global_id: vec3<u32>,
) {
    let frag_coord = vec2<i32>(global_id.xy);

    let dimensions = textureDimensions(input_texture);

    if global_id.x >= dimensions.x || global_id.y >= dimensions.y {
        return;
    }

    if (outside_bounds(frag_coord)) {
        let color = textureLoad(input_texture, frag_coord, 0);
        textureStore(output_texture, frag_coord, color);
        return;
    }

    var accumulation = vec3<f32>(0.0);
    if orientation.vertical > 0 {
        accumulation = vertical(frag_coord);
    } else {
        accumulation = horizontal(frag_coord);
    }

    let sum = accumulation / (f32(settings.size) * 2.0 + 1.0);
    textureStore(output_texture, frag_coord, vec4<f32>(sum, 1));
}


fn vertical(frag_coord: vec2<i32>) -> vec3<f32> {
    let size = i32(settings.size);
    let height = textureDimensions(input_texture).y;

    var accumulation = vec3<f32>(0.0);

    for (var i = -size; i <= size; i++) {
        var pos = frag_coord + vec2<i32>(0, i);
        pos.y = clamp(pos.y, 0, i32(height));

        let color = textureLoad(input_texture, pos, 0);
        accumulation += color.rgb;
    }

    return accumulation;
}

fn horizontal(frag_coord: vec2<i32>) -> vec3<f32> {
    let size = i32(settings.size);
    let width = textureDimensions(input_texture).x;

    var accumulation = vec3<f32>(0.0);

    for (var i = -size; i <= size; i++) {
        var pos = frag_coord + vec2<i32>(i, 0);
        pos.x = clamp(pos.x, 0, i32(width));

        let color = textureLoad(input_texture, pos, 0);
        accumulation += color.rgb;
    }
    
    return accumulation;
}

fn outside_bounds(pos: vec2<i32>) -> bool {
    return pos.y < i32(settings.y) 
        || pos.y >= i32(settings.y + settings.height)
        || pos.x < i32(settings.x) 
        || pos.x >= i32(settings.x + settings.width);
}

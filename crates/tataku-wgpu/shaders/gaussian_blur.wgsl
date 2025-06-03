// yoinked from https://github.com/redwarp/filters/blob/main/core/src/blur.rs
// all credit to redwarp

struct Settings {
    filter_size: u32,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
};

struct Orientation {
    vertical: u32,
};

struct Kernel {
  sum: f32,
  values: array<f32>,
};

@group(0) @binding(0) var<uniform> settings: Settings;
@group(0) @binding(1) var<storage, read> kernel: Kernel;
@group(1) @binding(0) var input_texture: texture_2d<f32>;
@group(1) @binding(1) var output_texture: texture_storage_2d<bgra8unorm, write>;
@group(1) @binding(2) var<uniform> orientation: Orientation;

@compute
@workgroup_size(128)
fn main(
  @builtin(global_invocation_id) global_id : vec3<u32>,
) {
    let filter_radius = i32((settings.filter_size - 1u) / 2u);
    let filter_size = i32(settings.filter_size);
    let dimensions = textureDimensions(input_texture);
    var position = vec2<i32>(global_id.xy);

    if (orientation.vertical == 0u) {
        position = position.yx;
    }
    
    if (position.x >= i32(dimensions.x) || position.y >= i32(dimensions.y)) {
        return;
    }

    if (
        position.x < i32(settings.x) || position.x > i32(settings.x + settings.width)
        || position.y < i32(settings.y) || position.y > i32(settings.y + settings.height)
    ) {
        let original = textureLoad(input_texture, position, 0);
        textureStore(output_texture, position, original);
        return;
    }

    var color = vec4<f32>(0.0);
    if (orientation.vertical == 0u) {
        color = horizontal(position, filter_size, filter_radius);
    } else {
        color = vertical(position, filter_size, filter_radius);
    }

    textureStore(output_texture, position, color);
}


fn horizontal(
    position: vec2<i32>,
    filter_size: i32,
    filter_radius: i32,
) -> vec4<f32> {
    let original = textureLoad(input_texture, position, 0);
    var color = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    
    for (var i: i32 = 0; i < filter_size; i++) {
        let x = position.x - filter_radius + i;
        color = color + kernel.values[i] * textureLoad(input_texture, vec2<i32>(x, position.y), 0);
    }
    return color / kernel.sum;
}

fn vertical(
    position: vec2<i32>,
    filter_size: i32,
    filter_radius: i32,
) -> vec4<f32> {
    let original = textureLoad(input_texture, position, 0);
    var color = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    
    for (var i: i32 = 0; i < filter_size; i++) {
        let y = position.y - filter_radius + i;
        color = color + kernel.values[i] * textureLoad(input_texture, vec2<i32>(position.x, y), 0);
    }
    return color / kernel.sum;
}

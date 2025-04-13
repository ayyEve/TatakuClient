@group(0) @binding(0) var<storage, read_write> input: array<u32>;
@group(0) @binding(1) var<uniform> width: u32;

@compute
@workgroup_size(1, 1)
fn main(
  @builtin(global_invocation_id) global_id: vec3<u32>,
) {
    let index = global_id.x + global_id.y * width;

    // Bounds check to avoid out-of-bounds access
    if (index >= arrayLength(&input)) { return; }
    let rgba = input[index];

    // Extract bytes (little-endian: 0xAABBGGRR)
    let r: u32 = (rgba >> 0) & 0xFF;
    let g: u32 = (rgba >> 8) & 0xFF;
    let b: u32 = (rgba >> 16) & 0xFF;
    let a: u32 = (rgba >> 24) & 0xFF;

    input[index] = (a << 24) | (r << 16) | (g << 8) | b;
}

struct ParticleData {
    // how much life did this start out with?
    life_max: f32,
    // how much life is left
    lifetime: f32,

    pos_x: f32,
    pos_y: f32,
    velocity_x: f32,
    velocity_y: f32,

    scale: f32,
    opacity: f32,
    rotation: f32,

    info_index: u32,
    info_index2: u32,
    particle_index: u32,
}

struct EmitterInfo {
    scale_start: f32,
    scale_end: f32,

    opacity_start: f32,
    opacity_end: f32,

    rotation_start: f32,
    rotation_end: f32,

    _1: f32,
    _2: f32,
}

struct RunInfo {
    dt: f32,
}

@group(0) @binding(0) var<uniform> emitter_info: array<EmitterInfo, 300>;
@group(0) @binding(1) var<storage, read_write> particles: array<ParticleData, 300>;
@group(0) @binding(2) var<uniform> run_info: RunInfo;

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let dt = run_info.dt;
    let i = global_id.x;

    particles[i].lifetime -= dt;
    if particles[i].lifetime <= 0.0 { return; }

    // update position
    particles[i].pos_x += particles[i].velocity_x * dt;
    particles[i].pos_y += particles[i].velocity_y * dt;

    // get amount to lerp by
    let amount = 1.0 - (particles[i].lifetime / particles[i].life_max);

    // get info
    var info = emitter_info[particles[i].info_index];

    if info.scale_start != 0.0 || info.scale_end != 0.0 {
        particles[i].scale = lerp(info.scale_start, info.scale_end, amount);
    }
    if info.rotation_start != 0.0 || info.rotation_end != 0.0 {
        particles[i].rotation = lerp(info.rotation_start, info.rotation_end, amount);
    }
    if info.opacity_start != 0.0 || info.opacity_end != 0.0 {
        particles[i].opacity = lerp(info.opacity_start, info.opacity_end, amount);
    }
}

fn lerp(start: f32, end: f32, amount: f32) -> f32 {
    return start + (end - start) * amount;
}
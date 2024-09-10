use tataku_client_common::prelude::*;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuParticle {
    // how much life did this start out with?
    pub life_max: f32,
    // how much life is left
    pub lifetime: f32,

    // when i had these as [f32;2], it broke thing and idk why
    pub pos_x: f32,
    pub pos_y: f32,
    pub velocity_x: f32,
    pub velocity_y: f32,

    pub scale: f32,
    pub opacity: f32,
    pub rotation: f32,

    /// index of the emitter info gpu side
    pub info_index: u32,
    /// index of the emitter cpu side
    pub emitter_index: u32,

    /// the index of the particle within the particle pool cpu side
    pub particle_index: u32,
}
impl GpuParticle {
    pub fn new(p: &PoolEntry<Particle>, info: u32, emitter: u32) -> Self {
        Self {
            life_max: p.lifetime_max,
            lifetime: p.lifetime,
            pos_x: p.position.x,
            pos_y: p.position.y,
            velocity_x: p.velocity.x,
            velocity_y: p.velocity.y,

            scale: p.scale,
            rotation: p.rotation,
            opacity: p.color.a,

            info_index: info,
            emitter_index: emitter,
            particle_index: p.get_index() as u32
        }
    }

    pub const fn count_size(count:usize) -> usize {
        std::mem::size_of::<Self>() * count
    }
}

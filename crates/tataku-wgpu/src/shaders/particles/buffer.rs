
use crate::prelude::*;

pub const SIZE:u64 = 300;

pub struct ParticleBuffer {
    pub particle_buffer: Buffer,
    pub emitter_buffer: Buffer,
    pub run_info_buffer: Buffer,
    pub bind_group: BindGroup,
    pub layout: BindGroupLayout,

    pub readable_particle_buffer: Buffer,

    pub index: usize,

    pub particle_count: usize,

    // recording_periods_since_last_use: usize
}

impl ParticleBuffer {
    pub fn new(device: &Device, index: usize) -> Self {
        let emitter_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Emittor Info Buffer"),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            size: SIZE * std::mem::size_of::<EmitterInfoInner>() as u64,
            mapped_at_creation: false,
        });

        let particle_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Particle Buffer"),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
            size: SIZE * std::mem::size_of::<GpuParticle>() as u64,
            mapped_at_creation: false,
        });

        let run_info_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Run Info Buffer"),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            size: std::mem::size_of::<RunInfoInner>() as u64,
            mapped_at_creation: false,
        });

        let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("ghjkdfs"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: std::num::NonZeroU64::new(SIZE * std::mem::size_of::<EmitterInfoInner>() as u64)
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: std::num::NonZeroU64::new(SIZE * std::mem::size_of::<GpuParticle>() as u64)
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: std::num::NonZeroU64::new(std::mem::size_of::<RunInfoInner>() as u64)
                    },
                    count: None,
                }
            ]
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("hgoifdshgijfds"),
            layout: &layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: emitter_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: particle_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: run_info_buffer.as_entire_binding(),
                }
            ],
        });


        let readable_particle_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Particle Buffer 2"),
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            size: SIZE * std::mem::size_of::<GpuParticle>() as u64,
            mapped_at_creation: false,
        });

        Self {
            particle_buffer,
            emitter_buffer,
            run_info_buffer,
            bind_group,
            layout,
            index,
            particle_count: 0,

            readable_particle_buffer,
        }
    }
}

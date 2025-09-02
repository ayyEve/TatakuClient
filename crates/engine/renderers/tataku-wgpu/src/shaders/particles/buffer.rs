use crate::prelude::*;

pub(crate) const SIZE:u64 = 300;

pub(crate) struct Buffer {
    pub particle_buffer: wgpu::Buffer,
    pub emitter_buffer: wgpu::Buffer,
    pub run_info_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub layout: wgpu::BindGroupLayout,

    pub readable_particle_buffer: wgpu::Buffer,
    pub index: usize,
    pub particle_count: usize,
}
impl Buffer {
    pub fn new(device: &wgpu::Device, index: usize) -> Self {
        let emitter_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Emittor Info Buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            size: SIZE * std::mem::size_of::<super::GpuEmitterInfo>() as u64,
            mapped_at_creation: false,
        });

        let particle_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Particle Buffer"),
            usage: wgpu::BufferUsages::STORAGE 
                | wgpu::BufferUsages::COPY_DST 
                | wgpu::BufferUsages::COPY_SRC,
            size: SIZE * std::mem::size_of::<super::GpuParticle>() as u64,
            mapped_at_creation: false,
        });

        let run_info_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Run Info Buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            size: std::mem::size_of::<super::GpuRunInfo>() as u64,
            mapped_at_creation: false,
        });

        let layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("ghjkdfs"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: NonZeroU64::new(
                                SIZE * size_of::<super::GpuEmitterInfo>() as u64
                            )
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: NonZeroU64::new(
                                SIZE * size_of::<super::GpuParticle>() as u64
                            )
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: NonZeroU64::new(
                                size_of::<super::GpuRunInfo>() as u64
                            )
                        },
                        count: None,
                    }
                ]
            }
        );

        let bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("hgoifdshgijfds"),
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: emitter_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: particle_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: run_info_buffer.as_entire_binding(),
                    }
                ],
            }
        );


        let readable_particle_buffer = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("Particle Buffer 2"),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                size: SIZE * size_of::<super::GpuParticle>() as u64,
                mapped_at_creation: false,
            }
        );

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

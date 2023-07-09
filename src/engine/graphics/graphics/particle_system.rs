use crate::prelude::*;
use wgpu::BindGroupDescriptor;
use wgpu::BindGroupLayout;
use wgpu::BindGroupLayoutDescriptor;
use wgpu::Buffer;
use wgpu::BindGroup;
use wgpu::ComputePassDescriptor;
use wgpu::PipelineLayoutDescriptor;
use wgpu::ShaderStages;

const SIZE:u64 = 300;

pub struct ParticleSystem {
    emitters: Vec<EmitterRef>,
    last_update: Instant,

    pipeline: wgpu::ComputePipeline,

    // are we waiting for the gpu to compute?
    datas_pending: usize,
    sender: SyncSender<(usize, usize)>,
    receiver: Receiver<(usize, usize)>,

    used_buffers: Vec<ParticleBuffer>,
    available_buffers: Vec<ParticleBuffer>,
    current_buffer: Option<ParticleBuffer>,

    cpu_info_buffer: Vec<EmitterInfo>,
    cpu_particle_buffer: Vec<GpuParticle>,
}

impl ParticleSystem {
    pub fn new(device: &wgpu::Device) -> Self {
        let particle_compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Particle Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../../../shaders/particles.wgsl").into()),
        });

        let buffer = ParticleBuffer::new(device, 0);

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Compute Pipeline layout"),
            bind_group_layouts: &[ &buffer.layout ],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Particle compute pipeline"),
            layout: Some(&pipeline_layout),
            module: &particle_compute_shader,
            entry_point: "main",
        });
        
        let (sender, receiver) = sync_channel(1000);
        Self {
            emitters: Vec::new(),
            last_update: Instant::now(),

            pipeline,

            sender,
            receiver,
            datas_pending: 0,

            cpu_info_buffer: Vec::with_capacity(SIZE as usize),
            cpu_particle_buffer: Vec::with_capacity(SIZE as usize),
            current_buffer: Some(buffer),

            used_buffers: Vec::new(),
            available_buffers: Vec::new(),
        }
    }
    pub fn add(&mut self, emitter: EmitterRef) {
        self.emitters.push(emitter);
    }

    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if self.datas_pending > 0 {
            if let Ok((index, written_size)) = self.receiver.try_recv() {
                if let Some(buffer) = self.receive_from_gpu(index, written_size) {
                    buffer.readable_particle_buffer.unmap();
                }
                self.datas_pending -= 1;
            }

            return;
        }

        self.send_to_gpu(device, queue);
    }
    fn get_buffer_from_index(&self, index: usize) -> Option<&ParticleBuffer> {
        self.used_buffers.iter().find(|b|b.index == index)
    }

    /// returns the buffer if it should be unmapped
    fn receive_from_gpu(&self, index: usize, written_size: usize) -> Option<&ParticleBuffer> {
        let Some(buffer) = self.get_buffer_from_index(index) else { return None }; 

        // this shouldnt happen (anymore) but is here as a failsafe
        if GpuParticle::count_size(buffer.particle_count) != written_size { 
            warn!("got disagreeing written particle count"); 
            return None
        }

        let data = buffer.readable_particle_buffer.slice(..written_size as u64).get_mapped_range();

        let particles:Vec<GpuParticle> = data.chunks_exact(GpuParticle::count_size(1)).map(|bytes|bytemuck::cast_slice(bytes)[0]).collect();
        let emitters = self.emitters.iter().enumerate().map(|(n, i)|(n, i.0.upgrade().map(|e|e))).collect::<HashMap<_,_>>();
        
        for particle in particles {
            let Some(Some(emitter)) = emitters.get(&(particle.emitter_index as usize)) else { continue };
            let mut lock = emitter.write();
            if particle.lifetime <= 0.0 { lock.remove(particle.particle_index as usize); continue }

            let Some(cpu_p) = lock.get(particle.particle_index as usize) else { continue };
            cpu_p.lifetime = particle.lifetime;
            cpu_p.position.x = particle.pos_x;
            cpu_p.position.y = particle.pos_y;
            
            cpu_p.scale = particle.scale;
            cpu_p.rotation = particle.rotation;
            cpu_p.color.a = particle.opacity;
        }

        Some(buffer)
    }


    fn send_to_gpu(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let delta = self.last_update.elapsed_and_reset();
        self.available_buffers.extend(std::mem::take(&mut self.used_buffers));
        if self.current_buffer.is_none() {
            self.current_buffer = self.available_buffers.pop();
        }
        self.cpu_info_buffer.clear();
        self.cpu_particle_buffer.clear();

        let mut emitters = self.emitters.clone();
        let mut emitter_index = 0;
        emitters.retain(|emitter| {
            let info = emitter.1;
            let Some(emitter) = emitter.0.upgrade() else { return false };

            if self.cpu_particle_buffer.len() as u64 + 1 >= SIZE { 
                self.next_buffer(device, queue, delta);
            }

            let mut info_index = self.cpu_info_buffer.len() as u32;
            self.cpu_info_buffer.push(info);
            
            if let Some(pool) = emitter.try_read() {
                pool.iter_used().for_each(|p| {
                    if self.cpu_particle_buffer.len() as u64 + 1 >= SIZE { 
                        self.next_buffer(device, queue, delta);
                        
                        self.cpu_info_buffer.push(info);
                        info_index = 0;
                    }
                    self.cpu_particle_buffer.push(GpuParticle::new(p, info_index, emitter_index));
                });
            }

            emitter_index += 1;
            true
        });
        self.next_buffer(device, queue, delta);
        self.emitters = emitters;

        if self.used_buffers.is_empty() { return; }

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Render Encoder") });

        for buffer in &self.used_buffers {
            {
                let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor { label: Some("Particles") });
                compute_pass.set_pipeline(&self.pipeline);
                compute_pass.set_bind_group(0, &buffer.bind_group, &[]);
                compute_pass.dispatch_workgroups(buffer.particle_count as u32, 1, 1);
            }
            
            let write_size = GpuParticle::count_size(buffer.particle_count);
            encoder.copy_buffer_to_buffer(&buffer.particle_buffer, 0, &buffer.readable_particle_buffer, 0, write_size as wgpu::BufferAddress);
        }

        queue.submit([encoder.finish()]);

        for buffer in &self.used_buffers {
            let index = buffer.index;
            let write_size = GpuParticle::count_size(buffer.particle_count);
            let s = self.sender.clone();
            
            buffer.readable_particle_buffer.slice(..write_size as u64)
                .map_async(wgpu::MapMode::Read, move |_|s.send((index, write_size)).unwrap());
        }

        self.datas_pending = self.used_buffers.len();
    }

    fn next_buffer(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, delta: f32) {
        if self.cpu_particle_buffer.is_empty() { return }

        let mut buffer = std::mem::take(&mut self.current_buffer).unwrap_or_else(||ParticleBuffer::new(device, self.used_buffers.len()));
        buffer.particle_count = self.cpu_particle_buffer.len();

        queue.write_buffer(&buffer.emitter_buffer, 0, bytemuck::cast_slice(&self.cpu_info_buffer));
        queue.write_buffer(&buffer.particle_buffer, 0, bytemuck::cast_slice(&self.cpu_particle_buffer));
        queue.write_buffer(&buffer.run_info_buffer, 0, bytemuck::cast_slice(&[RunInfo::new(delta)]));
        self.used_buffers.push(buffer);

        self.current_buffer = self.available_buffers.pop();

        self.cpu_info_buffer.clear();
        self.cpu_particle_buffer.clear();
    }



}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuParticle {
    // how much life did this start out with?
    life_max: f32,
    // how much life is left
    lifetime: f32,

    // when i had these as [f32;2], it broke thing and idk why
    pos_x: f32,
    pos_y: f32,
    velocity_x: f32,
    velocity_y: f32,

    scale: f32,
    opacity: f32,
    rotation: f32,

    /// index of the emitter info gpu side
    info_index: u32,
    /// index of the emitter cpu side
    emitter_index: u32,

    /// the index of the particle within the particle pool cpu side
    particle_index: u32,
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

    const fn count_size(count:usize) -> usize {
        std::mem::size_of::<Self>() * count
    }
}

pub struct ParticleBuffer {
    pub particle_buffer: Buffer,
    pub emitter_buffer: Buffer,
    pub run_info_buffer: Buffer,
    pub bind_group: BindGroup,
    pub layout: BindGroupLayout,

    pub readable_particle_buffer: Buffer,

    index: usize,

    particle_count: usize,

    // recording_periods_since_last_use: usize
}
impl ParticleBuffer {
    pub fn new(device: &wgpu::Device, index: usize) -> Self {

        let emitter_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Emittor Info Buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            size: SIZE * std::mem::size_of::<EmitterInfo>() as u64,
            mapped_at_creation: false,
        });

        let particle_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Particle Buffer"),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            size: SIZE * std::mem::size_of::<GpuParticle>() as u64,
            mapped_at_creation: false,
        });

        let run_info_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Run Info Buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            size: std::mem::size_of::<RunInfo>() as u64,
            mapped_at_creation: false,
        });

        let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("ghjkdfs"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Uniform, 
                        has_dynamic_offset: false, 
                        min_binding_size: std::num::NonZeroU64::new(SIZE * std::mem::size_of::<EmitterInfo>() as u64)
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Storage { read_only: false }, 
                        has_dynamic_offset: false, 
                        min_binding_size: std::num::NonZeroU64::new(SIZE * std::mem::size_of::<GpuParticle>() as u64)
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Uniform, 
                        has_dynamic_offset: false, 
                        min_binding_size: std::num::NonZeroU64::new(std::mem::size_of::<RunInfo>() as u64)
                    },
                    count: None,
                }
            ]
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
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
        });


        let readable_particle_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Particle Buffer 2"),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
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


#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct EmitterInfo {
    scale_start: f32,
    scale_end: f32,

    opacity_start: f32,
    opacity_end: f32,

    rotation_start: f32,
    rotation_end: f32,

    _1: f32,
    _2: f32,
}
impl EmitterInfo {
    pub fn new(scale: &EmitterVal, opacity: &EmitterVal, rotation: &EmitterVal) -> Self {
        Self {
            scale_start: scale.range.start,
            scale_end: scale.range.end,

            opacity_start: opacity.range.start,
            opacity_end: opacity.range.end,

            rotation_start: rotation.range.start,
            rotation_end: rotation.range.end,

            _1: 0.0,
            _2: 0.0,
        }
    }
}


#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RunInfo { dt: f32 }
impl RunInfo {
    pub fn new(dt: f32) -> Self { Self { dt } }
}

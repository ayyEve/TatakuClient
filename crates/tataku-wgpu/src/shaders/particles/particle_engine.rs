
use crate::prelude::*;
use tataku_client_common::prelude::*;

use wgpu::{
    ComputePassDescriptor,
    PipelineLayoutDescriptor,
};

use std::{collections::HashMap, sync::mpsc::{sync_channel, Receiver, SyncSender}};

pub struct ParticleSystem {
    emitters: Vec<Box<dyn EmitterReference>>,
    last_update: Instant,

    pipeline: wgpu::ComputePipeline,

    // are we waiting for the gpu to compute?
    datas_pending: usize,
    sender: SyncSender<(usize, usize)>,
    receiver: Receiver<(usize, usize)>,

    used_buffers: Vec<ParticleBuffer>,
    available_buffers: Vec<ParticleBuffer>,
    current_buffer: Option<ParticleBuffer>,

    cpu_info_buffer: Vec<EmitterInfoInner>,
    cpu_particle_buffer: Vec<GpuParticle>,
}

impl ParticleSystem {
    pub fn new(device: &wgpu::Device) -> Self {
        let particle_compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Particle Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(crate::shader_files::PARTICLES.into()),
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
            compilation_options: wgpu::PipelineCompilationOptions::default(),
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
    pub fn add(&mut self, emitter: Box<dyn EmitterReference>) {
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
        let buffer = self.get_buffer_from_index(index)?;

        // this shouldnt happen (anymore) but is here as a failsafe
        if GpuParticle::count_size(buffer.particle_count) != written_size {
            warn!("got disagreeing written particle count");
            return None
        }

        let data = buffer.readable_particle_buffer.slice(..written_size as u64).get_mapped_range();

        let particles:Vec<GpuParticle> = data.chunks_exact(GpuParticle::count_size(1)).map(|bytes|bytemuck::cast_slice(bytes)[0]).collect();
        let mut emitters = self
            .emitters
            .iter()
            .enumerate()
            .map(|(n, i)| (n, i.get_pool()))
            .collect::<HashMap<_,_>>();

        for particle in particles {
            let Some(Some(pool)) = emitters.get_mut(&(particle.emitter_index as usize)) else { continue };
            if particle.lifetime <= 0.0 { pool.remove(particle.particle_index as usize); continue }

            let Some(cpu_p) = pool.get(particle.particle_index as usize) else { continue };
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

        let mut emitters = std::mem::take(&mut self.emitters);
        let mut emitter_index = 0;
        emitters.retain(|emitter| {
            let info = emitter.get_info().into();
            let Some(pool) = emitter.get_pool() else { return false };

            if self.cpu_particle_buffer.len() as u64 + 1 >= SIZE {
                self.next_buffer(device, queue, delta);
            }

            let mut info_index = self.cpu_info_buffer.len() as u32;
            self.cpu_info_buffer.push(info);

            // if let Some(pool) = emitter.try_read() {
                pool.iter_used().for_each(|p| {
                    if self.cpu_particle_buffer.len() as u64 + 1 >= SIZE {
                        self.next_buffer(device, queue, delta);

                        self.cpu_info_buffer.push(info);
                        info_index = 0;
                    }
                    self.cpu_particle_buffer.push(GpuParticle::new(p, info_index, emitter_index));
                });
            // }

            emitter_index += 1;
            true
        });
        self.next_buffer(device, queue, delta);
        self.emitters = emitters;

        if self.used_buffers.is_empty() { return; }

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Render Encoder") });

        for buffer in &self.used_buffers {
            {
                let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor { label: Some("Particles"), timestamp_writes: None, });
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

        let mut buffer = std::mem::take(&mut self.current_buffer).unwrap_or_else(|| ParticleBuffer::new(device, self.used_buffers.len()));
        buffer.particle_count = self.cpu_particle_buffer.len();

        queue.write_buffer(&buffer.emitter_buffer, 0, bytemuck::cast_slice(&self.cpu_info_buffer));
        queue.write_buffer(&buffer.particle_buffer, 0, bytemuck::cast_slice(&self.cpu_particle_buffer));
        queue.write_buffer(&buffer.run_info_buffer, 0, bytemuck::cast_slice(&[RunInfoInner::new(delta)]));
        self.used_buffers.push(buffer);

        self.current_buffer = self.available_buffers.pop();

        self.cpu_info_buffer.clear();
        self.cpu_particle_buffer.clear();
    }
}

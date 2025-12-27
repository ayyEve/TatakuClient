use crate::prelude::*;
use wgpu::util::DeviceExt;
use engine::MatrixHelpers;

const MAX_DEPTH:f32 = 8192.0 * 8192.0;

const PROJECTION_MATRIX_SIZE: NonZeroU64 = NonZeroU64::new(
    std::mem::size_of::<[[f32; 4]; 4]>() as u64
).unwrap();

pub(crate) struct ProjectionMatrix {
    matrix: tataku::Matrix,

    pub bind_group: wgpu::BindGroup,
    pub layout: wgpu::BindGroupLayout,

    pub buffer: wgpu::Buffer,
}

impl ProjectionMatrix {
    pub fn new(
        window_size: tataku::Vector2,
        device: &wgpu::Device,
    ) -> Self {
        let layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("Texture/Sampler bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: Some(PROJECTION_MATRIX_SIZE)
                        },
                        count: None,
                    },
                ]
            }
        );

        let matrix = Self::create_projection(window_size);
        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Projection Matrix Buffer"),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                contents: bytemuck::cast_slice(&matrix.to_raw()),
            }
        );

        let bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("Projection Matrix Bind Group"),
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buffer.as_entire_binding(),
                    },
                ],
            }
        );

        Self {
            matrix,
            bind_group,
            layout,
            buffer,
        }
    }

    pub fn update_projection(
        &mut self,
        window_size: tataku::Vector2,
        queue: &wgpu::Queue
    ) {
        self.matrix = Self::create_projection(window_size);
        self.write_projection(self.matrix, queue);
    }

    pub fn write_projection(
        &self,
        projection: tataku::Matrix,
        queue: &wgpu::Queue,
    ) {
        queue.write_buffer(
            &self.buffer,
            0,
            bytemuck::cast_slice(&projection.to_raw())
        );
    }

    pub fn reapply_projection(
        &self,
        queue: &wgpu::Queue,
    ) {
        // reapply the window projection matrix
        self.write_projection(self.matrix, queue);
    } 

    pub fn create_projection(draw_size: tataku::Vector2) -> tataku::Matrix {
        let sx = 2.0 / draw_size.x;
        let sy = -2.0 / draw_size.y;

        // setup depth range
        let far = MAX_DEPTH;
        let near = -far;
        let depth_range = 1.0 / (far - near);

        [
            [sx, 0.0, 0.0, 0.0],
            [0.0, sy, 0.0, 0.0],
            [0.0, 0.0, depth_range, 0.0],
            [-1.0, 1.0, -near * depth_range, 1.0]
        ].into()
    }
}

impl AsRef<wgpu::Buffer> for ProjectionMatrix {
    fn as_ref(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}

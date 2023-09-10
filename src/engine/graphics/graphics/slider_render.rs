use std::mem::size_of;

pub struct SliderRender {
    // todo: make buffers

    // Radius of inner slider body
    pub circle_radius: f32,
    // Width of border around slider body
    pub border_width: f32,

    // Grids for different sliders. Slices of this array represent an individual slider grid,
    // where each value is a slice into the `grid_cells` array.
    pub slider_grids: Vec<GridCell>,
    // Grid cells for different sliders. Slices of this array represent an individual cell,
    // where each value is an index into the `line_segments` array.
    pub grid_cells: Vec<u32>,
    // Line segments of all sliders in the current render
    pub line_segments: Vec<LineSegment>,
}

/// Vertex buffer layout for sliders
#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
#[cfg_attr(feature="graphics", derive(bytemuck::Pod, bytemuck::Zeroable))]
pub struct SliderVertex {
    pub position: [f32; 2],

    pub slider_index: u32,
}

/// Vertex buffer layout for sliders
#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
#[cfg_attr(feature="graphics", derive(bytemuck::Pod, bytemuck::Zeroable))]
pub struct SliderData {
    /// Radius of inner slider body
    pub circle_radius: f32,
    /// Width of border around slider body
    pub border_width: f32,

    /// snaking progress as a percentage (0-1)
    pub snake_percentage: f32,
    
    // slider velocity (neb to describe this properly)
    pub slider_velocity: f32,

    /// Origin position of grid in viewport space
    pub grid_origin: [f32; 2],
    /// Size of the slider in grid units
    pub grid_size: [u32; 2],
    /// Grid cells of this slider. This represents the start index into the
    //// `slider_grids` array, where the length of the slice is the area of the
    // grid, as given by `grid_size`.
    pub grid_index: u32,

    /// Colour of the body of slider
    pub body_color: [f32; 4],
    /// Colour of the border of the slider
    pub border_color: [f32; 4],
}

/// Slice into the index buffer representing the grid cell
#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
#[cfg_attr(feature="graphics", derive(bytemuck::Pod, bytemuck::Zeroable))]
pub struct GridCell {
    /// Starting index for slice in `grid_cells` array
    pub index: u32,
    /// Length of slice in `grid_cells` array
    pub length: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
#[cfg_attr(feature="graphics", derive(bytemuck::Pod, bytemuck::Zeroable))]
pub struct LineSegment {
    pub p1: [f32; 2],
    pub p2: [f32; 2],
}

#[cfg(feature="graphics")]
impl SliderVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        // todo: convert to macro

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // slider index
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Uint32,
                },
            ]
        }
    }
}


#[cfg(feature="graphics")]
pub fn create_slider_pipeline(
    device: &wgpu::Device, 
    config: &wgpu::SurfaceConfiguration,
    projection_matrix_bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    use crate::prelude::SLIDER_BIND_GROUP_LAYOUT;

    let slider_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Slider Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../../../../shaders/slider.wgsl").into()),
    });

    let slider_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("slider group layout"),
        entries: &[
            // slider_data
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(std::mem::size_of::<SliderData>() as u64 * 2)
                },
                count: None,
            },

            // slider_grids
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(std::mem::size_of::<GridCell>() as u64 * 2)
                },
                count: None,
            },

            // grid_cells
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(std::mem::size_of::<u32>() as u64 * 2)
                },
                count: None,
            },

            // line_segments
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(std::mem::size_of::<LineSegment>() as u64 * 2)
                },
                count: None,
            },

        ],
    });
    SLIDER_BIND_GROUP_LAYOUT.set(slider_bind_group_layout).unwrap();

    let slider_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Slider Pipeline Layout"),
        bind_group_layouts: &[
            &projection_matrix_bind_group_layout,
            SLIDER_BIND_GROUP_LAYOUT.get().unwrap(),
        ],
        push_constant_ranges: &[],
    });

    let slider_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(&format!("Slider Pipeline")),
        layout: Some(&slider_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &slider_shader,
            entry_point: "slider_vs_main",
            buffers: &[ SliderVertex::desc() ],
        },
        fragment: Some(wgpu::FragmentState {
            module: &slider_shader,
            entry_point: "slider_fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(crate::prelude::BlendMode::AlphaBlending.get_blend_state()),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    });

    slider_pipeline
}
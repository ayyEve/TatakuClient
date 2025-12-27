use std::ops::Range;
use tataku_engine::*;

const TEX_VIEW_DESC: wgpu::TextureViewDescriptor = wgpu::TextureViewDescriptor {
    label: Some("atlas_texture_view"),
    format: None,
    dimension: None,
    usage: None,
    aspect: wgpu::TextureAspect::All,
    base_mip_level: 0,
    mip_level_count: None,
    base_array_layer: 0,
    array_layer_count: None,
};

pub struct WgpuAtlas {
    atlas: tataku::Atlas,
    /// the last texture is always the font texture
    textures: Vec<wgpu::TextureView>,
    glyph_atlas: GlyphAtlas,

    pub bind_group: wgpu::BindGroup,
    pub layout: wgpu::BindGroupLayout,

    sampler: wgpu::Sampler,
}
impl WgpuAtlas {
    // must not go past 16
    pub const LAYER_COUNT: Range<u32> = 4..16;

    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
    ) -> Self {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        
        let atlas_size = device
            .limits()
            .max_texture_dimension_2d
            .min(8192);
    
        let mut textures = Vec::new();
        for _ in 0..Self::LAYER_COUNT.start {
            textures.push(create_texture(
                device,
                format, 
                atlas_size,
            ));
        }
        let (bind_group, layout) = create_bind_group(
            device, 
            &textures, 
            &sampler
        );


        Self {
            atlas: tataku::Atlas::new(
                atlas_size,
                atlas_size,
                Self::LAYER_COUNT.start - 1,
            ),
            textures,

            glyph_atlas: GlyphAtlas::new(atlas_size),

            bind_group,
            layout,

            sampler,
        }
    }

    pub fn reserve(
        &mut self, 
        width: u32,
        height: u32,
        glyph: bool,
        device: &wgpu::Device,

    ) -> AtlasResult {
        if glyph {
            return self.glyph_atlas.reserve(
                width, 
                height, 
                self.textures.len() as u32 - 1
            );
        }

        if let Some(a) = self.atlas.try_insert(width, height) {
            return AtlasResult::Ok(a);
        }

        if (self.textures.len() as u32) < Self::LAYER_COUNT.end {
            self.add_layer(device);

            if let Some(a) = self.atlas.try_insert(width, height) {
                AtlasResult::Resized(a)
            } else {
                AtlasResult::NoSpace
            }
        } else {
            AtlasResult::NoSpace
        }
    }
    
    pub fn remove(
        &mut self, 
        tex: tataku::TextureReference,
    ) {
        self.atlas.remove_entry(tex);
    }


    pub fn clear_glyphs(&mut self) {
        self.glyph_atlas.clear();
    }


    pub fn get_texture(&self, info: &tataku::TextureReference) -> &wgpu::Texture {
        self.textures
            .get(info.layer as usize)
            .unwrap()
            .texture()
    }

    pub fn dump(
        &self, 
        engine: &crate::WgpuEngine,
        path: &str,
    ) {
        std::fs::create_dir_all(path).unwrap();

        for (n, tex) in self
            .textures
            .iter()
            .enumerate()
        {
            let tex = tex.texture();
            println!("Reading atlas {n}");
            let (data, [width, height]) = engine.texture_to_bytes(tex);

            let path = format!("{path}/atlas_{n}.png");
            let file = std::fs::File::create(&path).unwrap();
            let png = image::codecs::png::PngEncoder::new(file);

            use image::ImageEncoder;
            png.write_image(
                &data,
                width,
                height,
                image::ExtendedColorType::Rgba8,
            ).unwrap();
        }

    }


    fn update_bind_group(
        &mut self,
        device: &wgpu::Device,
    ) {
        let (bind_group, layout) = create_bind_group(
            device, 
            &self.textures, 
            &self.sampler,
        );

        self.bind_group = bind_group;
        self.layout = layout;
    }

    fn add_layer(
        &mut self, 
        device: &wgpu::Device,
    ) {
        let first = self.textures[0].texture();
        let tex = create_texture(
            device, 
            first.format(),
            first.width()
        );

        self.atlas.add_layer();
        self.textures.push(tex);
        self.update_bind_group(device);
    }
}

fn create_bind_group(
    device: &wgpu::Device,
    view_list: &[ wgpu::TextureView ],
    sampler: &wgpu::Sampler,
) -> (wgpu::BindGroup, wgpu::BindGroupLayout) {
    let view_list = view_list
        .iter()
        .collect::<Vec<_>>();

    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("atlas group layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { 
                        filterable: true 
                    },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: Some(std::num::NonZeroU32::new(view_list.len() as u32).unwrap()),
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    });
    
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("texture array bind group"),
        layout: &layout, 
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureViewArray(&view_list),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            }
        ],
    });

    (bind_group, layout)
}

fn create_texture(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    size: u32,
) -> wgpu::TextureView {
    let t = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("atlas_texture"),
        size: wgpu::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        view_formats: &[
            format.add_srgb_suffix(), 
            format.remove_srgb_suffix() 
        ],
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::RENDER_ATTACHMENT,
    });
    t.create_view(&TEX_VIEW_DESC)
}


#[derive(Default)]
pub struct GlyphAtlas {
    count: usize,
    size: [u32; 2],

    // cached data
    glyphs_per_row: u32,
    largest_glyph_size: [u32; 2],
}
impl GlyphAtlas {
    pub fn new(size: u32) -> Self {
        Self {
            size: [size, size],
            ..Default::default()
        }
    }
    pub fn clear(&mut self) {
        self.count = 0;
        self.largest_glyph_size = [0, 0];
    }
    
    /// just get the next square in the list
    pub fn reserve(
        &mut self, 
        width: u32, 
        height: u32,
        glyph_layer: u32,
    ) -> AtlasResult {
        use tataku::ATLAS_PADDING;
        if width == 0 || height == 0 {
            return AtlasResult::Ok(tataku::TextureReference::empty())
        }
        let width2 = width + ATLAS_PADDING * 2;
        let height2 = height + ATLAS_PADDING * 2;

        if width2 > self.largest_glyph_size[0] {
            self.largest_glyph_size[0] = width2;
            self.glyphs_per_row = self.size[0] / width2;
        }
        if height2 > self.largest_glyph_size[1] {
            self.largest_glyph_size[1] = height2;
        }

        let c = self.count as u32;
        self.count += 1;

        // size of one glyph
        let [w, h] = self.largest_glyph_size;

        let x = w * (c % self.glyphs_per_row) + ATLAS_PADDING;
        let y = h * (c / self.glyphs_per_row) + ATLAS_PADDING;

        if x + w > self.size[0] 
        || y + h > self.size[1] {
            return AtlasResult::NoSpace
        }

        AtlasResult::Ok(tataku::TextureReference::new(
            [x, y],
            [width, height],
            glyph_layer,
            0,
            tataku::AtlasTag::Glyph,
            self.size,
        ))
    }
}


pub enum AtlasResult {
    Ok(tataku::TextureReference),
    Resized(tataku::TextureReference),
    NoSpace
}

use crate::prelude::*;
use crate::renderable_surface::*;

use vello::peniko::color::{ AlphaColor, Srgb };
const BASE_COLOR: AlphaColor<Srgb> = AlphaColor::from_rgba8(0, 0, 0, 0);

pub(crate) struct Pipeline {
    renderer: vello::Renderer,
    texture: wgpu::Texture,

    blitterer: wgpu::util::TextureBlitter,
}
impl Pipeline {
    pub fn create(
        device: &wgpu::Device,
        output: &WgpuTextureReference,
    ) -> Option<Self> {
        let renderer = vello::Renderer::new(
            device,
            vello::RendererOptions::default()
        )
            .inspect_err(|e| warn!("error initializing vello: {e:?}"))
            .ok()?;

        let blitterer = wgpu::util::TextureBlitterBuilder::new(
            device,
            output.view.texture().format()
        )
        .blend_state(wgpu::BlendState::ALPHA_BLENDING)
        .build();

        Some(Self {
            renderer,
            blitterer,
            texture: device.create_texture(&Self::tex_desc(output.size)),
        })
    }

    fn tex_desc(size: wgpu::Extent3d) -> wgpu::TextureDescriptor<'static> {
        wgpu::TextureDescriptor {
            label: Some("vello intermediate"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING 
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        }
    }

    pub fn perform(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        output: &WgpuTextureReference,
        data: &super::Buffer,
    ) {
        if self.texture.size() != output.size {
            self.texture = device.create_texture(
                &Self::tex_desc(output.size)
            );
        }

        let tex_view = self.texture.create_view(
            &wgpu::TextureViewDescriptor::default()
        );

        if let Err(e) = self.renderer.render_to_texture(
            device,
            queue,
            &data.scene,
            &tex_view,
            &vello::RenderParams {
                base_color: BASE_COLOR,
                width: output.size.width,
                height: output.size.height,
                antialiasing_method: vello::AaConfig::Area,
            }
        ) {
            error!("error rendering vello: {e:?}");
            return 
        }

        let mut encoder = device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("vello blitter encoder")
            }
        );
        self.blitterer.copy(
            device, 
            &mut encoder, 
            &tex_view, 
            &output.view
        );

        // println!("{:#?}", data.list);


        queue.submit([ encoder.finish() ]);
    }
}

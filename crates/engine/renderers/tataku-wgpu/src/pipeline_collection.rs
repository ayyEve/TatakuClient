use crate::{prelude::*, renderable_surface::WgpuTextureReference};
use tataku::GraphicsPipeline;


pub(crate) struct PipelineCollection {
    slider: wgpu::RenderPipeline,
    flashlight: wgpu::RenderPipeline,
    standard: Vec<wgpu::RenderPipeline>,

    pub box_blur: shaders::box_blur::Pipeline,
    pub gaussian_blur: shaders::gaussian_blur::Pipeline,

    #[cfg(feature="vello_rendering")]
    vello: Option<shaders::vello::Pipeline>,
}
impl PipelineCollection {
    pub fn new(
        device: &wgpu::Device,
        projection_matrix: &crate::ProjectionMatrix,
        atlas: &crate::atlas::WgpuAtlas,
        intermediate_tex_ref: &WgpuTextureReference,
    ) -> Self {
        #[cfg(feature="vello_rendering")]
        let vello = shaders::vello::Pipeline::create(&device, intermediate_tex_ref);

        let box_blur = shaders::box_blur::Pipeline::new(device, intermediate_tex_ref);
        let gaussian_blur = shaders::gaussian_blur::Pipeline::new(device, intermediate_tex_ref);

        let mut pipelines = shaders::standard::create_standard_pipelines(
            device,
            projection_matrix,
            atlas,
        );

        let slider = shaders::slider::create_slider_pipeline(
            device,
            projection_matrix
        );
        let flashlight = shaders::flashlight::create_flashlight_pipeline(
            device,
            projection_matrix
        );

        pipelines.sort_by_key(|(k, _)| (*k) as u8);
        let standard = pipelines.into_iter().map(|(_,p)| p).collect::<Vec<_>>();

        Self {
            slider,
            standard,
            flashlight,

            box_blur,
            gaussian_blur,

            #[cfg(feature="vello_rendering")] 
            vello_,
        }
    }

    pub fn init_buffer_queues(
        &self,
        device: &wgpu::Device,
    ) -> Vec<(PipelineType, Box<RenderBufferQueueType>)> {
        vec![
            (PipelineType::Standard, Box::new(RenderBufferQueueType::Standard(
                RenderBufferQueue::default().init(
                    device,
                    &self.standard[0]
                )
            ))),
            (PipelineType::Slider, Box::new(RenderBufferQueueType::Slider(
                RenderBufferQueue::default().init(
                    device,
                    &self.slider
                )
            ))),
            (PipelineType::Flashlight, Box::new(RenderBufferQueueType::Flashlight(
                RenderBufferQueue::default().init(
                    device,
                    &self.flashlight
                )
            ))),

            (PipelineType::BoxBlur, Box::new(RenderBufferQueueType::BoxBlur(
                RenderBufferQueue::default().init(
                    device,
                    &self.box_blur.pipeline
                )
            ))),
            (PipelineType::GaussianBlur, Box::new(RenderBufferQueueType::GaussianBlur(
                RenderBufferQueue::default().init(
                    device,
                    &self.gaussian_blur.pipeline
                )
            ))),

            #[cfg(feature="vello")]
            (PipelineType::Vello, Box::new(RenderBufferQueueType::Vello(
                RenderBufferQueue::default().init(
                    device,
                    WgpuPipeline::None
                )
            ))),
        ]
    }


    pub fn get(&self, p: GraphicsPipeline) -> &wgpu::RenderPipeline {
        match p {
            GraphicsPipeline::Slider => &self.slider,
            GraphicsPipeline::Flashlight => &self.flashlight,
            GraphicsPipeline::Standard(b) => &self.standard[b as u8 as usize],

            _ => unimplemented!()
        }
    }

    pub fn get_pipeline_reference(
        &self,
        p: GraphicsPipeline,
    ) -> WgpuPipeline<'_> {
        match p {
            GraphicsPipeline::BoxBlur => WgpuPipeline::Compute(&self.box_blur.pipeline),
            GraphicsPipeline::GaussianBlur => WgpuPipeline::Compute(&self.gaussian_blur.pipeline),
            GraphicsPipeline::Slider => WgpuPipeline::Render(&self.slider),
            GraphicsPipeline::Flashlight => WgpuPipeline::Render(&self.flashlight),
            GraphicsPipeline::Standard(b) => WgpuPipeline::Render(&self.standard[b as u8 as usize]),
            // _ => GraphicsPipeline::Standard(&self.pipelines[&last_buffer.graphics_pipeline()]),

            // #[cfg(feature="vello")] PipelineType::Vello => WgpuPipeline::None,
            _ => unimplemented!()
        }
    }

    pub fn perform_compute_pipeline(
        &mut self,
        p: PipelineType,
        i: &RenderBufferType,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        output: &WgpuTextureReference
    ) {
        match p {
            PipelineType::GaussianBlur => {
                let RenderBufferType::GaussianBlur(buffer) = i
                else { unreachable!() };

                self.gaussian_blur.perform(
                    device,
                    queue,
                    output,
                    buffer
                );
            }
            PipelineType::BoxBlur => {
                let RenderBufferType::BoxBlur(buffer) = i
                else { unreachable!() };

                self.box_blur.perform(
                    device,
                    queue,
                    output,
                    buffer
                );
            }

            #[cfg(feature="vello")]
            PipelineType::Vello => {
                let vello = self
                    .vello
                    .as_mut()
                    .unwrap();

                let RenderBufferType::Vello(buffer) = i
                else { unreachable!() };

                vello.perform(
                    device,
                    queue,
                    renderable.texture,
                    buffer
                );
            }

            _ => unreachable!()
        }
    }

}

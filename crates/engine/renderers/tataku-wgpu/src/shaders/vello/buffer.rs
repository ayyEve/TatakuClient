use crate::prelude::*;
use crate::wgpu_engine::WgpuPipeline;
use crate::buffer_queue::RenderBufferable;

pub(crate) struct Buffer {
    pub blend_mode: tataku::GraphicsPipeline,
    pub scissor: Option<tataku::Scissor>,
    pub scene: vello::Scene,

    pub used: u64,
}
impl RenderBufferable for Buffer {
    type Cache = CpuBuffer;
    const VTX_PER_BUF: u64 = 1;
    const IDX_PER_BUF: u64 = 1;

    // fn name() -> &'static str { "vello buffer" }
    fn should_write(&self) -> bool { self.used > 0 }

    fn reset(&mut self) {
        self.scissor = None;
        self.scene.reset();
        self.used = 0;
    }

    fn dump(&mut self, _queue: &wgpu::Queue, cache: &mut Self::Cache) {
        self.scene.reset();
        std::mem::swap(&mut self.scene, &mut cache.scene);
    }

    fn create_new_buffer(_device: &wgpu::Device, _: WgpuPipeline) -> Self {
        Self {
            scissor: None,
            blend_mode: tataku::GraphicsPipeline::None,
            scene: vello::Scene::new(),
            used: 0,
        }
    }
}

pub(crate) struct CpuBuffer {
    pub scene: vello::Scene,
}
impl Default for CpuBuffer {
    fn default() -> Self {
        Self {
            scene: vello::Scene::new(),
        }
    }
}

pub(crate) struct ReserveData<'a> {
    pub scene: &'a mut vello::Scene,
}

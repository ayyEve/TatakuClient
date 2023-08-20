#![allow(unused)]
use crate::prelude::*;

pub type Scissor = Option<[f32; 4]>;

pub struct GraphicsState {}
impl GraphicsState {
    pub fn create_render_target(&mut self, w:u32, h:u32, clear_color: Color, do_render: impl FnOnce(&mut GraphicsState, Matrix)) -> Option<RenderTarget> {
        None
    }
    pub fn update_render_target(&mut self, target: RenderTarget, do_render: impl FnOnce(&mut GraphicsState, Matrix)) {
    }
}

// texture stuff
impl GraphicsState {
    pub fn load_texture_bytes(&mut self, data: impl AsRef<[u8]>) -> TatakuResult<TextureReference> {
        Err(TatakuError::String(String::new()))
    }

    pub fn load_texture_rgba(&mut self, data: &Vec<u8>, width: u32, height: u32) -> TatakuResult<TextureReference> {
        Err(TatakuError::String(String::new()))
    }

    pub fn free_tex(&mut self, mut tex: TextureReference) {
    }
}



// draw helpers
impl GraphicsState {

    /// draw an arc with the center at 0,0
    pub fn draw_arc(&mut self, start: f32, end: f32, radius: f32, color: Color, resolution: u32, transform: Matrix, scissor: Scissor, blend_mode: BlendMode) {
    }

    pub fn draw_circle(&mut self, radius: f32, color: Color, border: Option<Border>, resolution: u32, transform: Matrix, scissor: Scissor, blend_mode: BlendMode) {
    
    }

    pub fn draw_line(&mut self, line: [f32; 4], thickness: f32, color: Color, transform: Matrix, scissor: Scissor, blend_mode: BlendMode) {
    }

    /// rect is [x,y,w,h]
    pub fn draw_rect(&mut self, rect: [f32; 4], border: Option<Border>, shape: Shape, color: Color, transform: Matrix, scissor: Scissor, blend_mode: BlendMode) {
        
    }

    pub fn draw_tex(&mut self, tex: &TextureReference, color: Color, h_flip: bool, v_flip: bool, transform: Matrix, scissor: Scissor, blend_mode: BlendMode) {
    }

}

// particle stuff 
impl GraphicsState {
    pub fn add_emitter(&mut self, emitter: EmitterRef) {
        // self.particle_system.add(emitter);
    }

    pub fn update_emitters(&mut self) {
        // self.particle_system.update(&self.device, &self.queue);
    }
}




#[derive(Copy, Clone, Serialize, Deserialize, Debug, Dropdown, Eq, PartialEq)]
pub enum PerformanceMode {
    PowerSaver,
    HighPerformance,
}


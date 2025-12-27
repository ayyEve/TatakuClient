use crate::*;
use tataku::Vector2;
use image::RgbaImage;

#[async_trait]
pub trait GraphicsInitializer<'window> {
    fn name(&self) -> &'static str;
    async fn init(
        &self,
        window: &'window dyn RawWindow,
        settings: settings::display::DisplaySettings
    ) -> tataku::Result<Box<dyn graphics::RenderingEngine + 'window>>;
}

pub trait RawWindow: raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle + Sync {}
impl<T> RawWindow for T where T: raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle + Sync {}

pub struct WindowInitializers<'a> {
    pub integrations: Vec<io::TatakuIntegrationBuilder>,
    pub graphics_init: Vec<Box<dyn GraphicsInitializer<'a>>>,
    pub window_creation_barrier: Arc<std::sync::Barrier>,
}


#[cfg(feature="graphics")]
pub struct WindowData {
    pub event_receiver: tokio::sync::mpsc::Receiver<engine::window::Event>,
    pub mouse_position_receiver: triple_buffer::Output<Vector2>,
    pub action_sender: Box<dyn WindowActionSender>,
}

#[cfg(feature="graphics")]
impl WindowData {
    pub fn send_event(&mut self, action: actions::window::WindowAction) {
        self.action_sender.send(action);
    }
}

#[cfg(feature="graphics")]
#[derive(Clone, Default)]
pub struct WindowCounters {
    pub render_count: Arc<std::sync::atomic::AtomicU32>,
    pub render_frametime: Arc<std::sync::atomic::AtomicU32>,

    pub input_count: Arc<std::sync::atomic::AtomicU32>,
    pub input_frametime: Arc<std::sync::atomic::AtomicU32>,
}


pub struct WindowCreator {
    pub create: for<'w, 's> fn(WindowCreateValues<'w, 's>) -> (Box<dyn Window<'w> + 'w>, Box<dyn std::any::Any>),
}
pub struct WindowCreateValues<'w, 's> {
    pub event_sender: tokio::sync::mpsc::Sender<window::Event>,
    pub mouse_position_sender: engine::triple_buffer::Input<tataku::Vector2>,
    pub settings: &'s settings::Settings,
    
    pub counters: WindowCounters,
    pub init: WindowInitializers<'w>,
}



pub trait Window<'window> {
    fn get_texture_manager(&self) -> Box<dyn TextureManager>;
    fn get_action_sender(&self) -> Box<dyn WindowActionSender>;

    fn run(self: Box<Self>, data: &'window mut dyn std::any::Any);
}

pub trait TextureManager: Send + Sync {
    fn load_texture_data(&mut self, data: RgbaImage) -> tataku::Result<tataku::TextureReference>;
    fn free_texture(&mut self, tex: tataku::TextureReference);

    fn dump_atlas(&mut self);
}

pub trait WindowActionSender: Send + Sync {
    fn send(&mut self, action: actions::window::WindowAction);
}

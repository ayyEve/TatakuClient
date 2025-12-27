use crate::*;
use tataku::Vector2;
use image::RgbaImage;

#[async_trait]
pub trait GraphicsInitializer<'window> {
    fn name(&self) -> &'static str;
    async fn init(
        &self,
        window: &'window dyn Windowable,
        settings: settings::display::DisplaySettings
    ) -> tataku::Result<Box<dyn graphics::RenderingEngine + 'window>>;
}

pub trait Windowable: raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle + Sync {}
impl<T> Windowable for T where T: raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle + Sync {}

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


pub trait TatakuWindow<'window> {
    // fn new(
    //     event_sender: Sender<window::Event>,
    //     mouse_position_sender: engine::triple_buffer::Input<tataku::Vector2>,
    //     settings: &settings::Settings,
        
    //     #[cfg(feature="graphics")] window_counters: WindowCounters,
    //     init: WindowInitializers<'window>,
    // ) -> Self;
    fn get_texture_manager(&self) -> Box<dyn TextureManager>;
    fn get_action_sender(&self) -> Box<dyn WindowActionSender>;
}

pub trait TextureManager: Send + Sync {
    fn load_texture_data(&mut self, data: RgbaImage) -> tataku::Result<tataku::TextureReference>;
    fn free_texture(&mut self, tex: tataku::TextureReference);

    fn dump_atlas(&mut self);
}

pub trait WindowActionSender: Send + Sync {
    fn send(&mut self, action: actions::window::WindowAction);
}

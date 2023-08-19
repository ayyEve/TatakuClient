/**
 * Cursor Manager
 * 
 * this uses an mpsc channel because it may be inaccessible to things that need access
 * ie, a gamemode might want to hide the cursor, however it does not have direct access to the cursor field in game
 */
use crate::prelude::*;
use tokio::sync::mpsc::{Sender, Receiver, channel};

static CURSOR_EVENT_QUEUE:OnceCell<Sender<CursorEvent>> = OnceCell::const_new();

pub struct CursorManager {
    /// position of the visible cursor
    pub pos: Vector2,

    cursor_images: HashMap<CursorMode, Image>,
    cursor_mode: CursorMode,

    // cached settings
    ripple_radius_override: Option<f32>,
    // ripple_image: Option<Image>,

    cursor_rotation: f32,

    // event_receiver: Arc<Receiver<CursorEvent>>,
    event_receiver: Receiver<CursorEvent>,


    /// should the cursor be visible?
    visible: bool,

    left_pressed: bool,
    right_pressed: bool,

    settings: SettingsHelper,
    current_skin: CurrentSkinHelper,

    ripples: Vec<TransformGroup>,
    time: Instant,
}

impl CursorManager {
    pub async fn new() -> Self {
        let (sender, event_receiver) = channel(1000);
        CURSOR_EVENT_QUEUE.set(sender).expect("Cursor event queue already exists");

        let settings = SettingsHelper::new();
        Self {
            pos: Vector2::ZERO,

            cursor_images: HashMap::new(),
            cursor_mode: CursorMode::Normal,
            
            current_skin: CurrentSkinHelper::new(),
            cursor_rotation: 0.0,

            event_receiver,

            left_pressed: false,
            right_pressed: false,
            visible: true,
            ripple_radius_override: None,
            settings,

            ripples: Vec::new(),
            time: Instant::now(),
        }
    }


    pub async fn reload_skin(&mut self) {
        self.cursor_images.clear();

        for mode in [
            CursorMode::Normal,
            CursorMode::HorizontalResize,
            CursorMode::VerticalResize,
            CursorMode::Resize,
            CursorMode::Pointer,
            CursorMode::Text
        ] {
            if let Some(image) = SkinManager::get_texture(mode.tex_name(), true).await {
                self.cursor_images.insert(mode, image);
            }
        }

        self.cursor_rotation = 0.0;
    }

    fn get_cursor_image(&self) -> Option<&Image> {
        self.cursor_images.get(&self.cursor_mode)
    }


    pub async fn update(&mut self, _time: f32, cursor_pos: Vector2) {
        self.pos = cursor_pos;

        // check settings update 
        self.settings.update();


        // work through the event queue
        while let Ok(event) = self.event_receiver.try_recv() {
            match event {
                CursorEvent::OverrideRippleRadius(radius_maybe) => self.ripple_radius_override = radius_maybe,
                CursorEvent::SetVisible(show) => self.visible = show,
            }
        }


        if self.current_skin.update() {
            self.reload_skin().await;
        }

        // if self.current_skin.cursor_rotate {
        //     self.cursor_rotation = (time / 2000.0) % (PI * 2.0);
        // }

        // update ripples
        let time = self.time.as_millis();
        self.ripples.retain_mut(|ripple| {
            ripple.update(time);
            ripple.visible()
        });

    }

    pub fn left_pressed(&mut self, pressed: bool) {
        self.left_pressed = pressed;
        if pressed && self.settings.cursor_ripples { self.add_ripple() }
    }
    pub fn right_pressed(&mut self, pressed: bool) {
        self.right_pressed = pressed;
        if pressed && self.settings.cursor_ripples { self.add_ripple() }
    }

    pub fn draw_ripples(&self, list: &mut RenderableCollection) {
        if !self.visible { return }
        
        // draw ripples
        for ripple in self.ripples.iter() {
            list.push(ripple.clone())
            // ripple.draw(list)
        }
    }

    pub fn draw(&mut self, list: &mut RenderableCollection) {
        if !self.visible { return }

        // draw cursor itself
        if let Some(mut cursor) = self.get_cursor_image().cloned() {
            cursor.pos = self.pos;
            cursor.rotation = self.cursor_rotation;
            // cursor.current_color = self.color;
            
            list.push(cursor.clone());
        } else {
            // use font awesome as fallback
            let (c, align) = match self.cursor_mode {
                CursorMode::Normal => (FontAwesome::ArrowPointer, Align::TopLeft),
                CursorMode::HorizontalResize => (FontAwesome::LeftRight, Align::CenterMiddle),
                CursorMode::VerticalResize => (FontAwesome::UpDown, Align::CenterMiddle),
                CursorMode::Resize => (FontAwesome::UpDownLeftRight, Align::CenterMiddle),
                CursorMode::Pointer => (FontAwesome::HandPointer, Align::TopLeft),
                CursorMode::Text => (FontAwesome::ICursor, Align::CenterMiddle),
            };

            let mut text = Text::new(self.pos, 32.0, c, self.settings.cursor_color, Font::FontAwesome);
            text.rotation = self.cursor_rotation;

            if align == Align::CenterMiddle {
                let size = text.measure_text();
                text.pos -= size / 2.0;
            }
            list.push(text);

            // list.push(Circle::new(
            //     self.pos,
            //     self.settings.cursor_scale,
            //     self.color,
            //     if self.settings.cursor_border > 0.0 {
            //         Some(Border::new(
            //             self.border_color,
            //             self.settings.cursor_border
            //         ))
            //     } else { None }
            // ));
        }
    }

    fn add_ripple(&mut self) {
        let mut group = TransformGroup::new(self.pos).alpha(0.0).border_alpha(1.0);
        let duration = 500.0;
        let time = self.time.as_millis();

        // if let Some(mut ripple) = self.ripple_image.clone() {

        //     ripple.color.a = self.ripple_color.a;
        //     ripple.pos = self.pos;

        //     // set scale
        //     const SCALE:f64 = 0.25;
        //     ripple.scale = Vector2::ONE * SCALE;

        //     let end_scale = self
        //         .ripple_radius_override
        //         .map(|r|r / ripple.size().x / 2.0)
        //         .unwrap_or(self.settings.cursor_ripple_final_scale)
        //         * SCALE;

        //     // add to transform group and make it ripple
        //     group.push(ripple);
        //     group.ripple_scale_range(0.0, duration, time, end_scale..SCALE, Some(2.0..0.0), Some(0.2));
        // } else {

            // primitive ripple, not always correct
            let radius = 1.0;
            let end_radius = self.ripple_radius_override.unwrap_or(radius * self.settings.cursor_ripple_final_scale);

            let end_scale = end_radius / radius;

            // let end_scale = self.settings.cursor_ripple_final_scale * self.ripple_radius_override.map(|r| DEFAULT_CURSOR_SIZE / r).unwrap_or(1.0);

            group.push(Circle::new(
                Vector2::ZERO,
                radius,
                Color::WHITE.alpha(0.5),
                Some(Border::new(Color::WHITE, 2.0 / end_scale))
            ));
            group.ripple(0.0, duration, time, end_scale, true, Some(0.2));
        // }


        self.ripples.push(group);
    }

}

impl CursorManager {
    fn add_event(event: CursorEvent) {
        // should always be okay
        if let Some(q) = CURSOR_EVENT_QUEUE.get() {
            if let Err(e) = q.try_send(event) {
                error!("cursor channel error: {e}")
            }
        }
    }


    pub fn set_visible(visible: bool) {
        Self::add_event(CursorEvent::SetVisible(visible));
    }
    pub fn set_ripple_override(radius: Option<f32>) {
        Self::add_event(CursorEvent::OverrideRippleRadius(radius));
    }
}


#[derive(Copy, Clone)]
enum CursorEvent {
    SetVisible(bool),
    OverrideRippleRadius(Option<f32>),
}

#[allow(unused)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum CursorMode {
    /// regular cursor image
    Normal,
    HorizontalResize,
    VerticalResize,
    /// both horizontal and vertical resize
    Resize,
    /// hand pointing at thing
    Pointer,
    /// text cursor
    Text,
}
impl CursorMode {
    fn tex_name(&self) -> &str {
        match self {
            CursorMode::Normal => "tataku_cursor_normal",
            CursorMode::HorizontalResize => "tataku_cursor_hresize",
            CursorMode::VerticalResize => "tataku_cursor_vresize",
            CursorMode::Resize => "tataku_cursor_resize",
            CursorMode::Pointer => "tataku_cursor_pointer",
            CursorMode::Text => "tataku_cursor_text",
        }
    }
}

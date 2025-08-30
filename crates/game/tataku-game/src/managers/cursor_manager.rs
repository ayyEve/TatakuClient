/**
 * Cursor Manager
 */
use crate::prelude::*;

pub struct CursorManager {
    /// position of the visible cursor
    pub pos: Vector2,

    cursor_images: HashMap<CursorMode, Image>,
    cursor_mode: CursorMode,

    // cached settings
    ripple_radius_override: Option<f32>,
    // ripple_image: Option<Image>,

    cursor_rotation: f32,

    /// should the cursor be visible?
    visible: bool,

    left_pressed: bool,
    right_pressed: bool,

    current_skin: Arc<SkinSettings>,

    ripples: Vec<Trail>,
    time: f32,

    settings: CursorSettings,
}
impl CursorManager {
    pub fn new(skin: Arc<SkinSettings>, settings: CursorSettings) -> Self {
        Self {
            pos: Vector2::ZERO,

            cursor_images: HashMap::new(),
            cursor_mode: CursorMode::Normal,

            current_skin: skin,
            cursor_rotation: 0.0,

            left_pressed: false,
            right_pressed: false,
            visible: true,
            ripple_radius_override: None,
            settings,

            ripples: Vec::new(),
            time: 0.0
        }
    }


    #[cfg(feature="graphics")]
    pub fn reload_skin(&mut self, skin_manager: &mut dyn SkinProvider) {
        self.cursor_images.clear();
        self.current_skin = skin_manager.skin().clone();

        for mode in [
            CursorMode::Normal,
            CursorMode::HorizontalResize,
            CursorMode::VerticalResize,
            CursorMode::Resize,
            CursorMode::Pointer,
            CursorMode::Text
        ] {
            if let Some(image) = skin_manager.get_texture(
                mode.tex_name(),
                &TextureSource::Skin,
                SkinUsage::Game, true
            ) {
                self.cursor_images.insert(mode, image);
            }
        }

        self.cursor_rotation = 0.0;
    }

    fn get_cursor_image(&self) -> Option<&Image> {
        self.cursor_images.get(&self.cursor_mode)
    }


    pub fn update(&mut self, time: f32, cursor_pos: Vector2) {
        self.time = time;
        self.pos = cursor_pos;

        // update ripples
        self.ripples.retain(|ripple| !ripple.complete(time));
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
            list.list.push(ripple.ripple(
                self.time,
                0.0,
                self.settings.cursor_ripple_final_radius,
                self.settings.cursor_ripple_color.alpha(0.2),
                Some(Border::new(self.settings.cursor_ripple_color.alpha(0.5), 2.0))
            ));
        }
    }

    pub fn draw(&mut self, list: &mut RenderableCollection) {
        if !self.visible { return }

        // draw cursor itself
        if let Some(mut cursor) = self.get_cursor_image().cloned() {
            cursor.pos = self.pos;
            cursor.rotation = self.cursor_rotation;
            list.push(cursor.clone());
        } else {
            // use font awesome as fallback
            let (c, align) = match self.cursor_mode {
                CursorMode::Normal => (FontAwesome::ArrowPointer, Alignment::TOP_LEFT),
                CursorMode::HorizontalResize => (FontAwesome::LeftRight, Alignment::CENTER),
                CursorMode::VerticalResize => (FontAwesome::UpDown, Alignment::CENTER),
                CursorMode::Resize => (FontAwesome::UpDownLeftRight, Alignment::CENTER),
                CursorMode::Pointer => (FontAwesome::HandPointer, Alignment::TOP_LEFT),
                CursorMode::Text => (FontAwesome::ICursor, Alignment::CENTER),
            };

            // let mut text = Text::new(
            //     self.pos,
            //     32.0,
            //     c,
            //     self.settings.cursor_color.color,
            //     DefaultFont::FontAwesome
            // );
            // text.rotation = self.cursor_rotation;

            // if align == Alignment::CENTER {
            //     let size = text.measure_text();
            //     text.pos -= size / 2.0;
            // }
            // list.push(text);
        }
    }

    fn add_ripple(&mut self) {
        let duration = 500.0;

        self.ripples.push(Trail::new(self.pos, self.time, duration));
    }


    pub fn handle_cursor_action(&mut self, action: CursorAction) {
        match action {
            CursorAction::OverrideRippleRadius(radius_maybe)
                => self.ripple_radius_override = radius_maybe,
            CursorAction::SetVisible(show) => {
                // trace!("setting cursor visible = {show}");
                self.visible = show;
            },
        }
    }
}

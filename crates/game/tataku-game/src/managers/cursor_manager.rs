/**
 * Cursor Manager
 */
use crate::prelude::*;
use ui::widget::TextLayoutContexts;

use tataku::{
    Color,
    Bounds,
    Border,
    Vector2,
    Alignment,
    DefaultFont,
    FontAwesome,
    HorizontalAlign,
};

use engine::{
    actions,
    actions::cursor::CursorMode,
};


#[derive(Default2)]
pub(crate) struct CursorManager {
    /// position of the visible cursor
    pos: Vector2,
    current_skin: Arc<graphics::SkinSettings>,
    settings: engine::settings::cursor::CursorSettings,

    // state data

    cursor_mode: CursorMode,
    cursor_images: HashMap<CursorMode, graphics::Image>,

    /// should the cursor be visible?
    #[default(true)] 
    visible: bool,
    cursor_rotation: f32,

    left_pressed: bool,
    right_pressed: bool,


    time: f32,
    ripples: Vec<graphics::Trail>,
    // ripple_image: Option<Image>,
    ripple_radius_override: Option<f32>,


    // fallback cursor config
    #[default(Arc::new(parley::Layout::new()))]
    layout: Arc<parley::Layout<Color>>,
    pos_offset: Vector2,
}
impl CursorManager {
    pub fn reload_skin(&mut self, skin_manager: &mut dyn graphics::SkinProvider) {
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
                Path::new(mode.tex_name()),
                &graphics::TextureSource::Skin,
                graphics::SkinUsage::Game, true
            ) {
                self.cursor_images.insert(mode, image);
            }
        }

        self.cursor_rotation = 0.0;
    }

    fn get_cursor_image(&self) -> Option<&graphics::Image> {
        self.cursor_images.get(&self.cursor_mode)
    }

    fn fallback_cursor(cursor_mode: CursorMode) -> FallbackCursorInfo {
        match cursor_mode {
            CursorMode::Normal => FallbackCursorInfo::new_offset(
                FontAwesome::ArrowPointer, 
                Alignment::TOP_LEFT, 
                Some(Vector2::new(-2.0, 3.0)),
            ),
            CursorMode::HorizontalResize => FallbackCursorInfo::new(
                FontAwesome::LeftRight, 
                Alignment::CENTER
            ),
            CursorMode::VerticalResize => FallbackCursorInfo::new(
                FontAwesome::UpDown, 
                Alignment::CENTER
            ),
            CursorMode::Resize => FallbackCursorInfo::new(
                FontAwesome::UpDownLeftRight, 
                Alignment::CENTER
            ),
            CursorMode::Pointer => FallbackCursorInfo::new(
                FontAwesome::HandPointer, 
                Alignment::TOP_LEFT
            ),
            CursorMode::Text => FallbackCursorInfo::new(
                FontAwesome::ICursor, 
                Alignment::CENTER
            ),
        }
    }

    pub fn handle_cursor_action(
        &mut self, 
        action: actions::cursor::CursorAction,
        text_layout_contexts: &mut TextLayoutContexts,
    ) {
        use actions::cursor::CursorAction as CursorAction;
        match action {
            CursorAction::OverrideRippleRadius(radius_maybe)
                => self.ripple_radius_override = radius_maybe,
            
            CursorAction::SetVisible(show) => {
                // trace!("setting cursor visible = {show}");
                self.visible = show;
            }

            CursorAction::SetCursorMode(new_mode) => {
                // if self.cursor_mode == new_mode { return }
                self.cursor_mode = new_mode;

                if self.get_cursor_image().is_some() { return };

                let fallback_info = Self::fallback_cursor(self.cursor_mode);
                let mut layout = text_layout_contexts.simple_text(
                    &fallback_info.char.to_string(), 
                    &ui::style::TextStyle {
                        font: DefaultFont::FontAwesome,
                        font_size: 32.0,
                        color: self.settings.cursor_color.color,
                        line_height: 32.0,
                        alignment: HorizontalAlign::Left,
                    }, 
                );
                layout.break_all_lines(None);
                self.layout = Arc::new(layout);

                let cursor_size = Vector2::new(
                    self.layout.width(), 
                    self.layout.height()
                );

                let pos = fallback_info.align.resolve(
                    &Bounds::default(), 
                    cursor_size,
                    true, 
                    true,
                );

                self.pos_offset = pos + fallback_info.extra_offset.unwrap_or_default();
            }
        }
    }

    pub fn update(&mut self, time: f32, cursor_pos: Vector2) {
        self.time = time;
        self.pos = cursor_pos;

        // update ripples
        self.ripples.retain(|ripple| !ripple.complete(time));
    }

    pub fn update_settings(
        &mut self, 
        settings: engine::settings::cursor::CursorSettings,
        actions: &mut actions::ActionQueue
    ) {
        self.settings = settings;

        // refresh color
        actions.push(actions::cursor::CursorAction::SetCursorMode(
            self.cursor_mode
        ).into());
    }

    pub fn left_pressed(&mut self, pressed: bool) {
        self.left_pressed = pressed;
        if pressed && self.settings.cursor_ripples { self.add_ripple() }
    }
    pub fn right_pressed(&mut self, pressed: bool) {
        self.right_pressed = pressed;
        if pressed && self.settings.cursor_ripples { self.add_ripple() }
    }

    pub fn draw_ripples(&self, list: &mut graphics::RenderableCollection) {
        if !self.visible { return }

        // draw ripples
        for ripple in self.ripples.iter() {
            list.push(ripple.ripple(
                self.time,
                0.0,
                self.settings.cursor_ripple_final_radius,
                self.settings.cursor_ripple_color.alpha(0.2),
                Some(Border::new(self.settings.cursor_ripple_color.alpha(0.5), 2.0))
            ));
        }
    }

    pub fn draw(&mut self, list: &mut graphics::RenderableCollection) {
        if !self.visible { return }

        let transform = graphics::Transform {
            pos: self.pos,
            rotation: self.cursor_rotation,
            ..graphics::Transform::identity()
        };

        // draw cursor itself
        if let Some(cursor) = self.get_cursor_image().cloned() {
            list.push(cursor.with_transform(transform.matrix()));
        } else {
            // use font awesome as fallback
            list.push(graphics::Text::new(self.layout.clone())
                .with_transform(transform.matrix())
            );

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
        self.ripples.push(graphics::Trail::new(self.pos, self.time, duration));
    }

}


struct FallbackCursorInfo {
    /// What char to use
    char: FontAwesome,

    /// How should the cursor be aligned
    align: Alignment,

    /// Extra offset to tweak positioning
    /// 
    /// This is added to the cursor position
    extra_offset: Option<Vector2>,
}
impl FallbackCursorInfo {
    fn new(char: FontAwesome, align: Alignment) -> Self {
        Self::new_offset(char, align, None)
    }
    fn new_offset(
        char: FontAwesome, 
        align: Alignment, 
        extra_offset: Option<Vector2>
    ) -> Self {
        Self {
            char,
            align,
            extra_offset
        }
    }
}

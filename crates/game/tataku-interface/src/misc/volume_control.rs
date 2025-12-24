use crate::prelude::*;
use input::Key;
use tataku::{
    Vector2,
    Color,
    Border,
};
use ui::{
    style::*,
    widget::*,
};

/// how long should the volume thing be displayed when changed (ms)
const VOLUME_CHANGE_DISPLAY_TIME:f32 = 2_000.0;

#[derive(Default)]
/// helper to move volume things out of game, cleaning up code
pub struct VolumeControl {
    /// 0-2, 0 = master, 1 = effect, 2 = music
    vol_selected_index: u8,
    /// when the volume was changed, or the selected index changed
    vol_selected_time: f32,
    timer: tataku::Instant,

    settings: VolumeSettings,
    window_size: Vector2,
}
impl VolumeControl {
    fn _visible(&self) -> bool {
        let elapsed = self.timer.as_millis();
        elapsed - self.vol_selected_time < VOLUME_CHANGE_DISPLAY_TIME
    }

    pub fn window_size_changed(&mut self, window_size: Vector2) {
        self.window_size = window_size;
    }

    fn change(
        &mut self, 
        delta: f32, 
        settings: &mut engine::Settings
    ) -> Option<actions::song::SongAction> {
        let elapsed = self.timer.as_millis();

        // reset index back to 0 (master) if the volume hasnt been touched in a while
        if elapsed - self.vol_selected_time > VOLUME_CHANGE_DISPLAY_TIME + 1000.0 {
            self.vol_selected_index = 0;
        }

        // find out what volume to edit, and edit it
        match self.vol_selected_index {
            0 => settings.master_vol = (settings.master_vol + delta).clamp(0.0, 1.0),
            1 => settings.effect_vol = (settings.effect_vol + delta).clamp(0.0, 1.0),
            2 => settings.music_vol = (settings.music_vol + delta).clamp(0.0, 1.0),
            _ => unreachable!("lock.vol_selected_index out of bounds somehow")
        }
        self.settings.master = settings.master_vol;
        self.settings.effects = settings.effect_vol;
        self.settings.music = settings.music_vol;
        self.vol_selected_time = elapsed;

        Some(actions::song::SongAction::SetVolume(settings.get_music_vol()))
    }


    pub fn draw(
        &mut self, 
        list: &mut graphics::RenderableCollection,
        text_layout_contexts: &mut TextLayoutContexts,
    ) {
        let elapsed = self.timer.as_millis();

        // draw the volume things if needed
        if self.vol_selected_time > 0.0
        && elapsed - self.vol_selected_time < VOLUME_CHANGE_DISPLAY_TIME {
            const BOX_SIZE:Vector2 = Vector2::new(300.0, 100.0);
            let b = graphics::Rectangle::new(
                BOX_SIZE,
                Color::WHITE,
            ).border(Border::new(Color::BLACK, 1.2))
            .with_transform(tataku::Matrix::identity()
                .trans(self.window_size - BOX_SIZE)
            );

            // text 100px wide, bar 190px (10px padding)
            let border_padding = 10.0;
            let border_size = Vector2::new(200.0 - border_padding, 20.0);

            list.push(b);

            for (n, (text, value)) in [
                ("Master:", self.settings.master),
                ("Effects:", self.settings.effects),
                ("Music:", self.settings.music),
            ].iter().enumerate() {
                let r_offset = Vector2::new(
                    border_size.x + border_padding,
                    (90 - 30 * n) as f32
                );

                let color = if self.vol_selected_index == n as u8 { 
                    Color::RED 
                } else { 
                    Color::BLACK 
                };
                let mut layout = text_layout_contexts.simple_text(
                    text,
                    &TextStyle {
                        font_size: 20.0,
                        font: tataku::DefaultFont::Main,
                        color,
                        ..Default::default()
                    },
                );
                layout.break_all_lines(None);

                // text
                list.push(graphics::Text::new(layout)
                    .with_transform(tataku::Matrix::identity()
                        .trans(self.window_size - Vector2::new(300.0, r_offset.y)),
                    )
                );

                // fill
                list.push(graphics::Rectangle::new(
                    Vector2::new(border_size.x * *value, border_size.y),
                    Color::BLUE,
                ).with_transform(tataku::Matrix::identity()
                    .trans(self.window_size - r_offset)
                ));

                // border
                list.push(graphics::Rectangle::new(
                    border_size,
                    Color::TRANSPARENT,
                ).border(Border::new(Color::RED, 1.0))
                .with_transform(tataku::Matrix::identity()
                    .trans(self.window_size - r_offset)
                ));
            }

        }
    }

    pub fn on_mouse_move(&mut self, mouse_pos: Vector2) {
        let elapsed = self.timer.as_millis();

        let master_pos = Vector2::new(
            self.window_size.x - 300.0,
            self.window_size.y - 90.0
        );
        let effect_pos = Vector2::new(
            self.window_size.x - 300.0,
            self.window_size.y - 60.0
        );
        let music_pos = Vector2::new(
            self.window_size.x - 300.0,
            self.window_size.y - 30.0
        );

        // check if mouse moved over a volume button
        if mouse_pos.x >= master_pos.x
        && self.vol_selected_time > 0.0
        && elapsed - self.vol_selected_time < VOLUME_CHANGE_DISPLAY_TIME
        {
            if mouse_pos.y >= music_pos.y {
                self.vol_selected_index = 2;
                self.vol_selected_time = elapsed;
            } else if mouse_pos.y >= effect_pos.y {
                self.vol_selected_index = 1;
                self.vol_selected_time = elapsed;
            } else if mouse_pos.y >= master_pos.y {
                self.vol_selected_index = 0;
                self.vol_selected_time = elapsed;
            }
        }
    }

    pub fn on_mouse_wheel(
        &mut self,
        delta: f32,
        mods: input::KeyModifiers,
        settings: &mut engine::Settings
    ) -> Option<actions::song::SongAction> {
        if !mods.alt { return None }

        self.change(delta / 10.0, settings)
    }

    // #[cfg(feature="graphics")]
    pub fn on_key_press(
        &mut self,
        key: &Key,
        mods: input::KeyModifiers,
        actions: &mut actions::ActionQueue,
        settings: &mut engine::Settings,
    ) -> bool {
        let elapsed = self.timer.as_millis();

        if !mods.alt { return false }
        let action = match key {
            Key::Right => self.change(0.1, settings),
            Key::Left => self.change(-0.1, settings),
            Key::Up => {
                self.vol_selected_index = (3+(self.vol_selected_index as i8 - 1)) as u8 % 3;
                self.vol_selected_time = elapsed;
                None
            }
            Key::Down => {
                self.vol_selected_index = (self.vol_selected_index + 1) % 3;
                self.vol_selected_time = elapsed;
                None
            }

            _ => return false,
        };
        
        if let Some(action) = action {
            actions.push(action.into());
        }

        true
    }
}


#[derive(Default)]
struct VolumeSettings {
    master: f32,
    music: f32,
    effects: f32,
}

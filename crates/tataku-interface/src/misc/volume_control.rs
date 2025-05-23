use crate::prelude::*;

/// how long should the volume thing be displayed when changed
const VOLUME_CHANGE_DISPLAY_TIME:u64 = 2000;

/// helper to move volume things out of game, cleaning up code
pub struct VolumeControl {
    /// 0-2, 0 = master, 1 = effect, 2 = music
    vol_selected_index: u8, 
    /// when the volume was changed, or the selected index changed
    vol_selected_time: u64,
    timer: TatakuInstant,

    settings: VolumeSettings,
    window_size: Vector2,
}
impl VolumeControl {
    pub fn new() -> Self {
        Self {
            vol_selected_index: 0,
            vol_selected_time: 0,
            timer: TatakuInstant::now(),
            window_size: Vector2::ZERO,
            settings: VolumeSettings::default(),
        }
    }

    fn elapsed(&self) -> u64 {self.timer.elapsed().as_millis() as u64}
    fn _visible(&self) -> bool {
        let elapsed = self.elapsed();
        elapsed - self.vol_selected_time < VOLUME_CHANGE_DISPLAY_TIME
    }

    pub fn window_size_changed(&mut self, window_size: Vector2) {
        self.window_size = window_size;
    }

    fn change(&mut self, delta: f32, settings: &mut Settings) -> Option<SongAction> {
        let elapsed = self.elapsed();

        // reset index back to 0 (master) if the volume hasnt been touched in a while
        if elapsed - self.vol_selected_time > VOLUME_CHANGE_DISPLAY_TIME + 1000 { 
            self.vol_selected_index = 0 
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

        Some(SongAction::SetVolume(settings.get_music_vol()))
    }


    pub fn draw(&mut self, list: &mut RenderableCollection) {
        let elapsed = self.elapsed();

        // draw the volume things if needed
        if self.vol_selected_time > 0 
            && elapsed - self.vol_selected_time < VOLUME_CHANGE_DISPLAY_TIME 
            {
            const BOX_SIZE:Vector2 = Vector2::new(300.0, 100.0);
            let b = Rectangle::new(
                self.window_size - BOX_SIZE,
                BOX_SIZE,
                Color::WHITE,
                Some(Border::new(Color::BLACK, 1.2))
            );

            // text 100px wide, bar 190px (10px padding)
            let border_padding = 10.0;
            let border_size = Vector2::new(200.0 - border_padding, 20.0);
            
            // == master bar ==
            // text
            let mut master_text = Text::new(
                self.window_size - Vector2::new(300.0, 90.0),
                20.0,
                "Master:",
                Color::BLACK,
                Font::Main,
            );
            // border
            let master_border = Rectangle::new(
                self.window_size - Vector2::new(border_size.x + border_padding, 90.0),
                border_size,
                Color::TRANSPARENT,
                Some(Border::new(Color::RED, 1.0))
            );
            // fill
            let master_fill = Rectangle::new(
                self.window_size - Vector2::new(border_size.x + border_padding, 90.0),
                Vector2::new(border_size.x * self.settings.master, border_size.y),
                Color::BLUE,
                None
            );

            // == effects bar ==
            // text
            let mut effect_text = Text::new(
                self.window_size - Vector2::new(300.0, 60.0),
                20.0,
                "Effects:",
                Color::BLACK,
                Font::Main
            );
            // border
            let effect_border = Rectangle::new(
                self.window_size - Vector2::new(border_size.x + border_padding, 60.0),
                border_size,
                Color::TRANSPARENT,
                Some(Border::new(Color::RED, 1.0))
            );
            // fill
            let effect_fill = Rectangle::new(
                self.window_size - Vector2::new(border_size.x + border_padding, 60.0),
                Vector2::new(border_size.x * self.settings.effects, border_size.y),
                Color::BLUE,
                None
            );

            // == music bar ==
            // text
            let mut music_text = Text::new(
                self.window_size - Vector2::new(300.0, 30.0),
                20.0,
                "Music:",
                Color::BLACK,
                Font::Main
            );
            // border
            let music_border = Rectangle::new(
                self.window_size - Vector2::new(border_size.x + border_padding, 30.0),
                border_size,
                Color::TRANSPARENT,
                Some(Border::new(Color::RED, 1.0))
            );
            // fill
            let music_fill = Rectangle::new(
                self.window_size - Vector2::new(border_size.x + border_padding, 30.0),
                Vector2::new(border_size.x * self.settings.music, border_size.y),
                Color::BLUE,
                None
            );
            
            // highlight selected index
            match self.vol_selected_index {
                0 => master_text.color = Color::RED,
                1 => effect_text.color = Color::RED,
                2 => music_text.color = Color::RED,
                _ => error!("self.vol_selected_index out of bounds somehow")
            }

            list.push(b);
            list.push(master_text);
            list.push(master_fill);
            list.push(master_border);
            
            list.push(effect_text);
            list.push(effect_fill);
            list.push(effect_border);

            list.push(music_text);
            list.push(music_fill);
            list.push(music_border);
        }
    }

    pub fn on_mouse_move(&mut self, mouse_pos: Vector2) {
        let elapsed = self.elapsed();

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
            && self.vol_selected_time > 0 
            && elapsed as f32 - (self.vol_selected_time as f32) < VOLUME_CHANGE_DISPLAY_TIME as f32 
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
        mods: KeyModifiers, 
        settings: &mut Settings
    ) -> Option<SongAction> {
        if mods.alt {
            self.change(delta / 10.0, settings)
        } else {
            None
        }
    }

    // #[cfg(feature="graphics")]
    pub fn on_key_press(
        &mut self, 
        keys: &mut KeyCollection, 
        mods: KeyModifiers, 
        settings: &mut Settings
    ) -> bool {
        let elapsed = self.elapsed();

        if mods.alt {
            // let mut changed = false;

            if keys.has_and_remove(Key::Right) {
                self.change(0.1, settings);
            }
            if keys.has_and_remove(Key::Left) {
                self.change(-0.1, settings);
            }

            if keys.has_and_remove(Key::Up) {
                self.vol_selected_index = (3+(self.vol_selected_index as i8 - 1)) as u8 % 3;
                self.vol_selected_time = elapsed;
            }
            if keys.has_and_remove(Key::Down) {
                self.vol_selected_index = (self.vol_selected_index + 1) % 3;
                self.vol_selected_time = elapsed;
            }
        }

        false
    }
}


#[derive(Default)]
struct VolumeSettings {
    master: f32,
    music: f32,
    effects: f32,
}

use crate::prelude::*;

/// how long should the volume thing be displayed when changed
const VOLUME_CHANGE_DISPLAY_TIME:u64 = 2000;

/// helper to move volume things out of game, cleaning up code
pub struct VolumeControl {
    /// 0-2, 0 = master, 1 = effect, 2 = music
    vol_selected_index: u8, 
    ///when the volume was changed, or the selected index changed
    vol_selected_time: u64,
    timer: Instant
}
impl VolumeControl {
    pub fn new() -> Self {
        Self {
            vol_selected_index: 0,
            vol_selected_time: 0,
            timer: Instant::now()
        }
    }

    fn elapsed(&self) -> u64 {self.timer.elapsed().as_millis() as u64}
    fn visible(&self) -> bool {
        let elapsed = self.elapsed();
        elapsed - self.vol_selected_time < VOLUME_CHANGE_DISPLAY_TIME
    }
    fn change(&mut self, delta:f32) {
        let elapsed = self.elapsed();
        let mut settings = get_settings_mut!();

        // reset index back to 0 (master) if the volume hasnt been touched in a while
        if elapsed - self.vol_selected_time > VOLUME_CHANGE_DISPLAY_TIME + 1000 {self.vol_selected_index = 0}

        // find out what volume to edit, and edit it
        match self.vol_selected_index {
            0 => settings.master_vol = (settings.master_vol + delta).clamp(0.0, 1.0),
            1 => settings.effect_vol = (settings.effect_vol + delta).clamp(0.0, 1.0),
            2 => settings.music_vol = (settings.music_vol + delta).clamp(0.0, 1.0),
            _ => error!("lock.vol_selected_index out of bounds somehow")
        }

        
        if let Some(song) = Audio::get_song() {
            #[cfg(feature="bass_audio")]
            song.set_volume(settings.get_music_vol()).unwrap();
            #[cfg(feature="neb_audio")]
            song.set_volume(settings.get_music_vol());
        }

        self.vol_selected_time = elapsed;
    }


    pub fn draw(&mut self, _args: RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut list: Vec<Box<dyn Renderable>> = Vec::new();
        let elapsed = self.elapsed();

        // draw the volume things if needed
        if self.vol_selected_time > 0 && elapsed - self.vol_selected_time < VOLUME_CHANGE_DISPLAY_TIME {
            let font = get_font();
            let settings = get_settings!();
            let window_size:Vector2 = settings.window_size.into();

            const BOX_SIZE:Vector2 = Vector2::new(300.0, 100.0);
            let b = Rectangle::new(
                Color::WHITE,
                -7.0,
                window_size - BOX_SIZE,
                BOX_SIZE,
                Some(Border::new(Color::BLACK, 1.2))
            );

            // text 100px wide, bar 190px (10px padding)
            let border_padding = 10.0;
            let border_size = Vector2::new(200.0 - border_padding, 20.0);
            
            // == master bar ==
            // text
            let mut master_text = Text::new(
                Color::BLACK,
                -9.0,
                window_size - Vector2::new(300.0, 90.0),
                20,
                "Master:".to_owned(),
                font.clone(),
            );
            // border
            let master_border = Rectangle::new(
                Color::TRANSPARENT_WHITE,
                -9.0,
                window_size - Vector2::new(border_size.x + border_padding, 90.0),
                border_size,
                Some(Border::new(Color::RED, 1.0))
            );
            // fill
            let master_fill = Rectangle::new(
                Color::BLUE,
                -8.0,
                window_size - Vector2::new(border_size.x + border_padding, 90.0),
                Vector2::new(border_size.x * settings.master_vol as f64, border_size.y),
                None
            );

            // == effects bar ==
            // text
            let mut effect_text = Text::new(
                Color::BLACK,
                -9.0,
                window_size - Vector2::new(300.0, 60.0),
                20,
                "Effects:".to_owned(),
                font.clone()
            );
            // border
            let effect_border = Rectangle::new(
                Color::TRANSPARENT_WHITE,
                -9.0,
                window_size - Vector2::new(border_size.x + border_padding, 60.0),
                border_size,
                Some(Border::new(Color::RED, 1.0))
            );
            // fill
            let effect_fill = Rectangle::new(
                Color::BLUE,
                -8.0,
                window_size - Vector2::new(border_size.x + border_padding, 60.0),
                Vector2::new(border_size.x * settings.effect_vol as f64, border_size.y),
                None
            );

            // == music bar ==
            // text
            let mut music_text = Text::new(
                Color::BLACK,
                -9.0,
                window_size - Vector2::new(300.0, 30.0),
                20,
                "Music:".to_owned(),
                font.clone()
            );
            // border
            let music_border = Rectangle::new(
                Color::TRANSPARENT_WHITE,
                -9.0,
                window_size - Vector2::new(border_size.x + border_padding, 30.0),
                border_size,
                Some(Border::new(Color::RED, 1.0))
            );
            // fill
            let music_fill = Rectangle::new(
                Color::BLUE,
                -8.0,
                window_size - Vector2::new(border_size.x + border_padding, 30.0),
                Vector2::new(border_size.x * settings.music_vol as f64, border_size.y),
                None
            );
            
            // highlight selected index
            match self.vol_selected_index {
                0 => master_text.current_color = Color::RED,
                1 => effect_text.current_color = Color::RED,
                2 => music_text.current_color = Color::RED,
                _ => error!("self.vol_selected_index out of bounds somehow")
            }

            let a:[Box<dyn Renderable>; 10] = [
                Box::new(b),
                Box::new(master_text),
                Box::new(master_border),
                Box::new(master_fill),
                
                Box::new(effect_text),
                Box::new(effect_border),
                Box::new(effect_fill),

                Box::new(music_text),
                Box::new(music_border),
                Box::new(music_fill),
            ];

            list.extend(a);
        }

        list
    }

    pub fn on_mouse_move(&mut self, mouse_pos: Vector2) {
        let elapsed = self.elapsed();
        let window_size = Settings::window_size();

        let master_pos:Vector2 = Vector2::new(window_size.x - 300.0, window_size.y - 90.0);
        let effect_pos:Vector2 = Vector2::new(window_size.x - 300.0, window_size.y - 60.0);
        let music_pos:Vector2 = Vector2::new(window_size.x - 300.0, window_size.y - 30.0);

        // check if mouse moved over a volume button
        if self.vol_selected_time > 0 && elapsed as f64 - (self.vol_selected_time as f64) < VOLUME_CHANGE_DISPLAY_TIME as f64 {
            if mouse_pos.x >= master_pos.x {
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
    }

    pub fn on_mouse_wheel(&mut self, delta:f64, mods:KeyModifiers) -> bool {
        if mods.alt {
            self.change(delta as f32 / 10.0);
            return true
        }

        false
    }

    pub fn on_key_press(&mut self, keys:&mut Vec<Key>, mods:KeyModifiers) -> bool {
        let elapsed = self.elapsed();

        if mods.alt {
            let mut changed = false;

            if keys.contains(&Key::Right) {
                self.change(0.1);
                changed = true;
            }
            if keys.contains(&Key::Left) {
                keys.retain(|k|k == &Key::Left);
                self.change(-0.1);
                changed = true;
            }

            if keys.contains(&Key::Up) {
                self.vol_selected_index = (3+(self.vol_selected_index as i8 - 1)) as u8 % 3;
                self.vol_selected_time = elapsed;
                changed = true;
            }
            if keys.contains(&Key::Down) {
                self.vol_selected_index = (self.vol_selected_index + 1) % 3;
                self.vol_selected_time = elapsed;
                changed = true;
            }

            if changed {
                let remove = vec![&Key::Right, &Key::Left, &Key::Up, &Key::Down];
                keys.retain(|k| remove.contains(&k));
                return true;
            }
        }

        false
    }
}
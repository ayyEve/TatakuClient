impl Settings {
pub fn get_menu_items(&self, p: Vector2, prefix: String, sender: Arc<SyncSender<()>>) -> Vec<Box<dyn ScrollableItem>> {
let mut list:Vec<Box<dyn ScrollableItem>> = Vec::new();
let font = get_font();
list.push(Box::new(MenuSection::new(p, 80.0, "Audio Settings", Color::BLACK, font.clone())));

// global_offset
let mut i = Slider::new(p, Vector2::new(600.0, 50.0), "Global Offset", self.global_offset as f64, Some(-100.0..100.0), None, font.clone());
i.set_tag(&(prefix.clone() + "global_offset"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));
list.push(Box::new(MenuSection::new(p, 80.0, "Tataku Server Settings", Color::BLACK, font.clone())));

// username
let mut i = TextInput::new(p, Vector2::new(600.0, 50.0), "Tataku Username", &self.username, font.clone());
i.set_tag(&(prefix.clone() + "username"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// password
let mut i = TextInput::new(p, Vector2::new(600.0, 50.0), "Tataku Password", &self.password, font.clone());
i.is_password = true;
i.set_tag(&(prefix.clone() + "password"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// server_url
let mut i = TextInput::new(p, Vector2::new(600.0, 50.0), "Tataku Server Url", &self.server_url, font.clone());
i.set_tag(&(prefix.clone() + "server_url"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// score_url
let mut i = TextInput::new(p, Vector2::new(600.0, 50.0), "Tataku Score Url", &self.score_url, font.clone());
i.set_tag(&(prefix.clone() + "score_url"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));
list.push(Box::new(MenuSection::new(p, 80.0, "Osu Integration", Color::BLACK, font.clone())));

// osu_username
let mut i = TextInput::new(p, Vector2::new(600.0, 50.0), "Osu Username", &self.osu_username, font.clone());
i.set_tag(&(prefix.clone() + "osu_username"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// osu_password
let mut i = TextInput::new(p, Vector2::new(600.0, 50.0), "Osu Password", &self.osu_password, font.clone());
i.is_password = true;
i.set_tag(&(prefix.clone() + "osu_password"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// osu_api_key
let mut i = TextInput::new(p, Vector2::new(600.0, 50.0), "Osu Api Key", &self.osu_api_key, font.clone());
i.is_password = true;
i.set_tag(&(prefix.clone() + "osu_api_key"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));
list.push(Box::new(MenuSection::new(p, 80.0, "Osu Settings", Color::BLACK, font.clone())));

// standard_settings
list.extend(self.standard_settings.get_menu_items(p, prefix.clone(), sender.clone()));
list.push(Box::new(MenuSection::new(p, 80.0, "Taiko Settings", Color::BLACK, font.clone())));

// taiko_settings
list.extend(self.taiko_settings.get_menu_items(p, prefix.clone(), sender.clone()));
list.push(Box::new(MenuSection::new(p, 80.0, "Mania Settings", Color::BLACK, font.clone())));

// mania_settings
list.extend(self.mania_settings.get_menu_items(p, prefix.clone(), sender.clone()));
list.push(Box::new(MenuSection::new(p, 80.0, "Background Game Settings", Color::BLACK, font.clone())));

// background_game_settings
list.extend(self.background_game_settings.get_menu_items(p, prefix.clone(), sender.clone()));
list.push(Box::new(MenuSection::new(p, 80.0, "Common Game Settings", Color::BLACK, font.clone())));

// common_game_settings
list.extend(self.common_game_settings.get_menu_items(p, prefix.clone(), sender.clone()));

// allow_gamemode_cursor_ripple_override
let mut i = Checkbox::new(p, Vector2::new(600.0, 50.0), "Gamemode Ripple Override", self.allow_gamemode_cursor_ripple_override, font.clone());
i.set_tag(&(prefix.clone() + "allow_gamemode_cursor_ripple_override"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// beatmap_hitsounds
let mut i = Checkbox::new(p, Vector2::new(600.0, 50.0), "Beatmap Hitsounds", self.beatmap_hitsounds, font.clone());
i.set_tag(&(prefix.clone() + "beatmap_hitsounds"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));
list.push(Box::new(MenuSection::new(p, 80.0, "Window Settings", Color::BLACK, font.clone())));

// fps_target
let mut i = Slider::new(p, Vector2::new(600.0, 50.0), "FPS Limit", self.fps_target as f64, Some(15.0..240.0), None, font.clone());
i.set_tag(&(prefix.clone() + "fps_target"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// vsync
let mut i = Checkbox::new(p, Vector2::new(600.0, 50.0), "Vsync", self.vsync, font.clone());
i.set_tag(&(prefix.clone() + "vsync"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// update_target
let mut i = Slider::new(p, Vector2::new(600.0, 50.0), "Update Limit", self.update_target as f64, Some(500.0..10000.0), None, font.clone());
i.set_tag(&(prefix.clone() + "update_target"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// background_dim
let mut i = Slider::new(p, Vector2::new(600.0, 50.0), "Background Dim", self.background_dim as f64, Some(0.0..1.0), None, font.clone());
i.set_tag(&(prefix.clone() + "background_dim"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// pause_on_focus_lost
let mut i = Checkbox::new(p, Vector2::new(600.0, 50.0), "Pause on Focus Loss", self.pause_on_focus_lost, font.clone());
i.set_tag(&(prefix.clone() + "pause_on_focus_lost"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// raw_mouse_input
let mut i = Checkbox::new(p, Vector2::new(600.0, 50.0), "Raw Mouse Input (requires restart)", self.raw_mouse_input, font.clone());
i.set_tag(&(prefix.clone() + "raw_mouse_input"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// scroll_sensitivity
let mut i = Slider::new(p, Vector2::new(600.0, 50.0), "Scroll Sensitivity", self.scroll_sensitivity as f64, Some(0.1..5.0), None, font.clone());
i.set_tag(&(prefix.clone() + "scroll_sensitivity"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// fullscreen_monitor
let mut i = Dropdown::<FullscreenMonitor>::new(p, 600.0, 20.0, "Fullscreen", Some(self.fullscreen_monitor.clone()), font.clone());
i.set_tag(&(prefix.clone() + "fullscreen_monitor"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// refresh_monitors_button
let mut i = MenuButton::new(p, Vector2::new(600.0, 50.0), "Refresh Monitors", font.clone());
i.on_click = Arc::new(|_|GameWindow::refresh_monitors());
list.push(Box::new(i));
list.push(Box::new(MenuSection::new(p, 80.0, "Cursor Settings", Color::BLACK, font.clone())));

// cursor_color
let mut i = TextInput::new(p, Vector2::new(600.0, 50.0), "Cursor Color", &self.cursor_color, font.clone());
i.set_tag(&(prefix.clone() + "cursor_color"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// cursor_scale
let mut i = Slider::new(p, Vector2::new(600.0, 50.0), "Cursor Scale", self.cursor_scale as f64, Some(0.1..10.0), None, font.clone());
i.set_tag(&(prefix.clone() + "cursor_scale"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// cursor_border
let mut i = Slider::new(p, Vector2::new(600.0, 50.0), "Cursor Border", self.cursor_border as f64, Some(0.1..5.0), None, font.clone());
i.set_tag(&(prefix.clone() + "cursor_border"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// cursor_border_color
let mut i = TextInput::new(p, Vector2::new(600.0, 50.0), "Cursor Border Color", &self.cursor_border_color, font.clone());
i.set_tag(&(prefix.clone() + "cursor_border_color"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// cursor_ripples
let mut i = Checkbox::new(p, Vector2::new(600.0, 50.0), "Cursor Ripples", self.cursor_ripples, font.clone());
i.set_tag(&(prefix.clone() + "cursor_ripples"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// cursor_ripple_color
let mut i = TextInput::new(p, Vector2::new(600.0, 50.0), "Cursor Ripple Color", &self.cursor_ripple_color, font.clone());
i.set_tag(&(prefix.clone() + "cursor_ripple_color"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// cursor_ripple_final_scale
let mut i = Slider::new(p, Vector2::new(600.0, 50.0), "Cursor Ripple Scale", self.cursor_ripple_final_scale as f64, None, None, font.clone());
i.set_tag(&(prefix.clone() + "cursor_ripple_final_scale"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));
list.push(Box::new(MenuSection::new(p, 80.0, "Skin Settings", Color::BLACK, font.clone())));

// current_skin
let mut i = Dropdown::<SkinDropdownable>::new(p, 600.0, 20.0, "Skin", Some(SkinDropdownable::Skin(self.current_skin.clone())), font.clone());
i.set_tag(&(prefix.clone() + "current_skin"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// refresh_skins_button
let mut i = MenuButton::new(p, Vector2::new(600.0, 50.0), "Refresh Skins", font.clone());
i.on_click = Arc::new(|_|SkinManager::refresh_skins());
list.push(Box::new(i));

// theme
let mut i = Dropdown::<SelectedTheme>::new(p, 600.0, 20.0, "Theme", Some(self.theme.clone()), font.clone());
i.set_tag(&(prefix.clone() + "theme"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));
list.push(Box::new(MenuSection::new(p, 80.0, "Common Keybinds", Color::BLACK, font.clone())));

// key_user_panel
let mut i = KeyButton::new(p, Vector2::new(600.0, 50.0), self.key_user_panel, "User Panel Key", font.clone());
i.set_tag(&(prefix.clone() + "key_user_panel"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));
list.push(Box::new(MenuSection::new(p, 80.0, "DoubleTap Protection", Color::BLACK, font.clone())));

// enable_double_tap_protection
let mut i = Checkbox::new(p, Vector2::new(600.0, 50.0), "Enable DoubleTap Protection", self.enable_double_tap_protection, font.clone());
i.set_tag(&(prefix.clone() + "enable_double_tap_protection"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// double_tap_protection_duration
let mut i = Slider::new(p, Vector2::new(600.0, 50.0), "DoubleTap Protection Leniency", self.double_tap_protection_duration as f64, Some(10.0..200.0), None, font.clone());
i.set_tag(&(prefix.clone() + "double_tap_protection_duration"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));
list.push(Box::new(MenuSection::new(p, 80.0, "Integrations", Color::BLACK, font.clone())));

// integrations
list.extend(self.integrations.get_menu_items(p, prefix.clone(), sender.clone()));
list.push(Box::new(MenuSection::new(p, 80.0, "Log Settings", Color::BLACK, font.clone())));

// logging_settings
list.extend(self.logging_settings.get_menu_items(p, prefix.clone(), sender.clone()));
list
}
pub fn from_menu(&mut self, prefix: String, list: &ScrollableArea) {

// global_offset

                if let Some(val) = list.get_tagged(prefix.clone() + "global_offset").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<f64>().expect(&format!("error downcasting for global_offset"));
                    
                    self.global_offset = (*val) as f32; 
                }

// username

                if let Some(val) = list.get_tagged(prefix.clone() + "username").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<String>().expect(&format!("error downcasting for username"));
                    self.username = val.clone(); 
                }

// password

                if let Some(val) = list.get_tagged(prefix.clone() + "password").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<String>().expect(&format!("error downcasting for password"));
                    self.password = val.clone(); 
                }

// server_url

                if let Some(val) = list.get_tagged(prefix.clone() + "server_url").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<String>().expect(&format!("error downcasting for server_url"));
                    self.server_url = val.clone(); 
                }

// score_url

                if let Some(val) = list.get_tagged(prefix.clone() + "score_url").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<String>().expect(&format!("error downcasting for score_url"));
                    self.score_url = val.clone(); 
                }

// osu_username

                if let Some(val) = list.get_tagged(prefix.clone() + "osu_username").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<String>().expect(&format!("error downcasting for osu_username"));
                    self.osu_username = val.clone(); 
                }

// osu_password

                if let Some(val) = list.get_tagged(prefix.clone() + "osu_password").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<String>().expect(&format!("error downcasting for osu_password"));
                    self.osu_password = val.clone(); 
                }

// osu_api_key

                if let Some(val) = list.get_tagged(prefix.clone() + "osu_api_key").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<String>().expect(&format!("error downcasting for osu_api_key"));
                    self.osu_api_key = val.clone(); 
                }

// standard_settings
self.standard_settings.from_menu(prefix.clone(), list);

// taiko_settings
self.taiko_settings.from_menu(prefix.clone(), list);

// mania_settings
self.mania_settings.from_menu(prefix.clone(), list);

// background_game_settings
self.background_game_settings.from_menu(prefix.clone(), list);

// common_game_settings
self.common_game_settings.from_menu(prefix.clone(), list);

// allow_gamemode_cursor_ripple_override

                if let Some(val) = list.get_tagged(prefix.clone() + "allow_gamemode_cursor_ripple_override").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<bool>().expect(&format!("error downcasting for allow_gamemode_cursor_ripple_override"));
                    
                    self.allow_gamemode_cursor_ripple_override = val.clone(); 
                }

// beatmap_hitsounds

                if let Some(val) = list.get_tagged(prefix.clone() + "beatmap_hitsounds").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<bool>().expect(&format!("error downcasting for beatmap_hitsounds"));
                    
                    self.beatmap_hitsounds = val.clone(); 
                }

// fps_target

                if let Some(val) = list.get_tagged(prefix.clone() + "fps_target").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<f64>().expect(&format!("error downcasting for fps_target"));
                    
                    self.fps_target = (*val) as u64; 
                }

// vsync

                if let Some(val) = list.get_tagged(prefix.clone() + "vsync").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<bool>().expect(&format!("error downcasting for vsync"));
                    
                    self.vsync = val.clone(); 
                }

// update_target

                if let Some(val) = list.get_tagged(prefix.clone() + "update_target").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<f64>().expect(&format!("error downcasting for update_target"));
                    
                    self.update_target = (*val) as u64; 
                }

// background_dim

                if let Some(val) = list.get_tagged(prefix.clone() + "background_dim").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<f64>().expect(&format!("error downcasting for background_dim"));
                    
                    self.background_dim = (*val) as f32; 
                }

// pause_on_focus_lost

                if let Some(val) = list.get_tagged(prefix.clone() + "pause_on_focus_lost").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<bool>().expect(&format!("error downcasting for pause_on_focus_lost"));
                    
                    self.pause_on_focus_lost = val.clone(); 
                }

// raw_mouse_input

                if let Some(val) = list.get_tagged(prefix.clone() + "raw_mouse_input").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<bool>().expect(&format!("error downcasting for raw_mouse_input"));
                    
                    self.raw_mouse_input = val.clone(); 
                }

// scroll_sensitivity

                if let Some(val) = list.get_tagged(prefix.clone() + "scroll_sensitivity").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<f64>().expect(&format!("error downcasting for scroll_sensitivity"));
                    
                    self.scroll_sensitivity = (*val) as f32; 
                }

// fullscreen_monitor

                    if let Some(val) = list.get_tagged(prefix.clone() + "fullscreen_monitor").first().map(|i|i.get_value()) {
                        let val = val.downcast_ref::<Option<FullscreenMonitor>>().expect(&format!("error downcasting for fullscreen_monitor"));
                        
                        if let Some(val) = val {
                            self.fullscreen_monitor = val.to_owned(); 
                        }
                    }

// refresh_monitors_button

// cursor_color

                if let Some(val) = list.get_tagged(prefix.clone() + "cursor_color").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<String>().expect(&format!("error downcasting for cursor_color"));
                    self.cursor_color = val.clone(); 
                }

// cursor_scale

                if let Some(val) = list.get_tagged(prefix.clone() + "cursor_scale").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<f64>().expect(&format!("error downcasting for cursor_scale"));
                    
                    self.cursor_scale = (*val) as f32; 
                }

// cursor_border

                if let Some(val) = list.get_tagged(prefix.clone() + "cursor_border").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<f64>().expect(&format!("error downcasting for cursor_border"));
                    
                    self.cursor_border = (*val) as f32; 
                }

// cursor_border_color

                if let Some(val) = list.get_tagged(prefix.clone() + "cursor_border_color").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<String>().expect(&format!("error downcasting for cursor_border_color"));
                    self.cursor_border_color = val.clone(); 
                }

// cursor_ripples

                if let Some(val) = list.get_tagged(prefix.clone() + "cursor_ripples").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<bool>().expect(&format!("error downcasting for cursor_ripples"));
                    
                    self.cursor_ripples = val.clone(); 
                }

// cursor_ripple_color

                if let Some(val) = list.get_tagged(prefix.clone() + "cursor_ripple_color").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<String>().expect(&format!("error downcasting for cursor_ripple_color"));
                    self.cursor_ripple_color = val.clone(); 
                }

// cursor_ripple_final_scale

                if let Some(val) = list.get_tagged(prefix.clone() + "cursor_ripple_final_scale").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<f64>().expect(&format!("error downcasting for cursor_ripple_final_scale"));
                    
                    self.cursor_ripple_final_scale = (*val) as f32; 
                }

// current_skin

                    if let Some(val) = list.get_tagged(prefix.clone() + "current_skin").first().map(|i|i.get_value()) {
                        let val = val.downcast_ref::<Option<SkinDropdownable>>().expect(&format!("error downcasting for current_skin"));
                        
                        if let Some(SkinDropdownable::Skin(val)) = val {
                            self.current_skin = val.to_owned(); 
                        }
                    }

// refresh_skins_button

// theme

                    if let Some(val) = list.get_tagged(prefix.clone() + "theme").first().map(|i|i.get_value()) {
                        let val = val.downcast_ref::<Option<SelectedTheme>>().expect(&format!("error downcasting for theme"));
                        
                        if let Some(val) = val {
                            self.theme = val.to_owned(); 
                        }
                    }

// key_user_panel

                if let Some(val) = list.get_tagged(prefix.clone() + "key_user_panel").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<Key>().expect(&format!("error downcasting for key_user_panel"));
                    self.key_user_panel = val.clone(); 
                }

// enable_double_tap_protection

                if let Some(val) = list.get_tagged(prefix.clone() + "enable_double_tap_protection").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<bool>().expect(&format!("error downcasting for enable_double_tap_protection"));
                    
                    self.enable_double_tap_protection = val.clone(); 
                }

// double_tap_protection_duration

                if let Some(val) = list.get_tagged(prefix.clone() + "double_tap_protection_duration").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<f64>().expect(&format!("error downcasting for double_tap_protection_duration"));
                    
                    self.double_tap_protection_duration = (*val) as f32; 
                }

// integrations
self.integrations.from_menu(prefix.clone(), list);

// logging_settings
self.logging_settings.from_menu(prefix.clone(), list);
}
}
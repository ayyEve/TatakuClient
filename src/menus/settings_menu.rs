use crate::prelude::*;

const BUTTON_SIZE:Vector2 = Vector2::new(100.0, 50.0);
const KEYBUTTON_SIZE:Vector2 = Vector2::new(400.0, 50.0);

const SECTION_HEIGHT:f64 = 80.0;
const SECTION_XOFFSET:f64 = 20.0;
const SCROLLABLE_YOFFSET:f64 = 20.0;

pub struct SettingsMenu {
    scroll_area: ScrollableArea,

    finalize_list: Vec<Arc<dyn OnFinalize>>
}
impl SettingsMenu {
    pub fn new() -> SettingsMenu {
        let settings = get_settings!();
        let p = Vector2::new(10.0 + SECTION_XOFFSET, 0.0); // scroll area edits the y
        let window_size = Settings::window_size();

        // setup items
        let mut scroll_area = ScrollableArea::new(Vector2::new(10.0, SCROLLABLE_YOFFSET), Vector2::new(window_size.x - 20.0, window_size.y - SCROLLABLE_YOFFSET*2.0), true);
        let mut finalize_list:Vec<Arc<dyn OnFinalize>> = Vec::new();

        let mut tag_counter = 0;

        let mut make_tag = || {
            tag_counter += 1;
            format!("{}", tag_counter)
        };
        
        macro_rules! convert_return_value {
            // ($item_type:tt, $setting_type:tt, $val:expr) => {};
            (TextInput, String, $val:expr) => {
                $val.to_owned()
            };
            (KeyButton, Key, $val:expr) => {
                *$val
            };
            (Checkbox, bool, $val:expr) => {
                *$val
            };
            (Slider, f32, $val:expr) => {
                *$val as f32
            };
            (Slider, f64, $val:expr) => {
                *$val
            };
        }
        macro_rules! convert_settings_value {
            // ($setting:expr, $t:tt) => {}
            
            ($setting:expr, f32) => {
                *$setting as f32
            };
            ($setting:expr, f64) => {
                *$setting as f64
            };
            ($setting:expr, Key) => {
                *$setting
            };
            ($setting:expr, bool) => {
                *$setting
            };

            ($setting:expr, $t:tt) => {
                $setting
            }
        }

        macro_rules! convert_settings_type {
            (f32) => {
                f64
            };
            ($settings_type: tt) => {
                $settings_type
            }
        }
        let font = get_font();

        macro_rules! add_item {
            ($text:expr, TextInput, $setting:expr) => {
                TextInput::new(p, Vector2::new(600.0, 50.0), $text.clone(), convert_settings_value!($setting, String), font.clone())
            };
            ($text:expr, KeyButton, $setting:expr) => {
                KeyButton::new(p, KEYBUTTON_SIZE, convert_settings_value!($setting, Key), $text.clone(), font.clone())
            };
            ($text:expr, Checkbox, $setting:expr) => {
                Checkbox::new(p, Vector2::new(200.0, BUTTON_SIZE.y), $text.clone(), convert_settings_value!($setting, bool), font.clone())
            };
            ($text:expr, Slider, $setting:expr) => {
                Slider::new(p, Vector2::new(400.0, BUTTON_SIZE.y), $text.clone(), convert_settings_value!($setting, f64), None, None, font.clone())
            };
            

            // menu section
            ($text:expr, MenuSection) => {
                scroll_area.add_item(Box::new(MenuSection::new(
                    p - Vector2::new(SECTION_XOFFSET, 0.0), 
                    SECTION_HEIGHT, 
                    $text, 
                    font.clone()
                )));
            };

            // input item
            // ($text:expr, $item_type:tt, $setting:ident, $setting_type:tt, $struct_name:ident, $($setting2:ident)?, $(setting3:ident)?) => {
            ($text:expr, $item_type:tt, $setting:ident, $setting_type:tt, $struct_name:ident, $mod_fn:expr) => {
                // create a tag
                let tag = make_tag();

                // create and add text item
                let mut item = add_item!($text, $item_type, &settings.$setting);
                item.set_tag(tag.as_str());
                $mod_fn(&mut item);
                scroll_area.add_item(Box::new(item));

                // idk how to do this better 
                struct $struct_name {
                    tag: String
                }
                impl OnFinalize for $struct_name {
                    fn on_finalize(&self, menu: &SettingsMenu, settings: &mut Settings) {
                        let val = menu.scroll_area.get_tagged(self.tag.clone());
                        let val = val.first().expect("error getting tagged");
                        let val = val.get_value();
                        let val = val.downcast_ref::<convert_settings_type!($setting_type)>()
                            .expect(&format!("error downcasting for {} ({})", self.tag, $text));
                        
                        settings.$setting = convert_return_value!($item_type, $setting_type, val);
                    }
                }

                finalize_list.push(Arc::new($struct_name{tag:tag.to_owned()}))
            };
            ($text:expr, $item_type:tt, $setting:ident, $setting2:ident, $setting_type:tt, $struct_name:ident, $mod_fn:expr) => {
                // create a tag
                let tag = make_tag();

                // create and add text item
                let mut item = add_item!($text, $item_type, &settings.$setting.$setting2);
                item.set_tag(tag.as_str());
                scroll_area.add_item(Box::new(item));

                // idk how to do this better 
                struct $struct_name {
                    tag: String
                }
                impl OnFinalize for $struct_name {
                    fn on_finalize(&self, menu: &SettingsMenu, settings: &mut Settings) {
                        let val = menu.scroll_area.get_tagged(self.tag.clone());
                        let val = val.first().expect("error getting tagged");
                        let val = val.get_value();
                        let val = val.downcast_ref::<convert_settings_type!($setting_type)>()
                            .expect(&format!("error downcasting for {} ({})", self.tag, $text));
                        
                        settings.$setting.$setting2 = convert_return_value!($item_type, $setting_type, val);
                    }
                }

                finalize_list.push(Arc::new($struct_name{tag:tag.to_owned()}))
            }
        }

        // osu login
        add_item!("osu! Login", MenuSection);
        add_item!("Username", TextInput, osu_username, String, OsuUsername, |_|{});
        add_item!("Password", TextInput, osu_password, String, OsuPassword, |_|{});

        // taiko keys
        add_item!("Key bindings", MenuSection);
        add_item!("Left Kat", KeyButton, taiko_settings, left_kat, Key, TaikoLeftKat, |_|{});
        add_item!("Left Don", KeyButton, taiko_settings, left_don, Key, TaikoLeftDon, |_|{});
        add_item!("Right Don", KeyButton, taiko_settings, right_don, Key, TaikoRightDon, |_|{});
        add_item!("Right Kat", KeyButton, taiko_settings, right_kat, Key, TaikoRightKat, |_|{});

        // sv
        add_item!("No Sv Changes", Checkbox, taiko_settings, static_sv, bool, TaikoSvChange, |_|{});
        add_item!("Slider Multiplier", Slider, taiko_settings, sv_multiplier, f32, TaikoSliderMultiplier, |thing:&mut Slider| {
            // thing.range = Some(0.1..2.0);
        });

        // bg
        add_item!("Background", MenuSection);
        add_item!("Background Dim", Slider, background_dim, f32, BackgroundDim, |_thing:&mut Slider| {
            // thing.range = Some(0.0..1.0);
        });

        // done button
        let mut done_button = MenuButton::new(p, BUTTON_SIZE, "Done", font.clone());
        done_button.set_tag("done");
        //TODO: make this not part of the scrollable?!?!
        scroll_area.add_item(Box::new(done_button));

        SettingsMenu {
            scroll_area,
            finalize_list
        }
    }

    pub fn finalize(&mut self, game:&mut Game) {
        // write settings to settings
        let mut settings = get_settings_mut!();

        let list = std::mem::take(&mut self.finalize_list);
        for i in list {
            i.on_finalize(self, &mut settings)
        }

        settings.check_hashes();
        settings.save();

        let menu = game.menus.get("main").unwrap().clone();
        game.queue_state_change(GameState::InMenu(menu));
    }
}
impl Menu<Game> for SettingsMenu {
    fn draw(&mut self, args:RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut list: Vec<Box<dyn Renderable>> = Vec::new();
        self.scroll_area.draw(args, Vector2::zero(), 0.0, &mut list);
        let window_size = Settings::window_size();

        // background
        list.push(visibility_bg(
            Vector2::new(10.0, SCROLLABLE_YOFFSET), 
            Vector2::new(window_size.x - 20.0, window_size.y - SCROLLABLE_YOFFSET*2.0),
            10.0
        ));

        list
    }

    fn on_click(&mut self, pos:Vector2, button:MouseButton, mods:KeyModifiers, game:&mut Game) {
        if let Some(tag) = self.scroll_area.on_click_tagged(pos, button, mods) {
            match tag.as_str() {
                "done" => self.finalize(game),
                _ => {}
            }
        }
    }

    fn on_key_press(&mut self, key:piston::Key, game:&mut Game, mods:KeyModifiers) {
        self.scroll_area.on_key_press(key, mods);

        if key == piston::Key::Escape {
            let menu = game.menus.get("main").unwrap().clone();
            game.queue_state_change(GameState::InMenu(menu));
            return;
        }
    }

    fn on_click_release(&mut self, pos:Vector2, button:MouseButton, g:&mut Game) {
        self.scroll_area.on_click_release(pos, button);
    }

    fn update(&mut self, _game: &mut Game) {self.scroll_area.update()}
    fn on_mouse_move(&mut self, pos:Vector2, _game:&mut Game) {self.scroll_area.on_mouse_move(pos)}
    fn on_scroll(&mut self, delta:f64, _game:&mut Game) {self.scroll_area.on_scroll(delta);}
    fn on_text(&mut self, text:String) {self.scroll_area.on_text(text)}
}
impl ControllerInputMenu<Game> for SettingsMenu {
    
}


trait OnFinalize {
    fn on_finalize(&self, menu: &SettingsMenu, settings: &mut Settings);
}

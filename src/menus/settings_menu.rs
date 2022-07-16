use crate::prelude::*;

const BUTTON_SIZE:Vector2 = Vector2::new(100.0, 50.0);
const KEYBUTTON_SIZE:Vector2 = Vector2::new(400.0, 50.0);

const SECTION_HEIGHT:f64 = 80.0;
const SECTION_XOFFSET:f64 = 20.0;
const SCROLLABLE_YOFFSET:f64 = 20.0;

pub struct SettingsMenu {
    scroll_area: ScrollableArea,

    finalize_list: Vec<Arc<dyn OnFinalize>>,
    window_size: Arc<WindowSize>,
}
impl SettingsMenu {
    pub async fn new() -> SettingsMenu {
        let settings = get_settings!();
        let p = Vector2::new(10.0 + SECTION_XOFFSET, 0.0); // scroll area edits the y
        let window_size = WindowSize::get();

        // setup items
        let mut scroll_area = ScrollableArea::new(Vector2::new(10.0, SCROLLABLE_YOFFSET), Vector2::new(window_size.x - 20.0, window_size.y - SCROLLABLE_YOFFSET*2.0), true);
        let mut finalize_list:Vec<Arc<dyn OnFinalize>> = Vec::new();

        // i really need to setup a proc macro for this instead
        // this is stupid

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
            (Dropdown, T, $val:expr) => {
                $val
            }
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
            };
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
                TextInput::<Font2, Text>::new(p, Vector2::new(600.0, 50.0), $text.clone(), convert_settings_value!($setting, String), font.clone())
            };
            ($text:expr, KeyButton, $setting:expr) => {
                KeyButton::<Font2, Text>::new(p, KEYBUTTON_SIZE, convert_settings_value!($setting, Key), $text.clone(), font.clone())
            };
            ($text:expr, Checkbox, $setting:expr) => {
                Checkbox::<Font2, Text>::new(p, Vector2::new(200.0, BUTTON_SIZE.y), $text.clone(), convert_settings_value!($setting, bool), font.clone())
            };
            ($text:expr, Slider, $setting:expr) => {
                Slider::<Font2, Text>::new(p, Vector2::new(400.0, BUTTON_SIZE.y), $text.clone(), convert_settings_value!($setting, f64), None, None, font.clone())
            };
            ($text:expr, Dropdown, $dropdown_type:tt, $dropdown_value:ident, $setting:expr) => {
                Dropdown::<$dropdown_type, Font2, Text>::new(p, 400.0, FontSize::new(20.0).unwrap(), $text.clone(), Some($dropdown_type::$dropdown_value($setting.clone())), font.clone())
            };
            

            // menu section
            ($text:expr, MenuSection) => {
                scroll_area.add_item(Box::new(MenuSection::<Font2, Text>::new(
                    p - Vector2::new(SECTION_XOFFSET, 0.0), 
                    SECTION_HEIGHT, 
                    $text, 
                    font.clone()
                )));
            };

            // input item
            // this one is special for dropdowns
            ($text:expr, Dropdown, $dropdown_type:tt, $dropdown_value:ident, $setting:ident, $struct_name:ident, $mod_fn:expr) => {
                // create a tag
                let tag = make_tag();

                // create and add text item
                let mut item:Dropdown<$dropdown_type, Font2, Text> = add_item!($text, Dropdown, $dropdown_type, $dropdown_value, &settings.$setting);
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
                        let val = val.downcast_ref::<Option<$dropdown_type>>()
                            .expect(&format!("error downcasting for {} ({})", self.tag, $text));
                        
                        if let Some($dropdown_type::$dropdown_value(val)) = val {
                            settings.$setting = val.to_owned(); 
                        }
                    }
                }

                finalize_list.push(Arc::new($struct_name{tag:tag.to_owned()}))
            };
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
                        
                        settings.$setting.$setting2 = convert_return_value!($item_type, $setting_type, val);
                    }
                }

                finalize_list.push(Arc::new($struct_name{tag:tag.to_owned()}))
            }
        }

        // tataku login
        add_item!("Tataku Login", MenuSection);
        add_item!("Server Url", TextInput, server_url, String, TatakuServerUrl, |_|{});
        add_item!("Username", TextInput, username, String, TatakuUsername, |_|{});
        add_item!("Password", TextInput, password, String, TatakuPassword, |_|{});

        // osu login
        add_item!("osu! Login", MenuSection);
        add_item!("Username", TextInput, osu_username, String, OsuUsername, |_|{});
        add_item!("Password", TextInput, osu_password, String, OsuPassword, |_|{});
        add_item!("Api Key", TextInput, osu_api_key, String, OsuApiKey, |_|{});

        // window items
        // todo

        // gameplay settings
        add_item!("Gameplay Settings", MenuSection);
        add_item!("Pause on focus loss", Checkbox, pause_on_focus_lost, bool, PauseOnFocusLost, |_|{});
        add_item!("Background Dim", Slider, background_dim, f32, BackgroundDim, |thing:&mut Slider<Font2, Text>| {
            thing.range = 0.0..1.0;
        });
        add_item!("Global Offset", Slider, global_offset, f32, GlobalOffset, |thing:&mut Slider<Font2, Text>| {
            thing.range = -100.0..100.0;
        });
        add_item!("Skin", Dropdown, SkinDropdownable, Skin, current_skin, CurrentSkin, |_|{});

        
        // cursor
        add_item!("Cursor Settings", MenuSection);
        add_item!("Cursor Color", TextInput, cursor_color, String, CursorColor, |_|{});
        add_item!("Cursor Scale", Slider, cursor_scale, f64, CursorScale, |thing:&mut Slider<Font2, Text>| {
            thing.range = 0.1..20.0;
        });
        add_item!("Cursor Border Color", TextInput, cursor_border_color, String, CursorBorderColor, |_|{});
        add_item!("Cursor Border Size", Slider, cursor_border, f32, CursorBorderSize, |thing:&mut Slider<Font2, Text>| {
            thing.range = 0.1..5.0;
        });

        
        // osu settings
        add_item!("Osu Settings", MenuSection);
        add_item!("Key 1", KeyButton, standard_settings, left_key, Key, OsuKey1, |_|{});
        add_item!("Key 2", KeyButton, standard_settings, right_key, Key, OsuKey2, |_|{});
        add_item!("Ignore Mouse Buttons", Checkbox, standard_settings, ignore_mouse_buttons, bool, OsuIgnoreMouseButtons, |_|{});
        add_item!("Follow Points", Checkbox, standard_settings, draw_follow_points, bool, OsuDrawFollowPoints, |_|{});
        add_item!("Display 300s", Checkbox, standard_settings, show_300s, bool, OsuShow300s, |_|{});
        add_item!("Hit Ripples", Checkbox, standard_settings, hit_ripples, bool, OsuHitRipples, |_|{});
        add_item!("Slider Tick Ripples", Checkbox, standard_settings, slider_tick_ripples, bool, OsuSliderTickRipples, |_|{});
        add_item!("Ripple Scale", Slider, standard_settings, ripple_scale, f64, OsuRippleScale, |slider:&mut Slider<Font2, Text>| {
            slider.range = 0.1..5.0;
        });
        add_item!("Beatmap Combo Colors", Checkbox, standard_settings, use_beatmap_combo_colors, bool, OsuBeatmapComboColors, |_|{});


        // taiko settings
        add_item!("Taiko Settings", MenuSection);
        add_item!("Left Kat", KeyButton, taiko_settings, left_kat, Key, TaikoLeftKat, |_|{});
        add_item!("Left Don", KeyButton, taiko_settings, left_don, Key, TaikoLeftDon, |_|{});
        add_item!("Right Don", KeyButton, taiko_settings, right_don, Key, TaikoRightDon, |_|{});
        add_item!("Right Kat", KeyButton, taiko_settings, right_kat, Key, TaikoRightKat, |_|{});
        add_item!("Ignore Mouse Buttons", Checkbox, taiko_settings, ignore_mouse_buttons, bool, TaikoIgnoreMouseButtons, |_|{});

        add_item!("No Sv Changes", Checkbox, taiko_settings, static_sv, bool, TaikoSvChange, |_|{});
        add_item!("Slider Multiplier", Slider, taiko_settings, sv_multiplier, f32, TaikoSliderMultiplier, |thing:&mut Slider<Font2, Text>| {
            thing.range = 0.1..2.0;
        });

        add_item!("Don Color", TextInput, taiko_settings, don_color_hex, String, TaikoDonColor, |_|{});
        add_item!("Kat Color", TextInput, taiko_settings, kat_color_hex, String, TaikoKatColor, |_|{});
        add_item!("Note Radius", Slider, taiko_settings, note_radius, f64, TaikoNoteRadius, |slider:&mut Slider<Font2, Text>| {
            slider.range = 1.0..100.0;
        });
        add_item!("Big Note Size Multiplier", Slider, taiko_settings, big_note_multiplier, f64, TaikoBigNoteMultiplier, |slider:&mut Slider<Font2, Text>| {
            slider.range = 1.0..5.0;
        });

        add_item!("Hit Area Radius Multiplier", Slider, taiko_settings, hit_area_radius_mult, f64, TaikoHitAreaMult, |slider:&mut Slider<Font2, Text>| {
            slider.range = 1.0..5.0;
        });
        add_item!("Playfield Height Padding", Slider, taiko_settings, playfield_height_padding, f64, TaikoPlayfieldHeightPadding, |slider:&mut Slider<Font2, Text>| {
            slider.range = 0.0..50.0;
        });

        
        // mania settings
        add_item!("Mania Settings", MenuSection);
        add_item!("Judgements Per Column", Checkbox, mania_settings, judgements_per_column, bool, ManiaColumnJudgements, |_|{});
        add_item!("Judgement Offset", Slider, mania_settings, judgement_indicator_offset, f64, ManiaJudgementOffset, |slider:&mut Slider<Font2, Text>| {
            slider.range = 0.0..500.0;
        });
        add_item!("SV Change Delta", Slider, mania_settings, sv_change_delta, f32, ManiaSvDelta, |slider:&mut Slider<Font2, Text>| {
            slider.range = 0.1..10.0;
        });


        // done button
        let mut done_button = MenuButton::<Font2, Text>::new(p, BUTTON_SIZE, "Done", font.clone());
        done_button.set_tag("done");
        //TODO: make this not part of the scrollable?!?!
        scroll_area.add_item(Box::new(done_button));

        SettingsMenu {
            scroll_area,
            finalize_list,
            window_size,
        }
    }

    pub async fn finalize(&mut self, game:&mut Game) {
        // write settings to settings
        let mut settings = get_settings_mut!();

        let list = std::mem::take(&mut self.finalize_list);
        for i in list {
            i.on_finalize(self, &mut settings)
        }

        settings.check_hashes();
        settings.save().await;

        let menu = game.menus.get("main").unwrap().clone();
        game.queue_state_change(GameState::InMenu(menu));
    }
}

#[async_trait]
impl AsyncMenu<Game> for SettingsMenu {
    async fn window_size_changed(&mut self, window_size: Arc<WindowSize>) {
        self.scroll_area.set_size(Vector2::new(window_size.x - 20.0, window_size.y - SCROLLABLE_YOFFSET*2.0));
        self.window_size = window_size;
    }

    
    async fn draw(&mut self, args:RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut list: Vec<Box<dyn Renderable>> = Vec::new();
        self.scroll_area.draw(args, Vector2::zero(), 0.0, &mut list);

        // background
        list.push(visibility_bg(
            Vector2::new(10.0, SCROLLABLE_YOFFSET), 
            Vector2::new(self.window_size.x - 20.0, self.window_size.y - SCROLLABLE_YOFFSET*2.0),
            10.0
        ));

        list
    }

    async fn on_click(&mut self, pos:Vector2, button:MouseButton, mods:KeyModifiers, game:&mut Game) {
        if let Some(tag) = self.scroll_area.on_click_tagged(pos, button, mods) {
            match tag.as_str() {
                "done" => self.finalize(game).await,
                _ => {}
            }
        }
    }

    async fn on_key_press(&mut self, key:piston::Key, game:&mut Game, mods:KeyModifiers) {
        self.scroll_area.on_key_press(key, mods);

        if key == piston::Key::Escape {
            let menu = game.menus.get("main").unwrap().clone();
            game.queue_state_change(GameState::InMenu(menu));
            return;
        }
    }

    async fn on_click_release(&mut self, pos:Vector2, button:MouseButton, _g:&mut Game) {
        self.scroll_area.on_click_release(pos, button);
    }

    async fn update(&mut self, _game: &mut Game) {self.scroll_area.update()}
    async fn on_mouse_move(&mut self, pos:Vector2, _game:&mut Game) {self.scroll_area.on_mouse_move(pos)}
    async fn on_scroll(&mut self, delta:f64, _game:&mut Game) {self.scroll_area.on_scroll(delta);}
    async fn on_text(&mut self, text:String) {self.scroll_area.on_text(text)}
}
impl ControllerInputMenu<Game> for SettingsMenu {
    
}


trait OnFinalize: Send + Sync {
    fn on_finalize(&self, menu: &SettingsMenu, settings: &mut Settings);
}

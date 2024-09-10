use crate::prelude::*;

// const BUTTON_SIZE:Vector2 = Vector2::new(100.0, 50.0);
// const SECTION_XOFFSET:f32 = 30.0;
// const SCROLLABLE_YOFFSET:f32 = 20.0;
// const WIDTH:f32 = 600.0;

// const SEARCH_HEIGHT:f32 = 50.0;

pub struct SettingsMenu {
    num: usize, 

    filter_text: String,

    // scroll_area: ScrollableArea,
    // search_text: TextInput,

    old_settings: Settings,

    // window_size: Arc<WindowSize>,
    // change_receiver: AsyncMutex<Receiver<()>>,
    mouse_pos: Vector2,

    should_close: bool,

    last_click_was_us: bool,
}
impl SettingsMenu {
    pub async fn new() -> SettingsMenu {
        SettingsMenu {
            num: 0,
            filter_text: String::new(),
            old_settings: Settings::get().as_ref().clone(),
            should_close: false,
            mouse_pos: Vector2::ZERO,
            last_click_was_us: false
        }
    }

    pub fn revert(&mut self) { 
        let mut s = Settings::get_mut();
        *s = self.old_settings.clone();
        s.skip_autosaveing = false;

        self.should_close = true;
    }
    pub fn finalize(&mut self) {
        // self.update_settings().await;
        let mut settings = Settings::get_mut();
        settings.skip_autosaveing = false;
        settings.check_hashes();

        self.should_close = true;
    }

}

#[async_trait]
impl Dialog for SettingsMenu {
    fn name(&self) -> &'static str { "settings_menu" }
    fn get_num(&self) -> usize { self.num }
    fn set_num(&mut self, num: usize) { self.num = num }
    fn should_close(&self) -> bool { self.should_close }
    async fn force_close(&mut self) { self.finalize() }



    async fn handle_message(&mut self, message: Message, _values: &mut dyn Reflect) {
        let Some(tag) = message.tag.clone().as_string() else { return };
        let mut tags = tag.split(".");
        
        let Some(first) = tags.next() else { return println!("no first?") };
        // if first != "settings" { return println!("first not settings?") };

        // let Some(prop) = props.next() else { return println!("no second?") };


        match first {
            "done" => self.finalize(),
            "revert" => self.revert(),
            "search" => if let Some(text) = message.message_type.as_text() { self.filter_text = text; },

            _ => Settings::get_mut().from_elements(&mut tags, message),
        }

    }
    
    fn view(&self, _values: &mut dyn Reflect) -> IcedElement {
        use iced_elements::*;

        // build settings list
        let owner = MessageOwner::new_dialog(self);
        let mut builder = SettingsBuilder::default();
        Settings::get().into_elements(
            "settings".to_owned(), 
            &ItemFilter::new(
                self.filter_text.clone().split(" ").map(String::from).collect(), 
                QueryType::Any
            ), 
            owner, 
            &mut builder
        );

        let items = builder.categories
            .into_iter()
            .filter(|(_,(a,_))|!a.is_empty())
            .map(|(name, (props, vals))|
            [
                // space
                row!( Text::new(" ").size(40.0); ),
                // category name
                row!( Text::new(name).size(40.0); ),
                // settings
                CullingColumn::with_children(
                    props.into_iter()
                        .zip(vals.into_iter())
                        .map(|(p,v)| 
                            row!(p, v; align_items = Alignment::Center, spacing = 5.0))
                        .collect()
                )
                .spacing(5.0)
                .into_element()
            ]
        ).flatten().collect();

        let window_size = WindowSize::get().0;
        
        col!(
            Text::new("Settings").size(40.0),

            // space
            Text::new(" ").size(10.0),

            // search text
            TextInput::new("Search", &self.filter_text)
                .size(30.0)
                .on_input(move |t| Message::new(owner, "search", MessageType::Text(t))),

            // space
            Text::new(" ").size(40.0),

            // items
            make_panel_scroll(items, "settings_list")
                .width(Shrink)
                .axis(iced::advanced::layout::flex::Axis::Vertical)
            ,

            // revert
            Button::new(Text::new("Revert").into_element())
                .on_press(Message::new(owner, "revert", MessageType::Click)),

            // done
            Button::new(Text::new("Done").into_element())
                .on_press(Message::new(owner, "done", MessageType::Click))
            ;
            width = Fixed(window_size.x / 3.0),
            height = Fill,
            spacing = 5.0
        )
    }



    // fn get_bounds(&self) -> Bounds {
    //     Bounds::new(
    //         Vector2::ZERO, 
    //         Vector2::new(WIDTH + SECTION_XOFFSET * 2.0, self.window_size.y)
    //     )
    // }

    // async fn window_size_changed(&mut self, window_size: Arc<WindowSize>) {
    //     self.scroll_area.set_size(Vector2::new(WIDTH + SECTION_XOFFSET+1.0, window_size.y - (SEARCH_HEIGHT + SCROLLABLE_YOFFSET*2.0)));
    //     self.window_size = window_size.clone();
    // }
    
    // async fn update(&mut self, _game: &mut Game) {
    //     if let Ok(Ok(_)) = self.change_receiver.try_lock().map(|e|e.try_recv()) {
    //         self.update_settings().await;
    //     }

    //     self.scroll_area.update();
        
    //     let old_text = self.search_text.get_text();
    //     self.search_text.update();

    //     if old_text != self.search_text.get_text() {
    //         self.apply_filter(true);
    //     }
    // }
    // async fn draw(&mut self, offset: Vector2, list: &mut RenderableCollection) {
    //     // background
    //     let bounds = self.get_bounds();
    //     list.push(visibility_bg(
    //         bounds.pos+offset, 
    //         bounds.size,
    //     ));

    //     self.search_text.draw(offset, list);
    //     self.scroll_area.draw(offset, list);
    // }

    // async fn on_mouse_down(&mut self, pos:Vector2, button:MouseButton, mods:&KeyModifiers, _game:&mut Game) -> bool {
    //     if !self.get_bounds().contains(pos) { return false }
    //     self.last_click_was_us = true;
    //     info!("click");
    //     self.search_text.on_click(pos, button, *mods);

    //     if let Some(tag) = self.scroll_area.on_click_tagged(pos, button, *mods) {
    //         match tag.as_str() {
    //             "done" => self.finalize().await,
    //             "revert" => self.revert().await,
    //             _ => {}
    //         }
    //     }

    //     true
    // }

    // async fn on_mouse_up(&mut self, pos:Vector2, button:MouseButton, _mods:&KeyModifiers, _g:&mut Game) -> bool {
    //     if !self.last_click_was_us { return false }
    //     self.last_click_was_us = false;
    //     info!("unclick");

    //     self.search_text.on_click_release(pos, button);
    //     self.scroll_area.on_click_release(pos, button);
    //     true
    // }

    // async fn on_key_press(&mut self, key:Key, mods:&KeyModifiers, _game:&mut Game) -> bool {
    //     if self.search_text.get_selected() {
    //         let old_text = self.search_text.get_text();
    //         self.search_text.on_key_press(key, *mods);

    //         let new_text = self.search_text.get_text();
    //         if new_text != old_text {
    //             self.apply_filter(true);
    //         }

    //         return true;
    //     }

    //     if self.scroll_area.get_selected_index().is_none() { return false }
    //     self.scroll_area.on_key_press(key, *mods);

    //     true
    // }

    // async fn on_key_release(&mut self, key:Key, _mods:&KeyModifiers, _game:&mut Game) -> bool {
    //     if self.search_text.get_selected() {
    //         self.search_text.on_key_release(key);
    //         return true
    //     }
    //     if self.scroll_area.get_selected_index().is_none() { return false }

    //     self.scroll_area.on_key_release(key);
    //     true
    // }

    // async fn on_mouse_move(&mut self, pos:Vector2, _game:&mut Game) {
    //     self.mouse_pos = pos;
    //     self.scroll_area.on_mouse_move(pos);
    //     self.search_text.on_mouse_move(pos)
    // }
    // async fn on_mouse_scroll(&mut self, delta:f32, _game:&mut Game) -> bool {
    //     if !self.get_bounds().contains(self.mouse_pos) { return false }
    //     self.scroll_area.on_scroll(delta);
    //     true
    // }
    // async fn on_text(&mut self, text:&String) -> bool {        
    //     if self.search_text.get_selected() {
    //         let old_text = self.search_text.get_text();
    //         self.search_text.on_text(text.clone());
    //         let new_text = self.search_text.get_text();
    //         if new_text != old_text {
    //             self.apply_filter(true);
    //         }

    //         return true;
    //     }
    //     if self.scroll_area.get_selected_index().is_none() { return false }

    //     self.scroll_area.on_text(text.clone()); 
    //     true
    // }


}


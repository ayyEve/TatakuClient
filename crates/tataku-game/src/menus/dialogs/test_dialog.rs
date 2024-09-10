use crate::prelude::*;

// use tokio::sync::mpsc::{Receiver, channel};

/// this is a dialog i'll use to test stuff with
pub struct TestDialog {
    num: usize,
    // list: ScrollableArea,
    // add_button: MenuButton,
    // search_text: TextInput,

    // receiver: Receiver<()>,

    // last_text: String,

    should_close: bool,

    // yes_no_receiver: Option<Receiver<YesNoResult>>
}
impl TestDialog {
    #[allow(unused)]
    pub fn new() -> Self {
        // let mut add_button = MenuButton::new(Vector2::ZERO, Vector2::new(100.0, 50.0), "Add", Font::Main);
        // let search_text = TextInput::new(Vector2::with_x(110.0), Vector2::new(200.0, 50.0), "Search", "", Font::Main);

        // let (sender, receiver) = channel(10);
        // add_button.on_click = Arc::new(move |_|sender.try_send(()).unwrap());

        // let list = ScrollableArea::new(Vector2::with_y(60.0), Vector2::new(1000.0, 500.0), ListMode::Grid(GridSettings::new(Vector2::new(5.0, 20.0),HorizontalAlign::Center)));

        Self {
            num: 0,
            // list,
            // search_text,
            // add_button,
            // receiver,
            // last_text: String::new(),

            should_close: false,
            // yes_no_receiver: None
        }
    }
}

#[async_trait]
impl Dialog for TestDialog {
    fn name(&self) -> &'static str { "test_weee" }
    fn title(&self) -> &'static str { "Test Dialog" }
    fn get_num(&self) -> usize { self.num }
    fn set_num(&mut self, num: usize) { self.num = num }
    fn should_close(&self) -> bool { self.should_close }
    
    async fn update(&mut self, _values: &mut dyn Reflect) -> Vec<TatakuAction> { 
        // self.manager.update().await;

        Vec::new()
    }
    async fn handle_message(&mut self, _message: Message, _values: &mut dyn Reflect) {
    }
    fn view(&self, _values: &mut dyn Reflect) -> IcedElement {
        // use iced_elements::*;
        EmptyElement.into_element()
    }

    // fn get_bounds(&self) -> Bounds { Bounds::new(Vector2::ZERO, self.list.size() + Vector2::new(0.0, 50.0 + 60.0)) }
    // async fn window_size_changed(&mut self, _window_size: Arc<WindowSize>) {}

    
    // async fn force_close(&mut self) { self.should_close = true; }

    // async fn on_mouse_move(&mut self, pos:Vector2, _g:&mut Game) {
    //     self.list.on_mouse_move(pos);
    //     self.add_button.on_mouse_move(pos);
    //     self.search_text.on_mouse_move(pos);
    // }
    
    // async fn on_mouse_scroll(&mut self, delta:f32, _g:&mut Game) -> bool {
    //     self.list.on_scroll(delta);
    //     true
    // }
    // async fn on_mouse_down(&mut self, pos:Vector2, button:MouseButton, mods:&KeyModifiers, game:&mut Game) -> bool {
    //     if self.list.on_click(pos, button, *mods) && self.yes_no_receiver.is_none() {
    //         let (receiver, dialog) = YesNoDialog::new("Test", "Are you sure?", false);
    //         self.yes_no_receiver = Some(receiver);
    //         game.add_dialog(Box::new(DraggableDialog::new(DraggablePosition::CenterMiddle, Box::new(dialog))), false);
    //     }
    //     self.add_button.on_click(pos, button, *mods);
    //     self.search_text.on_click(pos, button, *mods);
    //     true
    // }
    // async fn on_mouse_up(&mut self, pos:Vector2, button:MouseButton, _mods:&KeyModifiers, _g:&mut Game) -> bool { 
    //     self.list.on_click_release(pos, button);
    //     self.add_button.on_click_release(pos, button);
    //     self.search_text.on_click_release(pos, button);
    //     true
    // }

    
    // async fn on_text(&mut self, text:&String) -> bool {
    //     self.search_text.on_text(text.clone());
    //     true
    // }
    // async fn on_key_press(&mut self, key:Key, mods:&KeyModifiers, _g:&mut Game) -> bool {
    //     self.search_text.on_key_press(key, *mods);
    //     true
    // }
    // async fn on_key_release(&mut self, key:Key, _mods:&KeyModifiers, _g:&mut Game) -> bool {
    //     self.search_text.on_key_release(key);
    //     true
    // }


    // async fn update(&mut self, _g:&mut Game) {
    //     if let Ok(()) = self.receiver.try_recv() {
    //         let c = self.list.items.len() as u32;
    //         let item = PanelUser::new(OnlineUser::new(c, format!("User {c}")));
    //         self.list.add_item(Box::new(item));
    //     }

    //     if let Some(receiver) = &mut self.yes_no_receiver {
    //         if let Ok(result) = receiver.try_recv() {
    //             println!("got result {result:?}")
    //         }
    //     }

    //     let text = self.search_text.get_text();
    //     if text != self.last_text {
    //         if text.is_empty() {
    //             self.list.apply_filter(&Vec::new(), true);
    //         } else {
    //             let query = text.split(" ").map(|a|a.to_owned()).collect::<Vec<String>>();
    //             self.list.apply_filter(&query, true);
    //         }
    //         self.list.refresh_layout();

    //         self.last_text = text;
    //     }
    // }

    // async fn draw(&mut self, offset: Vector2, list: &mut RenderableCollection) {
    //     list.push(visibility_bg(offset, self.get_bounds().size));

    //     self.list.draw(offset, list);
    //     self.add_button.draw(offset, list);
    //     self.search_text.draw(offset, list);
    // }

}



// pub struct StupidDialog {
//     num: usize, 

//     manager: GameplayPreview, //Option<IngameManager>,
//     should_close: bool,

//     // last_offset: Vector2,
//     // size: Vector2,

//     // beatmap: CurrentBeatmapHelper,
//     // mode: CurrentPlaymodeHelper,
// }
// impl StupidDialog {
//     pub async fn new() -> Self {
//         // let mode = CurrentPlaymodeHelper::new();
//         // let beatmap = CurrentBeatmapHelper::new();

//         // let size = Vector2::new(500.0, 500.0);

//         // let mut manager = if let Some(beatmap) = &beatmap.0 {
//         //     manager_from_playmode(mode.0.clone(), &beatmap).await.ok()
//         // } else { None };

//         // if let Some(manager) = &mut manager {
//         //     manager.make_menu_background();
//         //     manager.fit_to_area(Bounds::new(Vector2::ZERO, size)).await;
//         //     manager.start().await;
//         // }

//         Self {
//             num: 0,
//             manager: GameplayPreview::new(true, true, Arc::new(|_|true), MessageOwner::Dialog()),
//             should_close: false,
//             // last_offset: Vector2::ZERO,
//             // size,

//             // beatmap,
//             // mode,
//         }
//     }
// }

// #[async_trait]
// #[allow(unused)]
// impl Dialog for StupidDialog {
//     fn name(&self) -> &'static str { "this is so dumb" }
//     fn title(&self) -> &'static str { "why did i make this" }
//     fn get_num(&self) -> usize { self.num }
//     fn set_num(&mut self, num: usize) { self.num = num }

//     fn resizable(&self) -> bool { true }

//     fn should_close(&self) -> bool { self.should_close }
//     // fn get_bounds(&self) -> Bounds { Bounds::new(Vector2::ZERO, self.size) }
//     // async fn window_size_changed(&mut self, _window_size: Arc<WindowSize>) {
//     //     // self.size = window_size.0;
//     // }
    
//     // async fn resized(&mut self, new_size: Vector2) {
//     //     self.size = new_size;
        
//     //     if let Some(manager) = &mut self.manager {
//     //         manager.fit_to_area(Bounds::new(self.last_offset, self.size)).await;
//     //     }
//     // }

    
//     async fn force_close(&mut self) { self.should_close = true; }


    
//     async fn handle_message(&mut self, _message: Message, _values: &mut ValueCollection) {
//     }

//     // async fn update(&mut self, _values: &mut ShuntingYardValues) -> Vec<TatakuAction> { self.actions.take() }

    
//     fn view(&self, _values: &mut ValueCollection) -> IcedElement {
//         use iced_elements::*;
//         self.manager.widget()
//     }
    

//     // async fn update(&mut self, _g:&mut Game) {
//     //     if self.beatmap.update() || self.mode.update() {
//     //         self.manager = if let Some(beatmap) = &self.beatmap.0 {
//     //             manager_from_playmode(self.mode.0.clone(), &beatmap).await.ok()
//     //         } else { None };
//     //         if let Some(manager) = &mut self.manager {
//     //             manager.make_menu_background();
//     //             manager.fit_to_area(Bounds::new(self.last_offset, self.size)).await;
//     //             manager.start().    await;
//     //         }
//     //     }


//     //     if let Some(manager) = &mut self.manager {
//     //         manager.update().await;
//     //     }
//     // }

//     // async fn draw(&mut self, offset: Vector2, list: &mut RenderableCollection) {
//     //     list.push(Rectangle::new(offset, self.size, Color::GRAY.alpha(0.8), Some(Border::new(Color::BLACK, 2.0))));

//     //     if let Some(manager) = &mut self.manager {
//     //         if offset != self.last_offset {
//     //             self.last_offset = offset;
//     //             // manager.window_size_changed()
//     //             manager.fit_to_area(Bounds::new(offset, self.size)).await;
//     //         }

//     //         manager.draw(list).await;
//     //     }

//     // }

// }

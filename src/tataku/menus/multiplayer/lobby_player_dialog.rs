use crate::prelude::*;
const BUTTON_SIZE:Vector2 = Vector2::new(300.0, 50.0);

pub struct LobbyPlayerDialog {
    user_id: u32,
    should_close: bool,
    window_size: Vector2,

    list: ScrollableArea,
}
impl LobbyPlayerDialog {
    pub fn new(user_id: u32, is_self: bool, we_are_host: bool) -> GenericDialog {
        let mut dialog = GenericDialog::new("lobby_player_dialog");
        dialog.close_after_click = true;

        if we_are_host && !is_self {
            dialog.add_button("Transfer Host", Box::new(move |_,_| { tokio::spawn(OnlineManager::lobby_change_host(user_id)); }));
            dialog.add_button("Kick", Box::new(move |_,_| { tokio::spawn(OnlineManager::lobby_kick_user(user_id)); }));
        }
        dialog.add_button("Close", Box::new(|_,_|{}));
        dialog

        // let window_size = WindowSize::get().0;


        // let mut list = ScrollableArea::new(Vector2::with_x((window_size.x - BUTTON_SIZE.x) / 2.0), Vector2::new(BUTTON_SIZE.x, window_size.y), ListMode::VerticalList);

        // if we_are_host && !is_self {
        //     // make host
        //     list.add_item(Box::new(MenuButton::new(Vector2::ZERO, BUTTON_SIZE, "Transfer Host", Font::Main).with_tag("make_host")));
            
        //     // kick
        //     list.add_item(Box::new(MenuButton::new(Vector2::ZERO, BUTTON_SIZE, "Kick", Font::Main).with_tag("kick")));
        // }

        // // close
        // list.add_item(Box::new(MenuButton::new(Vector2::ZERO, BUTTON_SIZE, "Close", Font::Main).with_tag("close")));

        // Self {
        //     user_id,
        //     should_close: false,
        //     window_size,
        //     list
        // }
    }
}

// #[async_trait]
// impl Dialog<Game> for LobbyPlayerDialog {
//     fn name(&self) -> &'static str { "lobby_player_dialog" }
//     fn should_close(&self) -> bool { self.should_close }
//     fn get_bounds(&self) -> Bounds { Bounds::new(self.list.get_pos(), self.list.size()) }
//     async fn force_close(&mut self) { self.should_close = true; }

//     async fn window_size_changed(&mut self, window_size: Arc<WindowSize>) {
//         self.list.set_size(Vector2::new(BUTTON_SIZE.x, window_size.y));
//         self.list.set_pos(Vector2::with_x((window_size.x - BUTTON_SIZE.x) / 2.0));
//         self.window_size = window_size.0;
//     }
    
//     async fn on_mouse_move(&mut self, pos:Vector2, _g:&mut Game) {
//         self.list.on_mouse_move(pos);
//     }
//     async fn on_mouse_scroll(&mut self, delta:f32, _g:&mut Game) -> bool {
//         self.list.on_scroll(delta)
//     }
//     async fn on_mouse_down(&mut self, pos:Vector2, button:MouseButton, mods:&KeyModifiers, _g:&mut Game) -> bool {
//         if let Some(tag) = self.list.on_click_tagged(pos, button, *mods) {
//             match &*tag {
//                 "close" => self.should_close = true,
//                 "make_host" => {
//                     tokio::spawn(OnlineManager::lobby_change_host(self.user_id));
//                     self.should_close = true;
//                 }
//                 "kick" => {
//                     tokio::spawn(OnlineManager::lobby_kick_user(self.user_id));
//                     self.should_close = true;
//                 }

//                 _ => {}
//             }

//             true
//         } else {
//             false
//         }
//     }
//     async fn on_mouse_up(&mut self, pos:Vector2, button:MouseButton, _mods:&KeyModifiers, _g:&mut Game) -> bool {
//         self.list.on_click_release(pos, button);

//         self.list.get_hover()
//     }

    
//     async fn draw(&mut self, offset: Vector2, list: &mut RenderableCollection) {
//         list.push(visibility_bg(Vector2::ZERO, self.window_size));
//         self.list.draw(offset, list);
//     }
// }
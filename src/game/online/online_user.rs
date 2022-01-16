use crate::prelude::*;

pub const USER_ITEM_SIZE:Vector2 = Vector2::new(200.0, 100.0);
pub const USERNAME_OFFSET:Vector2 = Vector2::new(5.0, 5.0);

#[derive(Clone)]
pub struct OnlineUser {
    pos: Vector2,
    hover: bool,
    selected: bool,

    pub clicked: bool,

    pub user_id: u32,
    pub username: String,
    pub action: Option<UserAction>,
    pub action_text: Option<String>,
    pub mode: Option<PlayMode>,
}
impl OnlineUser {
    pub fn new(user_id:u32, username:String) -> Self {
        Self {
            user_id,
            username,
            action:None,
            action_text: None,
            mode: None,

            clicked: false,

            hover: false,
            selected: false,
            pos: Vector2::zero()
        }
    }
}
impl Default for OnlineUser {
    fn default() -> Self {
        Self { 
            pos: Vector2::zero(), 
            hover: Default::default(), 
            selected: Default::default(), 
            clicked: Default::default(), 
            user_id: Default::default(), 
            username: Default::default(), 
            action: Default::default(), 
            action_text: Default::default(),
            mode: Default::default()
        }
    }
}

impl ScrollableItem for OnlineUser {
    fn size(&self) -> Vector2 {USER_ITEM_SIZE}
    fn get_pos(&self) -> Vector2 {self.pos}
    fn set_pos(&mut self, pos:Vector2) {self.pos = pos}
    fn get_tag(&self) -> String {self.username.clone()}
    fn set_tag(&mut self, _tag:&str) {}

    fn get_hover(&self) -> bool {self.hover}
    fn set_hover(&mut self, hover:bool) {self.hover = hover}
    fn get_selected(&self) -> bool {self.selected}
    fn set_selected(&mut self, selected:bool) {self.selected = selected}


    fn draw(&mut self, _args:piston::RenderArgs, pos:Vector2, depth:f64) -> Vec<Box<dyn Renderable>> {
        let mut list:Vec<Box<dyn Renderable>> = Vec::new();
        let font = get_font("main");

        let pos = self.pos + pos;

        // bounding box
        let c = Color::new(0.5, 0.5, 0.5, 0.75);
        list.push(Box::new(Rectangle::new(
            c,
            depth,
            pos,
            USER_ITEM_SIZE,
            Some(Border::new(if self.hover {Color::RED} else {Color::new(0.75, 0.75, 0.75, 0.75)}, 2.0))
        )));

        // username
        list.push(Box::new(Text::new(
            Color::WHITE,
            depth - 1.0,
            pos + USERNAME_OFFSET,
            20,
            self.username.clone(),
            font.clone()
        )));

        // status
        if let Some(_action) = &self.action {
            
        }
        if let Some(action_text) = &self.action_text {
            list.push(Box::new(Text::new(
                Color::BLACK,
                depth - 1.0,
                pos + USERNAME_OFFSET + Vector2::new(0.0, 20.0),
                20,
                action_text.clone(),
                font.clone()
            )));
        }

        list
    }

    fn on_click(&mut self, _pos:Vector2, _button:piston::MouseButton, _mods: KeyModifiers) -> bool {
        if self.hover {self.clicked = true}
        self.hover
    }

}
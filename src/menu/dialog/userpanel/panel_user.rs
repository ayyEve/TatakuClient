use crate::prelude::*;

pub const USER_ITEM_SIZE:Vector2 = Vector2::new(200.0, 100.0);
pub const USERNAME_OFFSET:Vector2 = Vector2::new(5.0, 5.0);

#[derive(Clone)]
pub struct PanelUser {
    pos: Vector2,
    hover: bool,
    selected: bool,

    pub user: OnlineUser
}
impl PanelUser {
    pub fn new(user: OnlineUser) -> Self {
        Self {
            user,
            hover: false,
            selected: false,
            pos: Vector2::zero()
        }
    }
}
impl Default for PanelUser {
    fn default() -> Self {
        Self { 
            pos: Vector2::zero(), 
            hover: Default::default(), 
            selected: Default::default(), 

            user: Default::default()
        }
    }
}

impl ScrollableItem for PanelUser {
    fn size(&self) -> Vector2 {USER_ITEM_SIZE}
    fn get_pos(&self) -> Vector2 {self.pos}
    fn set_pos(&mut self, pos:Vector2) {self.pos = pos}
    fn get_tag(&self) -> String {self.user.username.clone()}
    fn set_tag(&mut self, _tag:&str) {}

    fn get_hover(&self) -> bool {self.hover}
    fn set_hover(&mut self, hover:bool) {self.hover = hover}
    fn get_selected(&self) -> bool {self.selected}
    fn set_selected(&mut self, selected:bool) {self.selected = selected}

    fn draw(&mut self, _args:piston::RenderArgs, pos:Vector2, depth:f64, list:&mut Vec<Box<dyn Renderable>>) {
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
            self.user.username.clone(),
            font.clone()
        )));

        // status
        if let Some(_action) = &self.user.action {
            
        }
        if let Some(action_text) = &self.user.action_text {
            list.push(Box::new(Text::new(
                Color::BLACK,
                depth - 1.0,
                pos + USERNAME_OFFSET + Vector2::new(0.0, 20.0),
                20,
                action_text.clone(),
                font.clone()
            )));
        }
    }

}
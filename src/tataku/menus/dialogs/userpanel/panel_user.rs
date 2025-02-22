use crate::prelude::*;

pub const USER_ITEM_SIZE:Vector2 = Vector2::new(300.0, 100.0);
pub const USERNAME_OFFSET:Vector2 = Vector2::new(5.0, 5.0);

#[derive(Clone, ScrollableGettersSetters)]
pub struct PanelUser {
    pos: Vector2,
    size: Vector2,
    hover: bool,
    selected: bool,
    tag: String,

    pub user: OnlineUser
}
impl PanelUser {
    pub fn new(user: OnlineUser) -> Self {
        Self {
            tag: user.username.clone(),
            size: USER_ITEM_SIZE,
            user,
            hover: false,
            selected: false,
            pos: Vector2::ZERO
        }
    }
}
impl Default for PanelUser {
    fn default() -> Self {
        Self { 
            pos: Vector2::ZERO, 
            size: USER_ITEM_SIZE,
            tag: String::new(),
            hover: Default::default(), 
            selected: Default::default(), 

            user: Default::default()
        }
    }
}

impl ScrollableItem for PanelUser {
    fn draw(&mut self, _args:piston::RenderArgs, pos:Vector2, depth:f64, list: &mut RenderableCollection) {
        let font = get_font();
        let pos = self.pos + pos;

        // bounding box
        let c = Color::new(0.5, 0.5, 0.5, 0.75);
        list.push(Rectangle::new(
            c,
            depth,
            pos,
            USER_ITEM_SIZE,
            Some(Border::new(if self.hover {Color::RED} else {Color::new(0.75, 0.75, 0.75, 0.75)}, 2.0))
        ));

        // username
        list.push(Text::new(
            Color::WHITE,
            depth - 1.0,
            pos + USERNAME_OFFSET,
            20,
            self.user.username.clone(),
            font.clone()
        ));

        // status
        if let Some(_action) = &self.user.action {
            
        }
        if let Some(action_text) = &self.user.action_text {
            list.push(Text::new(
                Color::BLACK,
                depth - 1.0,
                pos + USERNAME_OFFSET + Vector2::new(0.0, 20.0),
                20,
                action_text.clone(),
                font.clone()
            ));
        }
    }

}
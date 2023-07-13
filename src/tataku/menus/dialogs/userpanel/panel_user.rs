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
    fn get_keywords(&self) -> Vec<String> { self.user.username.split(" ").map(|a|a.to_lowercase().to_owned()).collect() }

    fn draw(&mut self, pos:Vector2, list: &mut RenderableCollection) {
        let font = get_font();
        let pos = self.pos + pos;

        // bounding box
        list.push(Rectangle::new(
            pos,
            USER_ITEM_SIZE,
            Color::new(0.5, 0.5, 0.5, 0.75),
            Some(Border::new(if self.hover {Color::RED} else {Color::new(0.75, 0.75, 0.75, 0.75)}, 2.0))
        ));

        // username
        list.push(Text::new(
            pos + USERNAME_OFFSET,
            20.0,
            self.user.username.clone(),
            Color::WHITE,
            font.clone()
        ));

        // status
        if let Some(_action) = &self.user.action {
            
        }
        if let Some(action_text) = &self.user.action_text {
            list.push(Text::new(
                pos + USERNAME_OFFSET + Vector2::new(0.0, 20.0),
                20.0,
                action_text.clone(),
                Color::BLACK,
                font.clone()
            ));
        }
    }
}
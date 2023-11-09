use crate::prelude::*;

pub const USER_ITEM_SIZE:Vector2 = Vector2::new(300.0, 100.0);
pub const USERNAME_OFFSET:Vector2 = Vector2::new(5.0, 5.0);

#[derive(Clone, ScrollableGettersSetters)]
pub struct PanelUser {
    pos: Vector2,
    size: Vector2,
    style: Style,
    node: Node,

    hover: bool,
    selected: bool,
    tag: String,

    pub user: OnlineUser
}
impl PanelUser {
    pub fn new(user: OnlineUser, layout_manager: &LayoutManager) -> Self {
        let style = Style {
            size: Size {
                width: Dimension::Percent(0.2),
                height: Dimension::Percent(0.2),
            },
            ..Default::default()
        };
        let node = layout_manager.create_node(&style);


        Self {
            pos: Vector2::ZERO,
            size: USER_ITEM_SIZE,
            style, 
            node,

            tag: user.username.clone(),
            user,
            hover: false,
            selected: false,
        }
    }
}
// impl Default for PanelUser {
//     fn default() -> Self {
//         Self { 
//             pos: Vector2::ZERO, 
//             size: USER_ITEM_SIZE,


//             tag: String::new(),
//             hover: Default::default(), 
//             selected: Default::default(), 

//             user: Default::default()
//         }
//     }
// }

impl ScrollableItem for PanelUser {
    fn get_style(&self) -> Style { self.style.clone() }
    fn apply_layout(&mut self, layout: &LayoutManager, parent_pos: Vector2) {
        let layout = layout.get_layout(self.node);
        self.pos = layout.location.into();
        self.pos += parent_pos;
        self.size = layout.size.into();
    }
    
    fn get_keywords(&self) -> Vec<String> { self.user.username.split(" ").map(|a|a.to_lowercase().to_owned()).collect() }

    fn draw(&mut self, pos:Vector2, list: &mut RenderableCollection) {
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
            Font::Main
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
                Font::Main
            ));
        }
    }
}
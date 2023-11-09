use crate::prelude::*;

#[derive(ScrollableGettersSetters)]
pub struct LobbySlotDisplay {
    pos: Vector2,
    size: Vector2,
    style: Style,
    node: Node,

    hover: bool,
    tag: String,
    ui_scale: Vector2,
    base_size: Vector2,

    items: ScrollableArea,
}
impl LobbySlotDisplay {
    pub fn new(slot: u8, state_receiver: AsyncReceiver<(LobbySlot, bool)>, player_receiver: AsyncReceiver<Option<LobbyPlayerInfo>>, layout_manager: &LayoutManager) -> Self {
        // let size = Vector2::new(width, 50.0);
        let style = Style {
            size: LayoutManager::full_width(),
            display: taffy::style::Display::Flex,
            flex_direction: taffy::style::FlexDirection::Row,
            ..Default::default()
        };

        let node = layout_manager.create_node(&style);
        let (pos, size) = LayoutManager::get_pos_size(&style);

        
        let mut items = ScrollableArea::new(style.clone(), ListMode::Grid(GridSettings::new(Vector2::new(5.0, 0.0), HorizontalAlign::Left)), layout_manager);
        let layout_manager = items.layout_manager.clone();
        items.add_item(Box::new(LobbySlotStatus::new( slot, state_receiver, &layout_manager)));
        items.add_item(Box::new(LobbySlotUser::new(slot, player_receiver, &layout_manager)));

        Self {
            pos,
            size,
            style,
            node,

            base_size: size,
            hover: false,
            tag: String::new(),
            ui_scale: Vector2::ONE,

            items
        }
    }
}

impl ScrollableItem for LobbySlotDisplay {
    fn get_style(&self) -> Style { self.style.clone() }
    fn apply_layout(&mut self, layout: &LayoutManager, parent_pos: Vector2) {
        let layout = layout.get_layout(self.node);
        self.pos = layout.location.into();
        self.pos += parent_pos;
        self.size = layout.size.into();
    }

    fn ui_scale_changed(&mut self, scale: Vector2) {
        self.ui_scale = scale;
        self.size = self.base_size * scale;
    }

    fn on_click_tagged(&mut self, pos:Vector2, button:MouseButton, mods:KeyModifiers) -> Option<String> {
        self.items.on_click_tagged(pos, button, mods)
    }

    fn on_mouse_move(&mut self, p:Vector2) {
        self.check_hover(p);
        self.items.on_mouse_move(p);
    }

    fn update(&mut self) {
        self.items.update();
        if self.pos != self.items.get_pos() {
            self.items.set_pos(self.pos);
        }
    }

    fn draw(&mut self, pos_offset:Vector2, list: &mut RenderableCollection) {
        // background and border
        list.push(Rectangle::new(self.pos + pos_offset, self.size, Color::GRAY.alpha(0.8), Some(Border::new(if self.hover {Color::RED} else {Color::BLACK}, 2.0))).shape(Shape::Round(4.0)));

        self.items.draw(pos_offset, list);
    }
}

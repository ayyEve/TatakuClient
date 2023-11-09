use crate::prelude::*;

pub struct ModDialog {
    should_close: bool,
    scroll: ScrollableArea,
    layout_manager: LayoutManager,

    // window_size: Arc<WindowSize>,
    bounds: Bounds,

    selected_index: usize,
}
impl ModDialog {
    pub async fn new(groups: Vec<GameplayModGroup>) -> Self {
        let mut layout_manager = LayoutManager::new();
        let mut new_groups = default_mod_groups();

        // see if any groups are named the same and merge them
        'outer: for g in groups {
            for n in new_groups.iter_mut() {
                if n.name == g.name {
                    n.mods.extend(g.mods.into_iter());
                    continue 'outer;
                }
            }

            new_groups.push(g);
        }

        // create the scrollable and add the mod buttons to it
        // let window_size = WindowSize::get();
        let mut scroll = ScrollableArea::new(Style {
            size: Size {
                width: Dimension::Percent(1.0),
                height: Dimension::Percent(1.0),
            },
            display: taffy::style::Display::Flex,
            flex_direction: taffy::style::FlexDirection::Column,
            ..Default::default()
        }, ListMode::VerticalList, &layout_manager);
        
        let layout_manager2 = scroll.layout_manager.clone();


        let section_style = Style {
            size: Size {
                width: Dimension::Percent(0.8),
                height: Dimension::Auto,
            },
            ..Default::default()
        };
        let btn_style = Style {
            size: Size {
                width: Dimension::Percent(0.8),
                height: Dimension::Auto,
            },
            ..Default::default()
        };
        let manager = ModManager::get();
        for group in new_groups {
            scroll.add_item(Box::new(MenuSection::new(section_style.clone(), &group.name, Color::WHITE, &layout_manager2, Font::Main)));
            
            for m in group.mods {
                scroll.add_item(Box::new(ModButton::new(btn_style.clone(), m, &manager, &layout_manager2)));
            }
        }

        let window_size = WindowSize::get().0;
        layout_manager.apply_layout(window_size);

        Self {
            scroll,
            layout_manager,
            bounds: Bounds::new(Vector2::ZERO, window_size),
            // window_size,
            should_close: false,
            selected_index: 0,
        }
    }

    fn increment_index(&mut self) {
        if self.scroll.items.len() == 0 { return } // should never happen but just to be safe

        let old = self.selected_index;
        self.selected_index = (self.selected_index + 1) % self.scroll.items.len();

        self.scroll.items.get_mut(old).unwrap().set_selected(false);
        self.scroll.items.get_mut(self.selected_index).unwrap().set_selected(true);
    }
    fn deincrement_index(&mut self) {
        if self.scroll.items.len() == 0 { return } // should never happen but just to be safe

        let old = self.selected_index;
        self.selected_index = if self.selected_index == 0 { self.scroll.items.len() - 1 } else { self.selected_index - 1 };

        self.scroll.items.get_mut(old).unwrap().set_selected(false);
        self.scroll.items.get_mut(self.selected_index).unwrap().set_selected(true);
    }
    fn toggle_current(&mut self) {
        if let Some(i) = self.scroll.items.get_mut(self.selected_index) {
            i.on_key_press(Key::Space, Default::default());
        }
    }
}

#[async_trait]
impl Dialog<Game> for ModDialog {
    fn name(&self) -> &'static str { "mod_menu" }
    fn should_close(&self) -> bool { self.should_close }
    fn get_bounds(&self) -> Bounds { self.bounds }
    async fn force_close(&mut self) { self.should_close = true; }

    async fn update(&mut self, _g: &mut Game) {
        self.scroll.update();
    }
    
    async fn draw(&mut self, offset: Vector2, list: &mut RenderableCollection) {
        self.draw_background(Color::BLACK, offset, list);
        self.scroll.draw(offset, list);
    }

    async fn on_key_press(&mut self, key:Key, _mods:&KeyModifiers, _g:&mut Game) -> bool {
        match key {
            Key::Up => {
                self.deincrement_index();
                true
            }
            Key::Down => {
                self.increment_index();
                true
            }
            Key::Space | Key::Return => {
                self.toggle_current();
                true
            }

            _ => false
        }
    }

    async fn on_mouse_move(&mut self, pos:Vector2, _g:&mut Game) {
        self.scroll.on_mouse_move(pos);
    }

    async fn on_mouse_scroll(&mut self, delta:f32, _g:&mut Game) -> bool {
        self.scroll.on_scroll(delta);
        false
    }

    async fn on_mouse_down(&mut self, pos:Vector2, button:MouseButton, mods:&KeyModifiers, _g:&mut Game) -> bool {
        self.scroll.on_click(pos, button, *mods);
        true
    }

    async fn on_mouse_up(&mut self, pos:Vector2, button:MouseButton, _mods:&KeyModifiers, _g:&mut Game) -> bool {
        self.scroll.on_click_release(pos, button);
        true
    }

    fn container_size_changed(&mut self, size: Vector2) {
        self.bounds.size = size;
        self.layout_manager.apply_layout(size);
        self.scroll.apply_layout(&self.layout_manager, Vector2::ZERO);
    }


    
    async fn on_controller_press(&mut self, _controller: &GamepadInfo, button: ControllerButton) -> bool {
        match button {
            ControllerButton::South => self.toggle_current(),
            ControllerButton::DPadUp => self.deincrement_index(),
            ControllerButton::DPadDown => self.increment_index(),
            ControllerButton::East | ControllerButton::Start => self.should_close = true,

            _ => {}
        }
        true
    }
    async fn on_controller_release(&mut self, _controller: &GamepadInfo, _button: ControllerButton) -> bool {
        true
    }
}

#[derive(ScrollableGettersSetters)]
struct ModButton {
    size: Vector2,
    pos: Vector2,
    style: Style,
    node: Node,

    hover: bool,
    selected: bool,

    checkbox: Checkbox,
    text: SimpleText,

    gameplay_mod: GameplayMod,
    mod_name: String,
    enabled: bool,

    mods: ModManagerHelper
}
impl ModButton {
    fn new(style: Style, gameplay_mod: GameplayMod, current_mods: &ModManager, layout_manager: &LayoutManager) -> Self {
        let enabled = current_mods.has_mod(gameplay_mod.name);
        let mod_name = gameplay_mod.display_name.to_owned();

        let (pos, size) = LayoutManager::get_pos_size(&style);
        let node = layout_manager.create_node(&style);

        // create a node to align the checkbox and description text
        let subnode = layout_manager.clone().with_parent(node);
        subnode.set_style(Style {
            size: Size {
                width: Dimension::Percent(1.0),
                height: Dimension::Auto,
            },
            display: taffy::style::Display::Flex,
            flex_direction: taffy::style::FlexDirection::Row,

            ..Default::default()
        });

        let checkbox = Checkbox::new(Style {
            ..Default::default()
        }, &mod_name, enabled, &subnode, Font::Main);

        let text = SimpleText::new(Style::default(), 30.0, gameplay_mod.description, &subnode);

        Self {
            size,
            pos, 
            style,
            node,

            checkbox,
            text,

            hover: false,
            selected: false,

            gameplay_mod,
            mod_name,

            enabled,
            mods: ModManagerHelper::new()
        }
    }

    fn toggle(&self) {
        let name = self.gameplay_mod.name;
        let removes:HashSet<String> = self.gameplay_mod.removes.iter().map(|m|(*m).to_owned()).collect();
        tokio::spawn(async move {
            let mut manager = ModManager::get_mut();
            manager.toggle_mod(name);
            manager.mods.retain(|m|!removes.contains(m));
        });
    }
}
impl ScrollableItem for ModButton {
    fn get_style(&self) -> Style { self.style.clone() }
    fn apply_layout(&mut self, layout_manager: &LayoutManager, parent_pos: Vector2) {
        let layout = layout_manager.get_layout(self.node);
        self.pos = layout.location.into();
        self.pos += parent_pos;
        self.size = layout.size.into();

        self.checkbox.apply_layout(layout_manager, self.pos);
        self.text.apply_layout(layout_manager, self.pos);
    }

    fn update(&mut self) {
        if self.mods.update() {
            self.enabled = self.mods.has_mod(self.gameplay_mod);
        }
        self.checkbox.set_hover(self.hover);
        self.checkbox.set_selected(self.selected);
    }

    fn draw(&mut self, pos_offset:Vector2, list: &mut RenderableCollection) {
        let pos_offset = self.pos + pos_offset;

        // let font_size = 30.0;
        // let desc_pos = pos_offset + cb_size.x_portion() + Vector2::new(10.0, (cb_size.y - font_size) / 2.0);
        // let desc_text = Text::new(
        //     desc_pos, 
        //     font_size, 
        //     self.gameplay_mod.description, 
        //     Color::WHITE, 
        //     Font::Main
        // );

        self.checkbox.draw(pos_offset, list);
        self.text.draw(pos_offset, list);
        // list.push(desc_text);
    }

    fn on_key_press(&mut self, key:Key, _mods:KeyModifiers) -> bool {
        if key == Key::Space {
            self.toggle()
        }

        self.get_selected()
    }

    fn on_click(&mut self, _pos:Vector2, _btn: MouseButton, _mods:KeyModifiers) -> bool {
        if self.hover {
            self.toggle();
        }

        self.hover
    }
}

use crate::prelude::*;

pub struct ModDialog {
    num: usize,
    should_close: bool,
    mod_groups: Vec<GameplayModGroup>,
}
impl ModDialog {
    pub async fn new(groups: Vec<GameplayModGroup>) -> Self {
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

        // // create the scrollable and add the mod buttons to it
        // let window_size = WindowSize::get();
        // let mut scroll = ScrollableArea::new(Vector2::with_y(20.0), window_size.0, ListMode::VerticalList);
        // let pos = Vector2::new(50.0, 0.0);

        // let manager = ModManager::get();
        // for group in new_groups {
        //     scroll.add_item(Box::new(MenuSection::new(pos, 50.0, &group.name, Color::WHITE, Font::Main)));
            
        //     for m in group.mods {
        //         scroll.add_item(Box::new(ModButton::new(pos, m, &manager)));
        //     }
        // }

        Self {
            num: 0,
            should_close: false,
            mod_groups: new_groups,
            // scroll,
            // window_size,
            // selected_index: 0
        }
    }

    fn increment_index(&mut self) {
        // if self.scroll.items.len() == 0 { return } // should never happen but just to be safe

        // let old = self.selected_index;
        // self.selected_index = (self.selected_index + 1) % self.scroll.items.len();

        // self.scroll.items.get_mut(old).unwrap().set_selected(false);
        // self.scroll.items.get_mut(self.selected_index).unwrap().set_selected(true);
    }
    fn deincrement_index(&mut self) {
        // if self.scroll.items.len() == 0 { return } // should never happen but just to be safe

        // let old = self.selected_index;
        // self.selected_index = if self.selected_index == 0 { self.scroll.items.len() - 1 } else { self.selected_index - 1 };

        // self.scroll.items.get_mut(old).unwrap().set_selected(false);
        // self.scroll.items.get_mut(self.selected_index).unwrap().set_selected(true);
    }
    // fn toggle_current(&mut self) {
    //     if let Some(i) = self.scroll.items.get_mut(self.selected_index) {
    //         i.on_key_press(Key::Space, Default::default());
    //     }
    // }

    fn toggle_mod(&self, m:GameplayMod) {
        let removes:HashSet<String> = m.removes.iter().map(|m|(*m).to_owned()).collect();

        let mut mods = ModManager::get_mut();
        mods.toggle_mod(m);
        mods.mods.retain(|m|!removes.contains(m));
    }
}

#[async_trait]
impl Dialog for ModDialog {
    fn name(&self) -> &'static str { "mod_menu" }
    fn get_num(&self) -> usize { self.num }
    fn set_num(&mut self, num: usize) { self.num = num }

    fn should_close(&self) -> bool { self.should_close }
    async fn force_close(&mut self) { self.should_close = true; }

    
    async fn handle_message(&mut self, message: Message, values: &mut ShuntingYardValues) {
        match message.tag {
            MessageTag::GameplayMod(m) => self.toggle_mod(m),

            _ => {}
        }
    }

    fn view(&self) -> IcedElement {
        use iced_elements::*;
        let mods = ModManager::get();
        let owner = MessageOwner::new_dialog(self);

        let mut items = Vec::new();
        for group in self.mod_groups.clone() {
            items.push(Text::new(group.name).width(Fill).into_element());
            items.push(Text::new("   ").width(Fill).into_element());

            for m in group.mods {
                items.push(row!(
                    Checkbox::new(m.name, mods.has_mod(m), move|_|Message::new(owner, m.clone(), MessageType::Custom(Arc::new(m)))).text_size(30.0).width(Fill),
                    Text::new(m.description).width(Fill).size(30.0);
                    width = Fill
                ))
            }
        }

        make_scrollable(items, "mods_list").into_element()
    }


    // async fn update(&mut self, _g: &mut Game) {
    //     self.scroll.update();
    // }
    
    // async fn draw(&mut self, offset: Vector2, list: &mut RenderableCollection) {
    //     self.draw_background(Color::BLACK, offset, list);
    //     self.scroll.draw(offset, list);
    // }

    // async fn on_key_press(&mut self, key:Key, _mods:&KeyModifiers, _g:&mut Game) -> bool {
    //     match key {
    //         Key::Up => {
    //             self.deincrement_index();
    //             true
    //         }
    //         Key::Down => {
    //             self.increment_index();
    //             true
    //         }
    //         Key::Space | Key::Return => {
    //             self.toggle_current();
    //             true
    //         }

    //         _ => false
    //     }
    // }

    // async fn on_mouse_move(&mut self, pos:Vector2, _g:&mut Game) {
    //     self.scroll.on_mouse_move(pos);
    // }

    // async fn on_mouse_down(&mut self, pos:Vector2, button:MouseButton, mods:&KeyModifiers, _g:&mut Game) -> bool {
    //     self.scroll.on_click(pos, button, *mods);
    //     true
    // }

    // async fn on_mouse_up(&mut self, pos:Vector2, button:MouseButton, _mods:&KeyModifiers, _g:&mut Game) -> bool {
    //     self.scroll.on_click_release(pos, button);
    //     true
    // }

}

#[derive(ScrollableGettersSetters)]
struct ModButton {
    size: Vector2,
    pos: Vector2,
    hover: bool,
    selected: bool,

    gameplay_mod: GameplayMod,
    mod_name: String,
    enabled: bool,

    mods: ModManagerHelper
}
impl ModButton {
    fn new(pos: Vector2, gameplay_mod: GameplayMod, current_mods: &ModManager) -> Self {
        let enabled = current_mods.has_mod(gameplay_mod.name);
        let mod_name = gameplay_mod.display_name.to_owned();

        Self {
            size: Vector2::new(500.0, 50.0),
            pos, 
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
    fn update(&mut self) {
        if self.mods.update() {
            self.enabled = self.mods.has_mod(self.gameplay_mod)
        }
    }

    fn draw(&mut self, pos_offset:Vector2, list: &mut RenderableCollection) {
        let pos_offset = self.pos + pos_offset;
        let cb_size = Vector2::new(200.0, 50.0);

        let mut checkbox = Checkbox::new(Vector2::ZERO, cb_size, &self.mod_name, self.enabled, Font::Main);
        checkbox.set_hover(self.hover);
        checkbox.set_selected(self.selected);

        let font_size = 30.0;
        let desc_pos = pos_offset + cb_size.x_portion() + Vector2::new(10.0, (cb_size.y - font_size) / 2.0);
        let desc_text = Text::new(
            desc_pos, 
            font_size, 
            self.gameplay_mod.description, 
            Color::WHITE, 
            Font::Main
        );

        checkbox.draw(pos_offset, list);
        list.push(desc_text);
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

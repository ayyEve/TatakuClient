use crate::prelude::*;

pub struct ModDialog {
    should_close: bool,
    scroll: ScrollableArea,

    window_size: Arc<WindowSize>,

    selected_index: usize
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

        // create the scrollable and add the mod buttons to it
        let window_size = WindowSize::get();
        let mut scroll = ScrollableArea::new(Vector2::with_y(20.0), window_size.0, ListMode::VerticalList);
        let pos = Vector2::new(50.0, 0.0);

        let font = get_font();
        let manager = ModManager::get();
        for group in new_groups {
            scroll.add_item(Box::new(MenuSection::new(pos, 50.0, &group.name, Color::WHITE, font.clone())));
            
            for m in group.mods {
                scroll.add_item(Box::new(ModButton::new(pos, m, &manager)));
            }
        }

        Self {
            should_close: false,
            scroll,
            window_size,
            selected_index: 0
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
    fn get_bounds(&self) -> Bounds { Bounds::new(Vector2::ZERO, self.window_size.0) }

    async fn update(&mut self, _g: &mut Game) {
        self.scroll.update();
    }
    
    async fn draw(&mut self, list: &mut RenderableCollection) {
        self.draw_background(Color::BLACK, list);
        self.scroll.draw(Vector2::ZERO, list);
    }

    async fn on_key_press(&mut self, key:Key, _mods:&KeyModifiers, _g:&mut Game) -> bool {
        match key {
            Key::Escape => {
                self.should_close = true;
                true
            }
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

    async fn window_size_changed(&mut self, window_size: Arc<WindowSize>) {
        self.window_size = window_size;
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
    hover: bool,
    selected: bool,

    gameplay_mod: Box<dyn GameplayMod>,
    mod_name: String,
    enabled: bool,

    mods: ModManagerHelper
}
impl ModButton {
    fn new(pos: Vector2, gameplay_mod: Box<dyn GameplayMod>, current_mods: &ModManager) -> Self {
        let enabled = current_mods.has_mod(gameplay_mod.name());
        let mod_name = gameplay_mod.display_name().to_owned();

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
        let name = self.gameplay_mod.name();
        let removes:HashSet<String> = self.gameplay_mod.removes().iter().map(|m|(*m).to_owned()).collect();
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
            self.enabled = self.mods.has_mod(self.gameplay_mod.name())
        }
    }

    fn draw(&mut self, pos_offset:Vector2, list: &mut RenderableCollection) {
        let pos_offset = self.pos + pos_offset;
        
        let font = get_font();
        let cb_size = Vector2::new(200.0, 50.0);

        let mut checkbox = Checkbox::new(Vector2::ZERO, cb_size, &self.mod_name, self.enabled, font.clone());
        checkbox.set_hover(self.hover);
        checkbox.set_selected(self.selected);

        let font_size = 30.0;
        let desc_pos = pos_offset + cb_size.x_portion() + Vector2::new(10.0, (cb_size.y - font_size) / 2.0);
        let desc_text = Text::new(
            desc_pos, 
            font_size, 
            self.gameplay_mod.description().to_owned(), 
            Color::WHITE, 
            font
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

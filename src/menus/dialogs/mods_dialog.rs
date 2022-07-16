use crate::prelude::*;

const MODS: &[&[&'static str]] = &[
    &["easy"], &["hardrock"],
    &["nofail"], &["autoplay"],
];

pub struct ModDialog {
    should_close: bool,

    buttons: Vec<ModButton>
}
impl ModDialog {
    pub async fn new() -> Self {

        let mut buttons = Vec::new();
        let mut current_pos = Vector2::new(100.0, 100.0);

        //TODO: properly implement rows
        for m in MODS.iter() {
            let b = ModButton::new(
                current_pos,
                m.iter().map(|a|a.to_owned().to_owned()).collect()
            ).await;
            current_pos += Vector2::x_only(200.0);
            buttons.push(b);
        }

        Self {
            should_close: false,
            buttons
        }
    }
}

#[async_trait]
impl Dialog<Game> for ModDialog {
    async fn window_size_changed(&mut self, window_size: Arc<WindowSize>) {
        // self.window_size = window_size;
        
    }


    fn name(&self) -> &'static str {"mod_menu"}
    fn should_close(&self) -> bool {self.should_close}
    fn get_bounds(&self) -> Rectangle { 
        Rectangle::bounds_only(Vector2::zero(), WindowSize::get().0) 
    }
    
    async fn draw(&mut self, args:&RenderArgs, depth: &f64, list: &mut Vec<Box<dyn Renderable>>) {
        self.draw_background(depth + 0.00000001, Color::BLACK, list);
        for b in self.buttons.iter_mut() {
            b.draw(*args, Vector2::zero(), *depth, list)
        }
    }

    async fn on_key_press(&mut self, key:&Key, _mods:&KeyModifiers, _g:&mut Game) -> bool {
        if key == &Key::Escape {
            self.should_close = true;
            return true;
        }

        false
    }


    async fn on_mouse_move(&mut self, pos:&Vector2, _g:&mut Game) {
        for b in self.buttons.iter_mut() {
            b.on_mouse_move(*pos)
        }
    }

    async fn on_mouse_scroll(&mut self, _delta:&f64, _g:&mut Game) -> bool {
        
        false
    }

    async fn on_mouse_down(&mut self, pos:&Vector2, button:&MouseButton, mods:&KeyModifiers, _g:&mut Game) -> bool {
        for b in self.buttons.iter_mut() {
            b.on_click(*pos, *button, *mods);
        }
        true
    }

    async fn on_mouse_up(&mut self, _pos:&Vector2, _button:&MouseButton, _mods:&KeyModifiers, _g:&mut Game) -> bool {false}
}

#[derive(ScrollableGettersSetters)]
struct ModButton {
    size: Vector2,
    pos: Vector2,
    hover: bool,

    mod_names: Vec<String>,
    mod_images: Vec<Option<Image>>,

    /// index of selected mod + 1
    /// 0 means none selected
    selected_mod: usize,
}
impl ModButton {
    async fn new(pos: Vector2, mod_names: Vec<String>) -> Self {

        let mut mod_images = Vec::new();
        for m in mod_names.iter() {
            mod_images.push(SkinManager::get_texture(format!("selection-mod-{}", m), true).await)
        }

        let mut selected_mod = 0;
        let mut manager = ModManager::get().await;
        for (i, name) in mod_names.iter().enumerate() {
            if *str_2_modval(name, &mut manager) {
                selected_mod = i + 1
            }
        }

        Self {
            size: Vector2::new(100.0, 100.0),
            pos, 
            mod_names,
            mod_images,
            selected_mod,
            hover: false
        }
    }

    fn apply_mod(&self) {
        let mut list = HashMap::new();
        for (i, name) in self.mod_names.iter().enumerate() {
            let use_val = i+1 == self.selected_mod && self.selected_mod > 0;
            trace!("set: {}, name:{}", use_val, name);
            list.insert(name.clone(), use_val);
        }

        tokio::spawn(async move {
            let mut manager = ModManager::get().await;
            
            for (name, use_val) in list {
                *str_2_modval(&name, &mut manager) = use_val;
            }
        });
    }
}
impl ScrollableItem for ModButton {
    fn draw(&mut self, _args:RenderArgs, pos_offset:Vector2, parent_depth:f64, list: &mut Vec<Box<dyn Renderable>>) {
        let is_selected = self.selected_mod > 0;
        let selected_mod = if is_selected {self.selected_mod - 1} else {0};

        // draw image
        if let Some(Some(mod_img)) = self.mod_images.get(selected_mod) {
            let mut img = mod_img.clone();
            // offset by size/2 because image origin is center (required for rotation)
            img.current_pos = self.pos + pos_offset + self.size/2.0;
            img.current_scale = self.size / img.tex_size();
            img.depth = parent_depth;
            if is_selected {
                img.current_scale *= 1.1;
                img.current_rotation = PI / 12.0;
            }
            list.push(Box::new(img));
        } else {
            // draw bounding box
            let mut rect = Rectangle::new(
                Color::GREEN, // TODO: customizable per-mod?
                parent_depth,
                self.pos + pos_offset,
                self.size,
                Some(Border::new(Color::BLACK, 2.0))
            );
            if is_selected {
                rect.current_scale *= 1.1;
                rect.current_rotation = PI / 3.0; // 60 degrees
            }

            // add text to lower third of bounding box
            let mut text = Text::new(
                Color::BLACK,
                parent_depth - 0.0000001,
                Vector2::zero(),
                32,
                self.mod_names[selected_mod].clone(),
                get_font()
            );
            text.center_text(Rectangle::bounds_only(
                self.pos + pos_offset + Vector2::new(0.0, self.size.y * (2.0/3.0)), 
                Vector2::new(self.size.x, self.size.y / 3.0)
            ))
        }

    }

    fn on_click(&mut self, _pos:Vector2, _btn: MouseButton, _mods:KeyModifiers) -> bool {
        if self.hover {
           self.selected_mod = (self.selected_mod + 1) % (self.mod_names.len() + 1);
           self.apply_mod()
        }

        self.hover
    }
}

// this is dumb but i dont care
fn str_2_modval<'a>(name: &String, manager: &'a mut ModManager) -> &'a mut bool {
    match &**name {
        "easy" => &mut manager.easy,
        "hardrock" => &mut manager.hard_rock,

        "nofail" => &mut manager.nofail,
        "autoplay" => &mut manager.autoplay,
        
        other => panic!("Unknown mod name: {}", other)
    }
}
use crate::prelude::*;

const MODS: &[&[&'static str]] = &[
    &["nofail"], &["autoplay"]
];

pub struct ModDialog {
    should_close: bool,

    buttons: Vec<ModButton>
}
impl ModDialog {
    pub fn new() -> Self {

        let mut buttons = Vec::new();
        let mut current_pos = Vector2::new(100.0, 100.0);

        //TODO: properly implement rows
        for m in MODS.iter() {
            let b = ModButton::new(
                current_pos,
                m.iter().map(|a|a.to_owned().to_owned()).collect()
            );
            current_pos += Vector2::x_only(200.0);
            buttons.push(b);
        }

        Self {
            should_close: false,
            buttons
        }
    }
}

impl Dialog<Game> for ModDialog {
    fn name(&self) -> &'static str {"mod_menu"}
    fn should_close(&self) -> bool {self.should_close}
    fn get_bounds(&self) -> Rectangle {Rectangle::bounds_only(Vector2::zero(), Settings::window_size())}
    
    fn draw(&mut self, args:&RenderArgs, depth: &f64, list: &mut Vec<Box<dyn Renderable>>) {
        self.draw_background(depth + 0.00000001, Color::BLACK, list);
        for b in self.buttons.iter_mut() {
            b.draw(*args, Vector2::zero(), *depth, list)
        }
    }

    fn on_key_press(&mut self, key:&Key, _mods:&KeyModifiers, _g:&mut Game) -> bool{
        if key == &Key::Escape {
            self.should_close = true;
            return true;
        }

        false
    }


    fn on_mouse_move(&mut self, pos:&Vector2, _g:&mut Game) {
        for b in self.buttons.iter_mut() {
            b.on_mouse_move(*pos)
        }
    }

    fn on_mouse_scroll(&mut self, _delta:&f64, _g:&mut Game) -> bool {
        
        false
    }

    fn on_mouse_down(&mut self, pos:&Vector2, button:&MouseButton, mods:&KeyModifiers, _g:&mut Game) -> bool {
        for b in self.buttons.iter_mut() {
            b.on_click(*pos, *button, *mods);
        }
        true
    }

    fn on_mouse_up(&mut self, _pos:&Vector2, _button:&MouseButton, _mods:&KeyModifiers, _g:&mut Game) -> bool {false}
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
    fn new(pos: Vector2, mod_names: Vec<String>) -> Self {
        let mut mod_images: Vec<Option<Image>> = mod_names.iter().map(|m|SKIN_MANAGER.write().get_texture(format!("selection-mod-{}", m), true)).collect();

        Self {
            size: Vector2::new(100.0, 100.0),
            pos, 
            mod_names,
            mod_images,
            selected_mod: 0,
            hover: false
        }
    }

    fn apply_mod(&self, manager: &mut ModManager) {
        for (i, name) in self.mod_names.iter().enumerate() {
            let use_val = i+1 == self.selected_mod && self.selected_mod > 0;
            println!("set: {}, name:{}", use_val, name);
            
            match &**name {
                "nofail" => manager.nofail = use_val,
                "autoplay" => manager.autoplay = use_val,
                
                other => println!("unknown mod name: {}", other)
            }
        }

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
           self.apply_mod(&mut ModManager::get())
        }

        self.hover
    }
}
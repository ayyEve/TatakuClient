use crate::prelude::*;

// TODO: change this to skin meta
lazy_static::lazy_static! {
    static ref SKINS:Vec<String> = {
        let mut list = Vec::new();
        for f in std::fs::read_dir(SKIN_FOLDER).unwrap() {
            list.push(f.unwrap().file_name().to_string_lossy().to_string())
        }
        list
    };
}

pub struct SkinSelect {
    should_close: bool,
    dropdown: Dropdown<SkinDropdownable>,
    current_skin: String
}
impl SkinSelect {
    pub fn new() -> Self {
        let current_skin = SKIN_MANAGER.read().current_skin();
        Self {
            dropdown: Dropdown::new(
                Vector2::new(300.0, 200.0),
                500.0,
                20,
                "Skin:",
                Some(SkinDropdownable::Skin("default".to_owned())),
                get_font("")
            ),
            current_skin,
            should_close: false,
        }
    }

    fn check_skin_change(&mut self) {
        let selected = self.dropdown.get_value().downcast::<Option<SkinDropdownable>>();
        if let Ok(s) = selected {
            if let Some(SkinDropdownable::Skin(s)) = *s {
                if s == self.current_skin {return}

                println!("skin changing to {}", s);
                self.current_skin = s.clone();
                SKIN_MANAGER.write().change_skin(s);
            }
        }
    }
}
impl Dialog<Game> for SkinSelect {
    fn name(&self) -> &'static str {"skin_select"}
    fn should_close(&self) -> bool {self.should_close}
    fn get_bounds(&self) -> Rectangle {Rectangle::bounds_only(Vector2::zero(), Settings::window_size())}
    
    fn draw(&mut self, args:&RenderArgs, depth: &f64, list: &mut Vec<Box<dyn Renderable>>) {
        self.draw_background(*depth, Color::PINK_LEMONADE, list);
        self.dropdown.draw(*args, Vector2::zero(), *depth, list)
    }

    fn update(&mut self, _g:&mut Game) {
        self.dropdown.update()
    }

    fn on_mouse_move(&mut self, p:&Vector2, _g:&mut Game) {
        self.dropdown.on_mouse_move(*p)
    }

    fn on_mouse_scroll(&mut self, delta:&f64, _g:&mut Game) -> bool {
        self.dropdown.on_scroll(*delta);
        true
    }

    fn on_mouse_down(&mut self, pos:&Vector2, button:&MouseButton, mods:&KeyModifiers, _g:&mut Game) -> bool {
        self.dropdown.on_click(*pos, *button, *mods);
        self.check_skin_change();
        true
    }
    fn on_mouse_up(&mut self, pos:&Vector2, button:&MouseButton, _mods:&KeyModifiers, _g:&mut Game) -> bool {
        self.dropdown.on_click_release(*pos, *button);
        true
    }

    fn on_text(&mut self, _text:&String) -> bool {
        true
    }

    fn on_key_press(&mut self, key:&Key, _mods:&KeyModifiers, _g:&mut Game) -> bool {
        if key == &Key::Escape {self.should_close = true}
        
        true
    }
    fn on_key_release(&mut self, _key:&Key, _mods:&KeyModifiers, _g:&mut Game) -> bool {
        true
    }
}



#[derive(Clone)]
pub enum SkinDropdownable {
    Skin(String)
}
impl Dropdownable for SkinDropdownable {
    fn variants() -> Vec<Self> {
        SKINS.iter().map(|s|Self::Skin(s.clone())).collect()
    }

    fn display_text(&self) -> String {
        let Self::Skin(s) = self;
        s.clone()
    }

    fn from_string(s:String) -> Self {
        Self::Skin(s)
    }
}



const CHECK_DURATION: f32 = 1000.0;
pub struct SkinChangeHelper {
    current_skin: String,

    interval: Instant
}
impl SkinChangeHelper {
    pub fn new() -> Self {
        let current_skin = SKIN_MANAGER.read().current_skin();
        Self {
            current_skin,
            interval: Instant::now()
        }
    }
    pub fn check(&mut self) -> bool {
        let mut changed = false;

        if self.interval.elapsed().as_secs_f32() * 1000.0 > CHECK_DURATION {
            let current_skin = SKIN_MANAGER.read().current_skin();
            if self.current_skin != current_skin {
                changed = true;
                self.current_skin = current_skin;
                self.interval = Instant::now();
            }
        }
        changed
    }
}
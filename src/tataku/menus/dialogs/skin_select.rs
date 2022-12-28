use crate::prelude::*;

// TODO: change this to skin meta
lazy_static::lazy_static! {
    static ref SKINS:Vec<String> = {
        let mut list = vec!["None".to_owned()];
        for f in std::fs::read_dir(SKIN_FOLDER).unwrap() {
            list.push(f.unwrap().file_name().to_string_lossy().to_string())
        }
        list
    };
}

pub struct SkinSelect {
    should_close: bool,
    dropdown: Dropdown<SkinDropdownable, Font2, Text>,
    current_skin: String
}
impl SkinSelect {
    pub async fn new() -> Self {
        let current_skin = get_settings!().current_skin.clone();
        Self {
            dropdown: Dropdown::new(
                Vector2::new(300.0, 200.0),
                500.0,
                FontSize::new(20.0).unwrap(),
                "Skin",
                Some(SkinDropdownable::Skin(current_skin.clone())),
                get_font()
            ),
            current_skin,
            should_close: false,
        }
    }

    async fn check_skin_change(&mut self) {
        let selected = self.dropdown.get_value().downcast::<Option<SkinDropdownable>>();
        if let Ok(s) = selected {
            if let Some(SkinDropdownable::Skin(s)) = *s {
                if s == self.current_skin {return}

                trace!("skin changing to {}", s);
                self.current_skin = s.clone();
                tokio::spawn(async move {
                    SkinManager::change_skin(s, true).await;
                });
            }
        }
    }
}
#[async_trait]
impl Dialog<Game> for SkinSelect {
    fn name(&self) -> &'static str {"skin_select"}
    fn should_close(&self) -> bool {self.should_close}
    fn get_bounds(&self) -> Rectangle {
        Rectangle::bounds_only(Vector2::ZERO, WindowSize::get().0)
    }
    
    async fn draw(&mut self, args:&RenderArgs, depth: &f64, list: &mut RenderableCollection) {
        self.draw_background(*depth, Color::WHITE, list);
        self.dropdown.draw(*args, Vector2::ZERO, *depth, list)
    }

    async fn update(&mut self, _g:&mut Game) {
        self.dropdown.update()
    }

    async fn on_mouse_move(&mut self, p:&Vector2, _g:&mut Game) {
        self.dropdown.on_mouse_move(*p)
    }

    async fn on_mouse_scroll(&mut self, delta:&f64, _g:&mut Game) -> bool {
        self.dropdown.on_scroll(*delta);
        true
    }

    async fn on_mouse_down(&mut self, pos:&Vector2, button:&MouseButton, mods:&KeyModifiers, _g:&mut Game) -> bool {
        self.dropdown.on_click(*pos, *button, *mods);
        self.check_skin_change().await;
        true
    }
    async fn on_mouse_up(&mut self, pos:&Vector2, button:&MouseButton, _mods:&KeyModifiers, _g:&mut Game) -> bool {
        self.dropdown.on_click_release(*pos, *button);
        true
    }

    async fn on_text(&mut self, _text:&String) -> bool {
        true
    }

    async fn on_key_press(&mut self, key:&Key, _mods:&KeyModifiers, _g:&mut Game) -> bool {
        if key == &Key::Escape {self.should_close = true}
        
        true
    }
    async fn on_key_release(&mut self, _key:&Key, _mods:&KeyModifiers, _g:&mut Game) -> bool {
        true
    }

    
    async fn window_size_changed(&mut self, _window_size: Arc<WindowSize>) {
        
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



pub struct SkinChangeHelper {
    current_skin: String,
}
impl SkinChangeHelper {
    pub fn new_empty() -> Self {
        Self { current_skin: String::new() }
    }
    pub async fn new() -> Self {
        let current_skin = get_settings!().current_skin.clone();
        Self {
            current_skin,
        }
    }
    pub async fn check(&mut self) -> bool {
        let mut changed = false;

        // let skin_manager = SKIN_MANAGER.read();
        let current_skin = &get_settings!().current_skin;
        if &self.current_skin != current_skin {
            changed = true;
            self.current_skin = current_skin.clone();
            // println!("skin changed");
        }
        changed
    }
}

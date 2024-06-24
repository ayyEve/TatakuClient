use crate::prelude::*;

const DEFAULT_SKIN:&str = "default";

lazy_static::lazy_static! {
    // static ref SKIN_MANAGER: Arc<AsyncRwLock<SkinManager>> = Arc::new(AsyncRwLock::new(SkinManager::new()));
    
    // TODO: change this to skin meta
    pub static ref AVAILABLE_SKINS:Arc<RwLock<Vec<String>>> = {
        let mut list = vec!["None".to_owned()];
        if let Ok(folder) = std::fs::read_dir(SKINS_FOLDER) {
            for f in folder.filter_map(|f|f.ok()) {
                list.push(f.file_name().to_string_lossy().to_string())
            }
        }
        Arc::new(RwLock::new(list))
    };
}

/// path to a texture file
#[inline]
fn get_tex_path(tex_name:&String, skin_name:&String) -> String {
    format!("{SKINS_FOLDER}/{skin_name}/{tex_name}.png")
}

pub struct SkinManager {
    skin_name: String,
    current_skin_config: Arc<SkinSettings>,

    texture_cache: HashMap<String, Option<Image>>,
}

// static
impl SkinManager {
    pub fn new(settings: &Settings) -> Self {
        let current_skin = settings.current_skin.clone();
        let current_skin_config = Arc::new(SkinSettings::from_file(format!("{SKINS_FOLDER}/{current_skin}/skin.ini")).unwrap_or_default());
        
        Self {
            skin_name: current_skin,
            current_skin_config,
            texture_cache: HashMap::new(),
        }
    }


    pub async fn get_texture<N: AsRef<str> + Send + Sync>(&mut self, name:N, allow_default:bool) -> Option<Image> {
        self.get_texture_grayscale(name, allow_default, false).await
    }

    pub async fn get_texture_grayscale<N: AsRef<str> + Send + Sync>(&mut self, name:N, allow_default:bool, grayscale: bool) -> Option<Image> {
        if let Some(t) = self.texture_cache.get(name.as_ref()) {
            t.clone()
        } else {
            self.load_texture_grayscale(
                name.as_ref(), 
                allow_default, 
                grayscale
            ).await
        }
    }

    
    pub fn skin(&self) -> &Arc<SkinSettings> {
        // SKIN_MANAGER.read().await.current_skin_config.clone()
        &self.current_skin_config
    }

    pub async fn change_skin(&mut self, new_skin:String) {
        if self.skin_name == new_skin { return }
        self.skin_name = new_skin.clone();
        self.current_skin_config = Arc::new(SkinSettings::from_file(format!("{SKINS_FOLDER}/{new_skin}/skin.ini")).unwrap_or_default());

        // free up the last skin's images in the atlas for reuse
        std::mem::take(&mut self.texture_cache).into_iter().filter_map(|(_, i)| i.map(|i| i.tex)).for_each(GameWindow::free_texture);
    }


    pub fn refresh_skins() {
        let mut list = vec!["None".to_owned()];
        for f in std::fs::read_dir(SKINS_FOLDER).unwrap() {
            list.push(f.unwrap().file_name().to_string_lossy().to_string())
        }

        *AVAILABLE_SKINS.write() = list
    }

    async fn load_texture_grayscale(&mut self, name:&str, allow_default:bool, grayscale: bool) -> Option<Image> {
        let name = name.to_owned();
        if let Some(tex) = self.texture_cache.get(&name).cloned() { return tex }

        if !self.texture_cache.contains_key(&name) {
            self.texture_cache.insert(name.clone(), None);

            let mut skin_names = vec![self.skin_name.clone()];
            if allow_default { skin_names.push(DEFAULT_SKIN.to_owned()) }

            for skin_name in skin_names {
                for (tex_name, scale) in vec![(name.clone() + "@2x", Vector2::ONE / 2.0), (name.clone(), Vector2::ONE)] {
                    let tex_path = get_tex_path(&tex_name, &skin_name);

                    if let Some(maybe_img) = Self::load_image_inner(&tex_path, grayscale, scale).await {
                        trace!("loaded tex {tex_path}");
                        self.texture_cache.insert(name.clone(), Some(maybe_img.clone()));
                        return Some(maybe_img)
                    }
                }
            }
        }

        None
    }

    pub async fn get_texture_noskin(&mut self, name:&str, grayscale: bool) -> Option<Image> {
        let name = name.to_owned();
        if let Some(tex) = self.texture_cache.get(&name).cloned() { return tex }

        self.texture_cache.insert(name.clone(), None);

        for (tex_name, scale) in vec![(name.clone() + "@2x", Vector2::ONE / 2.0), (name.clone(), Vector2::ONE)] {
            if let Some(maybe_img) = Self::load_image_inner(&tex_name, grayscale, scale).await {
                trace!("loaded tex {tex_name}");
                self.texture_cache.insert(name.clone(), Some(maybe_img.clone()));
                return Some(maybe_img);
            }
        }

        None
    }

    /// load an image file to an image struct
    /// non-main thread safe
    async fn load_image_inner(path: impl AsRef<str> + Send + Sync, use_grayscale: bool, base_scale: Vector2) -> Option<Image> {
        let path2 = path.as_ref().to_owned();

        let Ok(buf) = Io::read_file_async(&path2).await else { return None };

        match image::load_from_memory(&buf) {
            Ok(img) => {
                let mut img = img.into_rgba8();

                if use_grayscale {
                    for i in img.pixels_mut() {
                        let [r, g, b, _a] = &mut i.0;

                        let rf = *r as f32 / 255.0;
                        let gf = *g as f32 / 255.0;
                        let bf = *b as f32 / 255.0;

                        let gray = 0.299 * rf + 0.587 * gf + 0.114 * bf;

                        *r = (gray * 255.0) as u8;
                        *g = (gray * 255.0) as u8;
                        *b = (gray * 255.0) as u8;
                    }
                }

                let tex = GameWindow::load_texture_data(img).await.expect("no atlas");
                Some(Image::new(Vector2::ZERO, tex, base_scale))
            }
            Err(e) => {
                NotificationManager::add_error_notification(format!("Error loading image: {}", path.as_ref()), e).await;
                None
            }
        }
    }
}


pub struct SkinDropdownable;
impl Dropdownable2 for SkinDropdownable {
    type T = String;
    fn variants() -> Vec<String> {
        AVAILABLE_SKINS.read().clone() //.iter().map(|s|Self::Skin(s.clone())).collect()
    }
}

// #[derive(Clone)]
// pub enum SkinDropdownable {
//     Skin(String)
// }
// impl Dropdownable for SkinDropdownable {
//     fn variants() -> Vec<Self> {
//         AVAILABLE_SKINS.read().iter().map(|s|Self::Skin(s.clone())).collect()
//     }

//     fn display_text(&self) -> String {
//         let Self::Skin(s) = self;
//         s.clone()
//     }

//     fn from_string(s:String) -> Self {
//         Self::Skin(s)
//     }
// }
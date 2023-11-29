use crate::prelude::*;

const DEFAULT_SKIN:&str = "default";

lazy_static::lazy_static! {
    static ref SKIN_MANAGER: Arc<AsyncRwLock<SkinManager>> = Arc::new(AsyncRwLock::new(SkinManager::new()));
    
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
    last_skin: String,
    current_skin_config: Arc<SkinSettings>,

    texture_cache: HashMap<String, Option<Image>>,
    // audio_cache: HashMap<String, Option<Sound>>,
    settings: SettingsHelper
}

// static
impl SkinManager {
    pub fn new() -> Self {
        let settings = Settings::get();
        
        let current_skin = settings.current_skin.clone();
        let current_skin_config = Arc::new(SkinSettings::from_file(format!("{SKINS_FOLDER}/{current_skin}/skin.ini")).unwrap_or_default());
        GlobalValueManager::update(Arc::new(CurrentSkin(current_skin_config.clone())));
        
        Self {
            last_skin: current_skin,
            current_skin_config,
            texture_cache: HashMap::new(),
            settings: SettingsHelper::new(), 
        }
    }

    pub async fn init() {
        let s = SKIN_MANAGER.write().await;
        GlobalValueManager::update(Arc::new(CurrentSkin(s.current_skin_config.clone())));
    }

    pub async fn get_texture<N: AsRef<str> + Send + Sync>(name:N, allow_default:bool) -> Option<Image> {
        Self::get_texture_grayscale(name, allow_default, false).await
    }

    pub async fn get_texture_grayscale<N: AsRef<str> + Send + Sync>(name:N, allow_default:bool, grayscale: bool) -> Option<Image> {
        let skin_manager = SKIN_MANAGER.read().await;
        if let Some(t) = skin_manager.texture_cache.get(name.as_ref()) {
            t.clone()
        } else {
            drop(skin_manager);

            let mut skin_manager = SKIN_MANAGER.write().await;
            skin_manager.load_texture_grayscale(
                name, 
                allow_default, 
                grayscale
            ).await
        }
    }

    
    pub async fn current_skin_config() -> Arc<SkinSettings> {
        SKIN_MANAGER.read().await.current_skin_config.clone()
    }

    pub async fn change_skin(new_skin:String) {
        let mut s = SKIN_MANAGER.write().await;
        if s.last_skin == new_skin { return }
        s.last_skin = new_skin.clone();
        s.current_skin_config = Arc::new(SkinSettings::from_file(format!("{SKINS_FOLDER}/{new_skin}/skin.ini")).unwrap_or_default());

        // free up the last skin's images in the atlas for reuse
        std::mem::take(&mut s.texture_cache).into_iter().filter_map(|(_, i)| i.map(|i|i.tex)).for_each(GameWindow::free_texture);

        GlobalValueManager::update(Arc::new(CurrentSkin(s.current_skin_config.clone())));
    }


    pub fn refresh_skins() {
        let mut list = vec!["None".to_owned()];
        for f in std::fs::read_dir(SKINS_FOLDER).unwrap() {
            list.push(f.unwrap().file_name().to_string_lossy().to_string())
        }

        *AVAILABLE_SKINS.write() = list
    }
}

// instance
impl SkinManager {

    // async fn load_texture<N: AsRef<str> + Send + Sync>(&mut self, name:N, allow_default:bool) -> Option<Image> {
    //     // trace!("thread: {:?}", std::thread::current().id());
    //     // trace!("Getting tex: '{}'", name.as_ref());
    //     self.load_texture_grayscale(name, allow_default, false).await
    // }

    async fn load_texture_grayscale<N: AsRef<str> + Send + Sync>(&mut self, name:N, allow_default:bool, grayscale: bool) -> Option<Image> {
        // update settings before we load anything
        self.settings.update();

        let name = name.as_ref().to_owned();

        if !self.texture_cache.contains_key(&name) {
            self.texture_cache.insert(name.clone(), None);

            let mut skin_names = vec![self.settings.current_skin.clone()];
            if allow_default { skin_names.push(DEFAULT_SKIN.to_owned()) }

            'load: for skin_name in skin_names {
                for (tex_name, scale) in vec![(name.clone() + "@2x", Vector2::ONE / 2.0), (name.clone(), Vector2::ONE)] {
                    let tex_path = get_tex_path(&tex_name, &skin_name);

                    if let Some(maybe_img) = load_image(&tex_path, grayscale, scale).await {
                        trace!("loaded tex {tex_path}");
                        self.texture_cache.insert(name.clone(), Some(maybe_img));
                        break 'load;
                    }
                }
            }

            // let tex_path = get_tex_path(&name, &self.settings.current_skin);
            // let mut maybe_img = load_image(&tex_path, grayscale).await;

            // if maybe_img.is_none() && allow_default {
            //     trace!("Skin missing tex {tex_path}/{name}");
            //     maybe_img = load_image(get_tex_path(&name, &DEFAULT_SKIN.to_owned()), grayscale).await;
            // }

            // if let Some(img) = &mut maybe_img {
            //     img.set_size(img.tex_size());
            //     // img.initial_scale = Vector2::ONE;
            //     // img.current_scale = img.initial_scale;
            // }

            // self.texture_cache.insert(name.clone(), maybe_img);
        }

        self.texture_cache.get(&name).unwrap().clone()
    }
}



crate::create_value_helper!(CurrentSkin, Arc<SkinSettings>, CurrentSkinHelper);


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
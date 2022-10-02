#[allow(unused, dead_code)]
use crate::prelude::*;

const DEFAULT_SKIN:&str = "default";
use tokio::sync::RwLock;

lazy_static::lazy_static! {
    static ref SKIN_MANAGER: Arc<RwLock<SkinManager>> = Arc::new(RwLock::new(SkinManager::new()));
}


/// path to a texture file
#[inline]
fn get_tex_path(tex_name:&String, skin_name:&String) -> String {
    format!("{}/{}/{}.png", SKIN_FOLDER, skin_name, tex_name)
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

    pub async fn init() {
        let mut s = SKIN_MANAGER.write().await;
        s.settings = SettingsHelper::new().await;
        s.last_skin = s.settings.current_skin.clone();
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

    pub async fn change_skin(new_skin:String, set_settings: bool) {
        let mut s = SKIN_MANAGER.write().await;
        if s.last_skin == new_skin { return }
        if set_settings {
            let mut s = get_settings_mut!();
            s.current_skin = new_skin.clone();
        }
        s.last_skin = new_skin.clone();
        s.current_skin_config = Arc::new(SkinSettings::from_file(format!("{SKIN_FOLDER}/{new_skin}/skin.ini")).unwrap_or_default());
        s.texture_cache.clear();
    }
}

// instance
impl SkinManager {
    pub fn new() -> Self {
        let settings = tokio::task::block_in_place(||SETTINGS.get().unwrap().blocking_read());
        
        let current_skin = settings.current_skin.clone();
        let current_skin_config = Arc::new(SkinSettings::from_file(format!("{SKIN_FOLDER}/{current_skin}/skin.ini")).unwrap_or_default());
        
        Self {
            last_skin: String::new(),
            current_skin_config,
            texture_cache: HashMap::new(),
            settings: Default::default(), // default because this fn isnt async
        }
    }

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
            let tex_path = get_tex_path(&name, &self.settings.current_skin);
            let mut maybe_img = load_image(tex_path, grayscale).await;

            if maybe_img.is_none() && allow_default {
                trace!("Skin missing tex {}", name);
                maybe_img = load_image(get_tex_path(&name, &DEFAULT_SKIN.to_owned()), grayscale).await;
            }

            if let Some(img) = &mut maybe_img {
                img.set_size(img.tex_size());
                // img.initial_scale = Vector2::one();
                // img.current_scale = img.initial_scale;
            }

            self.texture_cache.insert(name.clone(), maybe_img);
        }

        self.texture_cache.get(&name).unwrap().clone()
    }
}

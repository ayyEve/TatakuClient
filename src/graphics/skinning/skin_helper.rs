#[allow(unused, dead_code)]
use crate::prelude::*;

const DEFAULT_SKIN:&str = "default";


lazy_static::lazy_static! {
    pub static ref SKIN_MANAGER: RwLock<SkinHelper> = RwLock::new(SkinHelper::new());
}


/// path to a texture file
#[inline]
fn get_tex_path(tex_name:&String, skin_name:&String) -> String {
    format!("{}/{}/{}.png", SKIN_FOLDER, skin_name, tex_name)
}

pub struct SkinHelper {
    // current_skin: String,
    current_skin_config: Arc<SkinSettings>,

    texture_cache: HashMap<String, Option<Image>>,
    // audio_cache: HashMap<String, Option<Sound>>,
}

impl SkinHelper {
    pub fn new() -> Self {
        let current_skin = get_settings!().current_skin.clone();
        let current_skin_config = Arc::new(SkinSettings::from_file(format!("{SKIN_FOLDER}/{current_skin}/skin.ini")).unwrap_or_default());
        
        Self {
            // current_skin,
            current_skin_config,
            texture_cache: HashMap::new(),
            // audio_cache: HashMap::new(),
        }
    }

    pub fn current_skin_config(&self) -> Arc<SkinSettings> {
        self.current_skin_config.clone()
    }

    // pub fn current_skin(&self) -> &String {
    //     &self.current_skin
    // }

    pub fn change_skin(&mut self, new_skin:String) {
        get_settings_mut!().current_skin = new_skin.clone();
        // self.current_skin = new_skin.clone();
        self.current_skin_config = Arc::new(SkinSettings::from_file(format!("{SKIN_FOLDER}/{new_skin}/skin.ini")).unwrap_or_default());
        self.texture_cache.clear();

        // self.audio_cache.clear();
    }

    pub fn get_texture<N: AsRef<str>>(&mut self, name:N, allow_default:bool) -> Option<Image> {
        // trace!("thread: {:?}", std::thread::current().id());
        // trace!("Getting tex: '{}'", name.as_ref());
        self.get_texture_grayscale(name, allow_default, false)
    }

    

    pub fn get_texture_grayscale<N: AsRef<str>>(&mut self, name:N, allow_default:bool, grayscale: bool) -> Option<Image> {
        // since opengl stuff needs to be done on the main thread, return none if we arent on it
        if !on_main_thread() { return None }

        let name = name.as_ref().to_owned();

        if !self.texture_cache.contains_key(&name) {
            let mut maybe_img = load_image(get_tex_path(&name, &get_settings!().current_skin), grayscale);

            if maybe_img.is_none() && allow_default {
                info!("Skin missing tex {}", name);
                maybe_img = load_image(get_tex_path(&name, &DEFAULT_SKIN.to_owned()), grayscale);
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

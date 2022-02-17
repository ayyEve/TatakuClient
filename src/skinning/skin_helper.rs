#[allow(unused, dead_code)]
use crate::prelude::*;

const SKIN_FOLDER:&str = "./skins";
const DEFAULT_SKIN:&str = "default";


lazy_static::lazy_static! {
    pub static ref SKIN_MANAGER: RwLock<SkinHelper> = RwLock::new(SkinHelper::new());
}


/// path to a texture file
fn get_tex_path(tex_name:&String, skin_name:&String) -> String {
    format!("{}/{}/{}.png", SKIN_FOLDER, skin_name, tex_name)
}

pub struct SkinHelper {
    current_skin: String,

    texture_cache: HashMap<String, Option<Image>>,
    // audio_cache: HashMap<String, Option<Sound>>,
}

impl SkinHelper {
    pub fn new() -> Self {
        Self {
            current_skin: DEFAULT_SKIN.to_owned(),
            texture_cache: HashMap::new(),
            // audio_cache: HashMap::new(),
        }
    }

    pub fn change_skin(&mut self, new_skin:String) {
        self.current_skin = new_skin;
        self.texture_cache.clear();
        // self.audio_cache.clear();
    }

    pub fn get_texture<N: AsRef<str>>(&mut self, name:N, allow_default:bool) -> Option<Image> {
        let name = name.as_ref().to_owned();
        if !self.texture_cache.contains_key(&name) {
            let mut t = match opengl_graphics::Texture::from_path(get_tex_path(&name, &self.current_skin), &opengl_graphics::TextureSettings::new()) {
                Ok(tex) => {
                    let (og_size_x, og_size_y) = tex.get_size();
                    Some(Image::new(Vector2::zero(), f64::MAX, tex, Vector2::new(og_size_x as f64, og_size_y as f64)))
                }
                Err(e) => {
                    println!("[Skin] Error loading tex \"{}/{}\": {}", self.current_skin, name, e);
                    None
                }
            };

            if t.is_none() && allow_default {
                t = match opengl_graphics::Texture::from_path(get_tex_path(&name, &DEFAULT_SKIN.to_owned()), &opengl_graphics::TextureSettings::new()) {
                    Ok(tex) => {
                        let (og_size_x, og_size_y) = tex.get_size();
                        Some(Image::new(Vector2::zero(), f64::MAX, tex, Vector2::new(og_size_x as f64, og_size_y as f64)))
                    }
                    Err(e) => {
                        println!("[Skin] Error loading default tex \"{}\": {}", name, e);
                        None
                    }
                };
            }

            self.texture_cache.insert(name.clone(), t);
        }

        self.texture_cache.get(&name).unwrap().clone()
    }
}
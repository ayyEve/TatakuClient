use crate::prelude::*;

const DEFAULT_SKIN:&str = "default";

lazy_static::lazy_static! {
    // static ref SKIN_MANAGER: Arc<AsyncRwLock<SkinManager>> = Arc::new(AsyncRwLock::new(SkinManager::new()));
    
    // TODO: change this to skin meta
    pub static ref AVAILABLE_SKINS:Arc<RwLock<Vec<String>>> = {
        let mut list = vec!["None".to_owned()];
        if let Ok(folder) = std::fs::read_dir(SKINS_FOLDER) {
            for f in folder.filter_map(|f| f.ok()) {
                list.push(f.file_name().to_string_lossy().to_string())
            }
        }
        Arc::new(RwLock::new(list))
    };
}

#[cfg(feature="graphics")]
pub struct SkinManager {
    skin_name: String,
    current_skin_config: Arc<SkinSettings>,
    textures: HashMap<(String, bool), HashMap<TextureSource, TextureEntry>>,
}

#[cfg(feature="graphics")]
// static
impl SkinManager {
    pub fn refresh_skins() {
        let mut list = vec!["None".to_owned()];
        for f in std::fs::read_dir(SKINS_FOLDER).unwrap() {
            list.push(f.unwrap().file_name().to_string_lossy().to_string())
        }

        *AVAILABLE_SKINS.write() = list
    }

    pub fn new(settings: &Settings) -> Self {
        let current_skin = settings.current_skin.clone();
        let current_skin_config = Arc::new(SkinSettings::from_file(format!("{SKINS_FOLDER}/{current_skin}/skin.ini")).unwrap_or_default());
        
        Self {
            skin_name: current_skin,
            current_skin_config,
            // texture_cache: HashMap::new(),
            textures: HashMap::new()
        }
    }

    pub fn skin(&self) -> &Arc<SkinSettings> {
        &self.current_skin_config
    }

    pub fn change_skin(&mut self, new_skin: String) {
        if self.skin_name == new_skin { return }
        self.skin_name = new_skin.clone();
        self.current_skin_config = Arc::new(SkinSettings::from_file(format!("{SKINS_FOLDER}/{new_skin}/skin.ini")).unwrap_or_default());

        // free up the last skin's images in the atlas for reuse
        self.free_by_source(TextureSource::Skin);
        self.free_all_unused();
    }

    pub fn free_by_source(&mut self, source: TextureSource) {
        let mut warned = false;

        for i in self.textures.values_mut() {
            let Some(a) = i.get_mut(&source) else { continue };

            if let TextureState::Success(i) = &a.image {
                if i.reference_count() > 1 {
                    if !warned {
                        debug!("Texture(s) still have references, not freeing: {source:?}");
                        warned = true;
                    }
                    continue
                }

                GameWindow::free_texture(*i.tex);
            }

            a.image = TextureState::Unloaded;
        }
    }
    pub fn free_by_usage(&mut self, usage: SkinUsage) {
        let mut warned = false;

        for entry in self.textures.values_mut().map(HashMap::values_mut).flatten() {
            if entry.usage != usage { continue }

            if let TextureState::Success(i) = &entry.image {
                if i.reference_count() > 1 {
                    if !warned {
                        debug!("Texture(s) still have references, not freeing: {usage:?}");
                        warned = true;
                    }

                    continue
                }

                GameWindow::free_texture(*i.tex);
            }

            entry.image = TextureState::Unloaded;
        }
    }

    pub fn free_all_unused(&mut self) {
        for i in self.textures.values_mut().map(HashMap::values_mut).flatten() {
            if let TextureState::Success(im) = &i.image {
                if im.reference_count() > 1 { continue }
                GameWindow::free_texture(*im.tex);
                i.image = TextureState::Unloaded;
            }
        }
    }


    pub async fn get_texture(
        &mut self, 
        name: impl AsRef<str> + Send + Sync, 
        source: &TextureSource,
        usage: SkinUsage,
        grayscale: bool,
    ) -> Option<Image> {
        let name = name.as_ref();
        let texture_key = (name.to_owned(), grayscale);
        if !self.textures.contains_key(&texture_key) {
            self.textures.insert(texture_key.clone(), HashMap::new());
        }
        
        // try to get the exact source if it exists
        let entry = self.textures.get_mut(&texture_key).unwrap();
        
        // try to get the texture with the source, and if that doesnt work, try the fallback, and if that doesnt work, try it's fallback, etc
        let mut try_source = Some(source.clone());
        while let Some(source) = try_source {
            try_source = source.get_fallback();

            match entry.get(&source) {
                // loading from this source has not been attempted yet, try loading it
                None
                // this image has been unloaded, try loading it
                | Some(TextureEntry { image: TextureState::Unloaded, .. })
                => {
                    // try to load the texture
                    let result = Self::load_texture(&source, name, grayscale, &self.skin_name).await;
                    entry.insert(source, TextureEntry { usage, image: result.clone() });

                    match result {
                        TextureState::Success(image) => return Some(image.clone()),
                        TextureState::Failed => continue,
                        TextureState::Unloaded => unreachable!(),
                    }
                }

                Some(TextureEntry { image: TextureState::Failed, .. }) => continue,
                Some(TextureEntry { image: TextureState::Success(image), .. }) => return Some(image.clone()),
            }

        }

        None
    }


    /// helper since most texture loads will look something like this
    pub async fn get_texture_then(
        &mut self,
        name: impl AsRef<str> + Send + Sync, 
        source: &TextureSource,
        usage: SkinUsage,
        grayscale: bool, 
        mut on_loaded: impl FnMut(&mut Image)
    ) -> Option<Image> {
        self
            .get_texture(name, source, usage, grayscale)
            .await
            .map(|mut i| { 
                on_loaded(&mut i); 
                i 
            })
    }


    // try to load a skin from the provided source. does not try fallbacks
    async fn load_texture(
        source: &TextureSource,
        name: impl AsRef<str> + Send + Sync, 
        grayscale: bool,

        skin_name: &str
    ) -> TextureState {
        let name = name.as_ref();
        
        // get paths to check for this source
        // try to load 2x resolution first
        let to_attempt = match source {
            // raw textures wont have a @2x variant
            TextureSource::Raw => vec![ (name.to_owned(), Vector2::ONE) ],

            // everything else should
            _ => vec![ (format!("{name}@2x"), Vector2::ONE / 2.0), (name.to_owned(), Vector2::ONE) ]
        };

        for (tex_name, scale) in to_attempt {
            // get the expected path to the texture file 
            let path = match &source {
                TextureSource::Raw => Path::new(&tex_name).to_path_buf(),
                TextureSource::Beatmap(beatmap_path) => Path::new(&beatmap_path).join(format!("{tex_name}.png")),
                TextureSource::Skin => Path::new(SKINS_FOLDER).join(skin_name).join(format!("{tex_name}.png")),
                TextureSource::DefaultSkin => Path::new(SKINS_FOLDER).join(DEFAULT_SKIN).join(format!("{tex_name}.png")),
            };

            // try loading the bytes. if we cant, try the next source 
            let Ok(buf) = Io::read_file_async(&path).await else { continue };

            // read the file bytes as an image
            match image::load_from_memory(&buf) {
                Err(e) => NotificationManager::add_error_notification(format!("Error loading image: {path:?}"), e).await,
                
                Ok(img) => {
                    let mut img = img.into_rgba8();

                    if grayscale {
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

                    // send the bytes to the gpu to load into the texture atlas
                    let tex = GameWindow::load_texture_data(img).await.expect("no atlas");
                    let image = Image::new(Vector2::ZERO, Arc::new(tex), scale);
                    return TextureState::Success(image);

                    // // create image from the texture
                    // let _image = Image::new(Vector2::ZERO, tex, scale);
                    // if image.is_none() { image = Some(_image.clone()) };

                    // // insert an entry for the image
                    // entries.insert(source, TextureEntry {
                    //     // source,
                    //     usage,
                    //     image: Some(_image),
                    // }); 
                }
            }

            // if let Some(maybe_img) = Self::load_image_inner(&tex_path, grayscale, scale).await {
            //     trace!("loaded tex {tex_path}");
            //     self.texture_cache.insert(name.clone(), Some(maybe_img.clone()));
            //     return Some(maybe_img)
            // }
        }

        TextureState::Failed
    }


    // pub async fn load_texture_async(
    //     &self, 
    //     source: &TextureSource,
    //     name: impl AsRef<str> + Send + Sync, 
    //     grayscale: bool,
    // ) -> AsyncTextureLoader {
    //     let f = async move {

    //     };


    // }

}


// #[derive(Default)]
// pub struct AsyncTextureLoader {
//     loader: AsyncLoader<Vec<(TextureSource, TextureLoadState)>>,
// }
// impl AsyncTextureLoader {
//     pub async fn check(&mut self, skin_manager: &mut SkinManager) -> Option<Image> {
//         let Some(result) = self.loader.check().await else { return None };

//     }
// }



#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum TextureSource {
    /// This texture came from a raw filepath
    Raw,

    /// This texture came from the beatmap's folder
    /// 
    /// Need to provide the beatmap's folder path 
    Beatmap(String),

    /// This texture came from the skin
    Skin,

    /// This texture came from the default skin
    DefaultSkin,
}
impl TextureSource {
    /// Try to get a backup option
    fn get_fallback(&self) -> Option<Self> {
        match self {
            // raw has nothing to fall back to
            Self::Raw => None,

            // if the beatmap doesnt have the texture, fall back to the skin
            Self::Beatmap(_) => Some(Self::Skin),

            // if the skin doesnt have the texture, fall back to the default skin
            Self::Skin => Some(Self::DefaultSkin),

            // if the default skin doesnt have the texture, give up
            Self::DefaultSkin => None,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum SkinUsage {
    /// This texture is used by the game (ie UI elements)
    /// It will only get unloaded on skin refresh (or if manually cleared)
    Game,

    /// This texture is used for the game's background.
    /// 
    /// this is special since it should be freed more often than anything else
    Background,

    /// This texture is used by the GameplayManager or Gamemodes
    /// It will only get unloaded on skin refresh (or if manually cleared)
    /// 
    /// This is basically the same as Self::Game, but allows for more granularity and maybe future use or smth
    Gamemode,

    /// This texture was only used by the beatmap
    /// These textures will be cleaned up after the gameplay manager is completed
    Beatmap,
}

struct TextureEntry {
    // source: TextureSource,
    usage: SkinUsage,
    image: TextureState,
}

#[derive(Default, Clone, Debug)]
enum TextureState {
    // Image was loaded successfully
    Success(Image),

    /// Image failed to load
    /// 
    /// An alternative should be tried instead
    Failed,

    /// Image was unloaded (freed)
    /// 
    /// The image should attempt to be reloaded
    #[default]
    Unloaded,
}


pub struct SkinDropdownable;
#[cfg(feature="graphics")]
impl Dropdownable2 for SkinDropdownable {
    type T = String;
    fn variants() -> Vec<String> {
        AVAILABLE_SKINS.read().clone() //.iter().map(|s|Self::Skin(s.clone())).collect()
    }
}
use crate::prelude::*;

#[async_trait::async_trait]
pub trait SkinProvider: Send + Sync + 'static {
    fn skin(&self) -> &Arc<SkinSettings>;

    async fn get_texture(
        &mut self,
        name: &str,
        source: &TextureSource,
        usage: SkinUsage,
        grayscale: bool,
    ) -> Option<Image>;

    fn free_by_source(&mut self, source: TextureSource);
    fn free_by_usage(&mut self, usage: SkinUsage);

    fn free_all_unused(&mut self);
}

impl dyn SkinProvider {
    /// helper since most texture loads will look something like this
    pub async fn get_texture_then(
        &mut self,
        name: &str,
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
}



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
    pub fn get_fallback(&self) -> Option<Self> {
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

pub struct TextureEntry {
    // source: TextureSource,
    pub usage: SkinUsage,
    pub image: TextureState,
}

#[derive(Default, Clone, Debug)]
pub enum TextureState {
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

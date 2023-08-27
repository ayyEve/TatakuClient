use crate::prelude::*;

pub const PLAYLIST_DATABASE:&str = "playlists.db";

lazy_static::lazy_static! {
    static ref PLAYLIST_MANAGER: Arc<AsyncRwLock<PlaylistManager>> = Default::default();
}

#[derive(Default)]
pub struct PlaylistManager {
    /// keyed by audio file hash
    pub metadatas: HashMap<String, PlaylistItemMetadata>,
    pub playlists: HashMap<String, Playlist>,
}
impl PlaylistManager {
    pub async fn get() -> tokio::sync::RwLockReadGuard<Self> {
        PLAYLIST_MANAGER.read().await
    }
    pub async fn get_mut() -> tokio::sync::RwLockWriteGuard<Self> {
        PLAYLIST_MANAGER.write().await
    }

    pub fn add_playlist_item(&mut self, list_name: &String, item: impl Into<Playlistable>) {
        let to_add;
        match item.into() {
            Playlistable::BeatmapMeta(b) => {
                let file_path = Path::new(&b.file_path).canonicalize().unwrap().parent().unwrap().join(b.audio_filename).to_string_lossy().to_string();
                let hash = Io::get_file_hash(&file_path).unwrap();

                to_add = PlaylistItemMetadata {
                    file_path,
                    hash,
                    artist: b.artist.clone(),
                    title: b.title.clone(),
                    album: None,
                    track_num: None,
                    source: todo!(),
                    wallpaper: todo!(),
                }
            }
        }


        let mut entry = self.playlists.entry(list_name.clone()).or_insert_with(||Playlist::new(list_name.clone()));
        if !entry.items.iter().find(|i|)
    } 
}


/// a full playlist
#[derive(Default, Clone, Debug, tataku_common::Serializable)]
pub struct Playlist {
    pub name: String,
    pub items: Vec<PlaylistItemMetadata>
}
impl Playlist {
    pub fn new(name: String) -> Self {
        Self {
            name,
            items: Vec::new(),
        }
    }
}


#[derive(Default, Clone, Debug, tataku_common::Serializable)]
pub struct PlaylistItemMetadata {
    /// path to the audio file
    pub file_path: String,

    /// hash of the audio file
    pub hash: String,

    pub artist: String,
    pub title: String,
    pub album: Option<String>,
    pub track_num: Option<u16>,

    pub source: Option<String>,

    /// path to a wallpaper file
    pub wallpaper: Option<String>,
}
impl PlaylistMetdata {
    pub fn from_path(path: impl AsRef<Path>) -> TatakuResult<Self> {
        let path = path.as_ref();

    }
}


pub enum Playlistable {
    Path(String),
    BeatmapMeta(Arc<BeatmapMeta>),
    PlaylistMeta(Arc<PlaylistItemMetadata>),
}
impl From<Arc<BeatmapMeta>> for Playlistable {
    fn from(value: Arc<BeatmapMeta>) -> Self {
        Self::BeatmapMeta(value)
    }
}
impl From<Arc<PlaylistItemMetadata>> for Playlistable {
    fn from(value: Arc<PlaylistItemMetadata>) -> Self {
        Self::PlaylistMeta(value)
    }
}
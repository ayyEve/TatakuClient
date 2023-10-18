use std::{fs::File, io::Write};

use super::prelude::*;
use crate::prelude::DOWNLOADS_DIR;


/// how big a direct download item is
pub const DIRECT_ITEM_SIZE:Vector2 = Vector2::new(500.0, 80.0);

// download item is a queue item
const DOWNLOAD_ITEM_SIZE:Vector2 = Vector2::new(300.0, 40.0);
const DOWNLOAD_ITEM_YMARGIN:f32 = 30.0;
const DOWNLOAD_ITEM_YOFFSET:f32 = SEARCH_BAR_HEIGHT + 10.0;
const DOWNLOAD_ITEM_XOFFSET:f32 = 5.0;
const SEARCH_BAR_HEIGHT:f32 = 50.0;

//TODO: properly implement this lol
const MAX_CONCURRENT_DOWNLOADS:usize = 5;

// this whole thing should probably be rewritten 
// now that i know what im doing lmao

type DirectDownloadItem = Arc<dyn DirectDownloadable>;

pub struct DirectMenu {
    scroll_area: ScrollableArea,

    items: HashMap<String, DirectDownloadItem>,
    downloading: Vec<DirectDownloadItem>,

    queue: Vec<DirectDownloadItem>,
    selected: Option<String>,

    /// attempted? succeeded? (path, pos)
    old_audio: Option<Option<(String, f32)>>,

    /// search input
    search_bar: TextInput,

    /// current search api
    current_api: Box<dyn DirectApi>,


    mode: String,
    // status: MapStatus,
    // _converts: bool,

    window_size: Arc<WindowSize>
}
impl DirectMenu {
    pub async fn new(mode: String) -> DirectMenu {
        let window_size = WindowSize::get();

        let mut x = DirectMenu {
            scroll_area: ScrollableArea::new(
                Vector2::new(0.0, SEARCH_BAR_HEIGHT+5.0), 
                Vector2::new(DIRECT_ITEM_SIZE.x, window_size.y - SEARCH_BAR_HEIGHT+5.0), 
                ListMode::VerticalList,
            ),
            downloading: Vec::new(),
            queue: Vec::new(),
            items: HashMap::new(),
            selected: None,
            old_audio: None,

            search_bar: TextInput::new(Vector2::ZERO, Vector2::new(window_size.x , SEARCH_BAR_HEIGHT), "Search", "", Font::Main),
            current_api: Box::new(OsuDirect::new()),

            mode,
            // status: MapStatus::Ranked,
            // _converts: false
            window_size: window_size,
        };

        x.do_search().await;
        x
    }
    async fn do_search(&mut self) {

        // build search params
        let mut search_params = SearchParams::default();
        let q = self.search_bar.get_text();
        if q.len() > 0 {search_params.text = Some(q)}
        search_params.mode = Some(self.mode.clone());

        // perform request
        let items = self.current_api.do_search(search_params).await;

        // clear lists
        self.items.clear();
        self.scroll_area.clear();

        // add items to our list
        for item in items {
            let i = DirectItem::new(item.clone());
            self.items.insert(i.get_tag(), item);
            self.scroll_area.add_item(Box::new(i));
        }

    }

    async fn do_preview_audio(&mut self, item: DirectDownloadItem) {
        if let Some(url) = item.audio_preview() {
            trace!("Preview audio");
            let req = reqwest::blocking::get(url.clone());
            if let Ok(thing) = req {
                let data = match thing.bytes() {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        warn!("Error converting mp3 preview to bytes: {}", e);
                        NotificationManager::add_text_notification("Error loading preview audio", 1000.0, Color::RED).await;
                        return;
                    }
                };
                
                let mut data2 = Vec::new();
                data.iter().for_each(|e| data2.push(e.clone()));

                // store last playing audio if needed
                if self.old_audio.is_none() {
                    if let Some((key, a)) = AudioManager::get_song_raw().await {
                        self.old_audio = Some(Some((key, a.get_position())));
                    }

                    // need to store that we made an attempt
                    if let None = self.old_audio {
                        self.old_audio = Some(None);
                    }
                }

                AudioManager::play_song_raw(url, data2).await.unwrap();
                
            } else if let Err(oof) = req {
                warn!("Error with preview: {}", oof);
            }
        }
    }

    /// go back to the main menu
    async fn back(&mut self, game:&mut Game) {

        if let Some(old_audio) = &self.old_audio {
            // stop the song thats playing, because its a preview
            AudioManager::stop_song().await;

            // restore previous audio
            if let Some((path, pos)) = old_audio.clone() {
                AudioManager::play_song(path, false, pos).await.unwrap();
            }
        }

        // let menu = game.menus.get("main").unwrap().clone();
        game.queue_state_change(GameState::InMenu(Box::new(MainMenu::new().await)));
    }
}

#[async_trait]
impl AsyncMenu<Game> for DirectMenu {
    async fn window_size_changed(&mut self, window_size: Arc<WindowSize>) {
        self.window_size = window_size;
        
    }

    
    async fn update(&mut self, _game:&mut Game) {
        // check download statuses
        let dir = std::fs::read_dir(DOWNLOADS_DIR).unwrap();
        let mut files = Vec::new();
        dir.for_each(|f| {
            if let Ok(thing) = f {
                files.push(thing.file_name().to_string_lossy().to_string());
            }
        });

        self.downloading.retain(|i| {
            !files.contains(&i.filename())
        });

        // while we have items to download and theres room in the queue
        while self.queue.len() > 0 && self.downloading.len() < MAX_CONCURRENT_DOWNLOADS {
            // take from the queue
            let i = self.queue.remove(0);
            // start the download
            i.download().await;
            // add to the downloading
            self.downloading.push(i)
        }
    }

    async fn draw(&mut self, list: &mut RenderableCollection) {
        self.scroll_area.draw(Vector2::ZERO, list);
        self.search_bar.draw(Vector2::ZERO, list);

        // draw download items
        if self.downloading.len() > 0 {

            let x = self.window_size.x - (DOWNLOAD_ITEM_SIZE.x + DOWNLOAD_ITEM_XOFFSET);

            // side bar background and border if hover
            list.push(Rectangle::new(
                Vector2::new(x, DOWNLOAD_ITEM_YOFFSET),
                Vector2::new(DOWNLOAD_ITEM_SIZE.x, self.window_size.y - DOWNLOAD_ITEM_YOFFSET * 2.0),
                Color::WHITE,
                Some(Border::new(Color::BLACK, 1.8))
            ));
            
            let mut counter = 0.0;

            // downloading
            for i in self.downloading.iter() {
                let pos = Vector2::new(x, DOWNLOAD_ITEM_YOFFSET + (DOWNLOAD_ITEM_SIZE.y + DOWNLOAD_ITEM_YMARGIN) * counter);
                // bounding box
                list.push(Rectangle::new(
                    pos,
                    DOWNLOAD_ITEM_SIZE,
                    Color::WHITE,
                    Some(Border::new(Color::BLUE, 1.5))
                ));
                // map text
                list.push(Text::new(
                    pos + Vector2::new(0.0, 15.0),
                    15.0, 
                    format!("{} (Downloading)", i.title()),
                    Color::BLACK,
                    Font::Main
                ));

                counter += 1.0;
            }
            
            // queued
            for i in self.queue.iter() {
                let pos = Vector2::new(x, DOWNLOAD_ITEM_YOFFSET + (DOWNLOAD_ITEM_SIZE.y + DOWNLOAD_ITEM_YMARGIN) * counter);
                // bounding box
                list.push(Rectangle::new(
                    pos,
                    DOWNLOAD_ITEM_SIZE,
                    Color::WHITE,
                    Some(Border::new(Color::BLACK, 1.5))
                ));
                // map text
                list.push(Text::new(
                    pos + Vector2::new(0.0, 15.0),
                    15.0,
                    format!("{} (Waiting...)", i.title()),
                    Color::BLACK,
                    Font::Main
                ));

                counter += 1.0;
            }
        }
    }
    
    async fn on_scroll(&mut self, delta:f32, _game:&mut Game) {
        self.scroll_area.on_scroll(delta);
    }

    async fn on_click(&mut self, pos:Vector2, button:MouseButton, mods:KeyModifiers, _game:&mut Game) {
        self.search_bar.on_click(pos, button, mods);

        // check if item was clicked
        if let Some(key) = self.scroll_area.on_click_tagged(pos, button, mods) {
            if let Some(selected) = self.selected.clone() {
                if key == selected {
                    if let Some(item) = self.items.get(&key) {
                        if item.is_downloading() {return};
                        self.queue.push(item.clone());
                    }
                    return;
                }
            }

            if let Some(item) = self.items.clone().get(&key) {
               self.do_preview_audio(item.clone()).await;
            }
            self.selected = Some(key.clone());
        }
    }

    async fn on_mouse_move(&mut self, pos:Vector2, _game:&mut Game) {
        self.search_bar.on_mouse_move(pos);
        self.scroll_area.on_mouse_move(pos);
    }

    async fn on_key_press(&mut self, key:Key, game:&mut Game, mods:KeyModifiers) {
        use Key::*;
        self.search_bar.on_key_press(key, mods);
        if key == Escape {return self.back(game).await}


        if mods.alt {
            let new_mode = match key {
                Key1 => Some("osu".to_owned()),
                Key2 => Some("taiko".to_owned()),
                Key3 => Some("catch".to_owned()),
                Key4 => Some("mania".to_owned()),
                _ => None
            };

            if let Some(new_mode) = new_mode {
                if self.mode != new_mode {
                    NotificationManager::add_text_notification(&format!("Searching for {} maps", new_mode), 1000.0, Color::BLUE).await;
                    self.mode = new_mode;
                    self.do_search().await;
                }
            }
        }
        // if mods.ctrl {
        //     let new_status = match key {
        //         D1 => Some(MapStatus::Graveyarded),
        //         D2 => Some(MapStatus::Ranked),
        //         D3 => Some(MapStatus::Approved),
        //         D4 => Some(MapStatus::Pending),
        //         D5 => Some(MapStatus::Loved),
        //         D6 => Some(MapStatus::All),
        //         _ => None
        //     };

        //     if let Some(new_status) = new_status {
        //         if self.status != new_status {
        //             self.status = new_status;
        //             self.do_search();
        //             NotificationManager::add_text_notification(&format!("Searching for {:?} maps", new_status), 1000.0, Color::BLUE);
        //         }
        //     }
        // }



        if key == Return {
            self.do_search().await;
        }
    }

    async fn on_text(&mut self, text:String) {
        self.search_bar.on_text(text);
    }
}


/// perform a download on another thread
pub(crate) fn perform_download(url:String, path:String) {
    debug!("Downloading '{}' to '{}'", url, path);
    tokio::spawn(async {
        let file_bytes = reqwest::get(url)
            .await
            .expect("error with request")
            .bytes()
            .await
            .expect("error getting bytes");
        
        write_file(path, &file_bytes).expect("Error writing file");
    });
}

fn write_file(file:String, bytes:&[u8]) -> std::io::Result<()> {
    let mut f = File::create(file)?;
    f.write_all(bytes)?;
    f.flush()?;
    Ok(())
}

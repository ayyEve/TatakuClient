use crate::prelude::*;

const SEARCH_BAR_HEIGHT:f32 = 50.0;

//TODO: properly implement this lol
const MAX_CONCURRENT_DOWNLOADS:usize = 5;

pub type DirectDownloadQueue = Vec<Arc<dyn DirectDownloadable>>;
// type DirectDownloadItem = Arc<dyn DirectDownloadable>;

pub struct DirectMenu {
    actions: ActionQueue,

    scroll_area: ScrollableArea,

    /// attempted? succeeded? (path, pos)
    old_audio: Option<Option<(String, f32)>>,

    /// search input
    search_bar: TextInput,

    /// current search api
    current_api: Box<dyn DirectApi>,

    /// what was the last tag that was clicked
    last_clicked_tag: String,

    mode: String,
    // status: MapStatus,
    // _converts: bool,

    window_size: Arc<WindowSize>,
}
impl DirectMenu {
    pub async fn new(mode: String) -> DirectMenu {
        let window_size = WindowSize::get();

        let mut x = DirectMenu {
            actions: ActionQueue::new(),

            scroll_area: ScrollableArea::new(
                Vector2::new(0.0, SEARCH_BAR_HEIGHT+5.0), 
                Vector2::new(DIRECT_ITEM_SIZE.x, window_size.y - SEARCH_BAR_HEIGHT+5.0), 
                ListMode::VerticalList,
            ),
            // items: HashMap::new(),
            old_audio: None,

            search_bar: TextInput::new(Vector2::ZERO, Vector2::new(window_size.x , SEARCH_BAR_HEIGHT), "Search", "", Font::Main),
            current_api: Box::new(OsuDirect),

            mode,
            last_clicked_tag: String::new(),
            // status: MapStatus::Ranked,
            // _converts: false
            window_size,
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

        // clear list
        self.scroll_area.clear();

        // add items to our list
        let queue = GlobalValueManager::get::<DirectDownloadQueue>()
            .unwrap()
            .iter()
            .map(|i|(i.filename(), i.clone()))
            .collect::<HashMap<_, _>>();

        for mut item in items {
            queue.get(&item.filename()).ok_do(|i|item = (*i).clone());
            
            self.scroll_area.add_item(Box::new(DirectItem::new(item, false)));
        }

    }

    async fn do_preview_audio(&mut self, tag: String) {
        let Some(url) = tag.split("|").nth(1) else { return }; 
        if url.is_empty() { return }

        trace!("Preview audio");
        let req = reqwest::get(url).await;
        if let Ok(thing) = req {
            let data = match thing.bytes().await {
                Ok(bytes) => bytes,
                Err(e) => {
                    warn!("Error converting mp3 preview to bytes: {}", e);
                    NotificationManager::add_text_notification("Error loading preview audio", 1000.0, Color::RED).await;
                    return;
                }
            };
            
            let data = data.iter().copied().collect();
            // let mut data2 = Vec::new();
            // data.iter().for_each(|e| data2.push(e.clone()));

            // store last playing audio if needed

            // if self.old_audio.is_none() {
            //     if let Some((key, a)) = AudioManager::get_song_raw().await {
            //         self.old_audio = Some(Some((key, a.get_position())));
            //     }

            //     // need to store that we made an attempt
            //     if let None = self.old_audio {
            //         self.old_audio = Some(None);
            //     }
            // }
            self.actions.push(SongAction::Set(SongMenuSetAction::PushQueue));
            
            // AudioManager::play_song_raw(url, data2).await.unwrap();
            self.actions.push(SongAction::Set(SongMenuSetAction::FromData(data, url.to_owned(), SongPlayData {
                play: true,
                volume: Some(Settings::get().get_music_vol()),
                ..Default::default()
            })));
            
        } else if let Err(oof) = req {
            warn!("Error with preview: {}", oof);
        }
        
    }

    /// go back to the main menu
    async fn back(&mut self, game:&mut Game) {
        self.actions.push(SongAction::Set(SongMenuSetAction::PopQueue));
        self.actions.push(SongAction::Play);

        // if let Some(old_audio) = &self.old_audio {
        //     // stop the song thats playing, because its a preview
        //     AudioManager::stop_song().await;

        //     // restore previous audio
        //     if let Some((path, pos)) = old_audio.clone() {
        //         AudioManager::play_song(path, false, pos).await.unwrap();
        //     }
        // }

        // let menu = game.menus.get("main").unwrap().clone();
        game.queue_state_change(GameState::SetMenu(Box::new(MainMenu::new().await)));
    }
}

#[async_trait]
impl AsyncMenu for DirectMenu {

    
    async fn update(&mut self, _values: &mut ValueCollection) -> Vec<TatakuAction> {
        self.scroll_area.update();
        self.search_bar.update();

        // if the tag ends with true, the download was complete
        let len = self.scroll_area.items.len();
        self.scroll_area.items.retain(|i|i.get_tag().ends_with("false"));
        if self.scroll_area.items.len() != len {
            self.scroll_area.refresh_layout();
        }
        

        // TODO: move this to a thread
        check_direct_download_queue();
        self.actions.take()
    }

    
    fn view(&self, _values: &mut ValueCollection) -> IcedElement {
        use iced_elements::*;

        col!(
            ;
        )
    }
    
    async fn handle_message(&mut self, message: Message, _values: &mut ValueCollection) {
        info!("got message {message:?}");
    }


    // async fn draw(&mut self, list: &mut RenderableCollection) {
    //     self.scroll_area.draw(Vector2::ZERO, list);
    //     self.search_bar.draw(Vector2::ZERO, list);

    //     // // draw download items
    //     // let x = self.window_size.x - (DOWNLOAD_ITEM_SIZE.x + DOWNLOAD_ITEM_XOFFSET);

    //     // // side bar background and border if hover
    //     // list.push(Rectangle::new(
    //     //     Vector2::new(x, DOWNLOAD_ITEM_YOFFSET),
    //     //     Vector2::new(DOWNLOAD_ITEM_SIZE.x, self.window_size.y - DOWNLOAD_ITEM_YOFFSET * 2.0),
    //     //     Color::WHITE,
    //     //     Some(Border::new(Color::BLACK, 1.8))
    //     // ));
        
    //     // let mut counter = 0.0;

    //     // // downloading
    //     // for i in self.downloading.iter() {
    //     //     let pos = Vector2::new(x, DOWNLOAD_ITEM_YOFFSET + (DOWNLOAD_ITEM_SIZE.y + DOWNLOAD_ITEM_YMARGIN) * counter);
    //     //     // bounding box
    //     //     list.push(Rectangle::new(
    //     //         pos,
    //     //         DOWNLOAD_ITEM_SIZE,
    //     //         Color::WHITE,
    //     //         Some(Border::new(Color::BLUE, 1.5))
    //     //     ));
    //     //     // map text
    //     //     list.push(Text::new(
    //     //         pos + Vector2::new(0.0, 15.0),
    //     //         15.0, 
    //     //         format!("{} (Downloading)", i.title()),
    //     //         Color::BLACK,
    //     //         Font::Main
    //     //     ));

    //     //     counter += 1.0;
    //     // }
            
    //     // // queued
    //     // for i in self.queue.iter() {
    //     //     let pos = Vector2::new(x, DOWNLOAD_ITEM_YOFFSET + (DOWNLOAD_ITEM_SIZE.y + DOWNLOAD_ITEM_YMARGIN) * counter);
    //     //     // bounding box
    //     //     list.push(Rectangle::new(
    //     //         pos,
    //     //         DOWNLOAD_ITEM_SIZE,
    //     //         Color::WHITE,
    //     //         Some(Border::new(Color::BLACK, 1.5))
    //     //     ));
    //     //     // map text
    //     //     list.push(Text::new(
    //     //         pos + Vector2::new(0.0, 15.0),
    //     //         15.0,
    //     //         format!("{} (Waiting...)", i.title()),
    //     //         Color::BLACK,
    //     //         Font::Main
    //     //     ));

    //     //     counter += 1.0;
    //     // }
    // }
    

    // async fn window_size_changed(&mut self, window_size: Arc<WindowSize>) {
    //     self.window_size = window_size;
        
    // }

    // async fn on_scroll(&mut self, delta:f32, _game:&mut Game) {
    //     self.scroll_area.on_scroll(delta);
    // }

    // async fn on_click(&mut self, pos:Vector2, button:MouseButton, mods:KeyModifiers, _game:&mut Game) {
    //     self.search_bar.on_click(pos, button, mods);

    //     // check if item was clicked
    //     if let Some(tag) = self.scroll_area.on_click_tagged(pos, button, mods) {
    //         if self.last_clicked_tag == tag {
    //             // item will add itself to the download queue

    //         } else {
    //            self.do_preview_audio(tag.clone()).await;
    //         }
    //     }
    // }

    // async fn on_mouse_move(&mut self, pos:Vector2, _game:&mut Game) {
    //     self.search_bar.on_mouse_move(pos);
    //     self.scroll_area.on_mouse_move(pos);
    // }

    // async fn on_key_press(&mut self, key:Key, game:&mut Game, mods:KeyModifiers) {
    //     use Key::*;
    //     self.search_bar.on_key_press(key, mods);
    //     if key == Escape {return self.back(game).await}


    //     if mods.alt {
    //         let new_mode = match key {
    //             Key1 => Some("osu".to_owned()),
    //             Key2 => Some("taiko".to_owned()),
    //             Key3 => Some("catch".to_owned()),
    //             Key4 => Some("mania".to_owned()),
    //             _ => None
    //         };

    //         if let Some(new_mode) = new_mode {
    //             if self.mode != new_mode {
    //                 NotificationManager::add_text_notification(&format!("Searching for {} maps", new_mode), 1000.0, Color::BLUE).await;
    //                 self.mode = new_mode;
    //                 self.do_search().await;
    //             }
    //         }
    //     }
    //     // if mods.ctrl {
    //     //     let new_status = match key {
    //     //         D1 => Some(MapStatus::Graveyarded),
    //     //         D2 => Some(MapStatus::Ranked),
    //     //         D3 => Some(MapStatus::Approved),
    //     //         D4 => Some(MapStatus::Pending),
    //     //         D5 => Some(MapStatus::Loved),
    //     //         D6 => Some(MapStatus::All),
    //     //         _ => None
    //     //     };

    //     //     if let Some(new_status) = new_status {
    //     //         if self.status != new_status {
    //     //             self.status = new_status;
    //     //             self.do_search();
    //     //             NotificationManager::add_text_notification(&format!("Searching for {:?} maps", new_status), 1000.0, Color::BLUE);
    //     //         }
    //     //     }
    //     // }



    //     if key == Return {
    //         self.do_search().await;
    //     }
    // }

    // async fn on_text(&mut self, text:String) {
    //     self.search_bar.on_text(text);
    // }
}


/// perform a download on another thread
pub(crate) fn perform_download(url:String, path:String, progress: Arc<RwLock<DownloadProgress>>) {
    debug!("Downloading '{url}' to '{path}'");

    tokio::spawn(async move {
        Downloader::download_existing_progress(DownloadOptions::new(url, 5), progress.clone());

        loop {
            tokio::task::yield_now().await;

            let progress = progress.read();
            if progress.complete() {
                debug!("direct download completed");
                let bytes = progress.data.as_ref().unwrap();
                std::fs::write(path, bytes).unwrap();
                break;
            }

            if progress.failed() {
                if let Some(e) = &progress.error {
                    error!("failed: {e}");
                }
                break;
            }
        }
    });
}



pub fn check_direct_download_queue() {
    let max_downloads = MAX_CONCURRENT_DOWNLOADS;

    let mut queue = GlobalValueManager::get_mut::<DirectDownloadQueue>().unwrap();
    let mut download_count = 0;

    queue.retain(|i| {
        if !i.is_downloading() { return true }

        let progress = i.get_download_progress().read();
        if progress.complete() || progress.failed() { return false; }

        download_count += 1;
        true
    });

    while download_count < max_downloads {
        let Some(a) = queue.iter().find(|a|!a.is_downloading()) else { break };
        a.download();
        download_count += 1;
    }

}
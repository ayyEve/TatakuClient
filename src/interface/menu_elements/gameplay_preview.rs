use crate::prelude::*;

pub struct GameplayPreview {
    actions: ActionQueue,
    // current_mods: ModManagerHelper,

    beatmap: SyValueHelper,
    playmode: SyValueHelper,
    mods: SyValueHelper,
    song_time: SyValueHelper,

    settings: SettingsHelper,
    manager: Option<GameplayId>,

    owner: MessageOwner,

    pub visualization: Option<Box<dyn Visualization>>,

    /// area to fit to
    pub fit_to: Option<Bounds>,

    /// if a song ends, should we handle restarting it?
    pub handle_song_restart: bool,

    /// use bg game settings, or global gamemode?
    use_global_playmode: bool,
    apply_rate: bool,
    check_enabled: Arc<dyn Fn(&Settings) -> bool + Send + Sync>,

    // loader: Option<AsyncLoader<TatakuResult<IngameManager>>>,

    widget_sender: Arc<Mutex<TripleBufferSender<Arc<dyn TatakuRenderable>>>>,
    event_receiver: AsyncReceiver<Bounds>,

    widget: GameplayPreviewWidget
}
impl GameplayPreview {
    pub fn new(use_global_playmode: bool, apply_rate: bool, check_enabled: Arc<dyn Fn(&Settings) -> bool + Send + Sync>, owner: MessageOwner) -> Self {
        let a: Arc<dyn TatakuRenderable> = Arc::new(TransformGroup::new(Vector2::ZERO));   
        let (widget_sender, widget_receiver) = TripleBuffer::new(&a).split();
        let (event_sender, event_receiver) = async_channel(5);

        let widget = GameplayPreviewWidget::new(widget_receiver, event_sender);
        
        Self {
            actions: ActionQueue::new(),
            // current_mods: ModManagerHelper::new(),
            beatmap: SyValueHelper::new("map.hash"),
            playmode: SyValueHelper::new("global.playmode_actual"),
            mods: SyValueHelper::new("global.mods"),
            song_time: SyValueHelper::new("song.position"),

            visualization: None,
            handle_song_restart: false,
            owner,

            settings: SettingsHelper::new(),
            manager: None,
            fit_to: None,
            use_global_playmode,
            apply_rate,
            // loader: None,
            check_enabled,

            widget_sender: Arc::new(Mutex::new(widget_sender)),
            event_receiver,
            widget
        }
    }

    pub fn width(mut self, width: iced::Length) -> Self {
        self.widget.width = width;
        self
    }
    pub fn height(mut self, height: iced::Length) -> Self {
        self.widget.height = height;
        self
    }

    pub fn is_enabled(&self) -> bool {
        (self.check_enabled)(&self.settings)
    }

    pub async fn setup(&mut self, _values: &ValueCollection, actions: &mut ActionQueue) {
        // self.current_playmode.update();
        // self.current_beatmap.update();
        // self.current_mods.update();
        self.settings.update();

        // make sure we're enabled before doing anything else
        if !self.is_enabled() { return }

        // // get the map hash and path
        // let Ok(hash) = Md5Hash::try_from(self.beatmap.as_string()) else { return trace!("manager no map") };
        // let Ok(path) = values.get_string("map.path") else { return error!("no map path") };
        // // let Some(map) = &self.current_beatmap.0 else { return };

        // // get the mode
        // let mode = if self.use_global_playmode { 
        //     self.playmode.as_string()
        //     // self.current_playmode.as_ref().0.clone() 
        // } else { 
        //     self.settings.background_game_settings.mode.clone() 
        // };

        // // abort the previous loading task
        // self.loader.ok_do(|i| i.abort());

        // let mods = values.try_get::<ModManager>("global.mods").unwrap_or_default();

        // let map = map.clone();
        // let f = async move { manager_from_playmode_path_hash(mode, path, hash, mods).await };
        // self.loader = Some(AsyncLoader::new(f));

        let draw_sender = self.widget_sender.clone();
        actions.push(GameAction::NewGameplayManager(NewManager {
            owner: self.owner,
            playmode: (!self.use_global_playmode).then(|| self.settings.background_game_settings.mode.clone()),
            gameplay_mode: Some(GameplayMode::Preview),
            area: self.fit_to,
            draw_function: Some(Arc::new(move |group| draw_sender.lock().write(Arc::new(group)))),

            ..Default::default()
        }));


        // match manager_from_playmode(mode, &map).await {
        //     Ok(mut manager) => {
        //         manager.make_menu_background();

        //         if let Some((pos, size)) = self.fit_to {
        //             manager.gamemode.fit_to_area(pos, size).await
        //         }
                
        //         manager.start().await;
        //         trace!("manager started");

        //         self.manager = Some(manager);
        //         // self.visualization.song_changed(&mut self.background_game);
        //     },
        //     Err(e) => {
        //         // self.visualization.song_changed(&mut None);
        //         NotificationManager::add_error_notification("Error loading beatmap", e).await;
        //     }
        // }

    }

    pub async fn update(&mut self, values: &mut ValueCollection, actions: &mut ActionQueue) {
        // check for settings changes
        if self.settings.update() {
            if !self.is_enabled() && self.manager.is_some() {
                self.manager = None;
            }
        }


        let last_song_time = self.song_time.as_f32().unwrap_or_default();
        if self.song_time.check(values) {
            if self.song_time.as_f32().unwrap_or_default() < last_song_time {
                self.setup(values, actions).await;
            }
        }


        // check for map/mode changes
        if self.beatmap.check(values) | self.playmode.check(values) {
            self.setup(values, actions).await;
        }
        // let mut refresh_map = self.current_playmode.update();
        // refresh_map |= self.current_beatmap.update();
        // if refresh_map { self.setup().await; }

        // check for new bounds
        if let Some(new_bounds) = self.event_receiver.try_recv().ok().filter(|bounds| Some(bounds) != self.fit_to.as_ref()) {
            self.fit_to_area(new_bounds).await;
        }

        // // update manager
        // if let Some(manager) = &mut self.manager {
        //     manager.update(values).await;
            
        //     if manager.completed {
        //         manager.on_complete();
        //         self.manager = None;
        //     }
        // }

        // update vis
        if let Some(vis) = &mut self.visualization {
            vis.update().await;
        }
        
        // check for state update
        if self.handle_song_restart {
            let stopped = values.get_bool("song.stopped").unwrap();
            let playing = values.get_bool("song.playing").unwrap();
            let paused = values.get_bool("song.paused").unwrap();
            let exists = stopped || playing || paused;

            let speed = ModManager::try_from(self.mods.deref()).map(|m| m.get_speed()).unwrap_or(1.0);

            if exists {
                if stopped {
                    if let Ok(preview) = values.get_f32("map.preview_time") {
                        actions.push(SongAction::SetPosition(preview));
                        if self.apply_rate {
                            actions.push(SongAction::SetRate(speed));
                        }

                        actions.push(SongAction::Play);
                    }
                }
            } else {
                if let Some(path) = values.get_string("map.audio_path").ok().filter(|s| !s.is_empty()) {
                    actions.push(SongAction::Set(SongMenuSetAction::FromFile(path, SongPlayData {
                        play: true,
                        position: values.get_f32("map.preview_time").ok(),
                        rate: self.apply_rate.then_some(speed),
                        volume: Some(self.settings.get_music_vol()),

                        ..Default::default()
                    })));
                }
            }

            // match AudioManager::get_song().await {
            //     Some(song) if !song.is_playing() && !song.is_paused() => {
            //         // restart the song at the preview point
            //         if let Some(map) = &self.current_beatmap.clone().0 {
            //             let _ = song.set_position(map.audio_preview);
            //             if self.apply_rate { song.set_rate(self.current_mods.get_speed()); }
                        
            //             song.play(false);
            //             self.setup().await;
            //         }
            //     }

            //     // no value, try to set it to something
            //     None => if let Some(map) = &self.current_beatmap.clone().0 {
            //         match AudioManager::play_song(map.audio_filename.clone(), true, map.audio_preview).await {
            //             Ok(audio) => if self.apply_rate { audio.set_rate(self.current_mods.get_speed()); },
            //             Err(_) => {error!("failed to set audio, crying")},
            //         }
            //     }
            //     _ => {}
            // }
        }

        // render to the drawable
        self.draw().await;

        actions.extend(self.actions.take());
    }

    async fn draw(&mut self) {
        let mut list = RenderableCollection::new();

        // if let Some(manager) = &mut self.manager {
        //     manager.draw(&mut list).await;
        // }

        // draw visualization if it exists
        if let Some((vis, bounds)) = self.visualization.as_mut().zip(self.fit_to) {
            vis.draw(bounds, &mut list).await;
        }

        let mut group = TransformGroup::from_collection(Vector2::ZERO, list);
        group.set_scissor(self.fit_to.map(|b| b.into_scissor()));
        self.widget_sender.lock().write(Arc::new(group));
    }

    // pub async fn key_down(&mut self, key:KeyInput, mods:KeyModifiers) {
    //     let Some(manager) = self.manager.as_mut() else { return };
    //     manager.key_down(key, mods).await
    // }

    // pub async fn window_size_changed(&mut self, window_size: Arc<WindowSize>) {
    //     let Some(manager) = self.manager.as_mut() else { return };
    //     manager.window_size_changed(window_size).await
    // }

    pub async fn fit_to_area(&mut self, bounds: Bounds) {
        // info!("fitting to area {bounds:?}");
        self.fit_to = Some(bounds);

        let Some(manager) = self.manager.as_ref() else { return };
        self.actions.push(GameAction::GameplayAction(manager.clone(), GameplayAction::FitToArea(bounds)));
    } 

    pub async fn skin_changed(&mut self, skin_manager: &mut SkinManager) {
        if let Some(vis) = &mut self.visualization {
            vis.reload_skin(skin_manager).await;
        }
    }

}

impl Clone for GameplayPreview {
    fn clone(&self) -> Self {
        Self::new(self.use_global_playmode, self.apply_rate, self.check_enabled.clone(), self.owner)
    }
}
impl core::fmt::Debug for GameplayPreview {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GameplayPreview")
    }
}



/// this is the widget that gets added to the ui
#[derive(Clone)]
struct GameplayPreviewWidget {
    width: iced::Length,
    height: iced::Length,

    draw_data: Arc<Mutex<TripleBufferReceiver<Arc<dyn TatakuRenderable>>>>,
    event_sender: AsyncSender<Bounds>,
}
impl GameplayPreviewWidget {
    fn new(draw_data: TripleBufferReceiver<Arc<dyn TatakuRenderable>>, event_sender: AsyncSender<Bounds>) -> Self {
        Self {
            width: iced::Length::Fill,
            height: iced::Length::Fill,
            draw_data: Arc::new(Mutex::new(draw_data)),
            event_sender,
        }
    }
}

use iced::advanced::widget::tree;
impl iced::advanced::Widget<Message, iced::Theme, IcedRenderer> for GameplayPreviewWidget {
    fn size(&self) -> iced::Size<iced::Length> { iced::Size::new(self.width, self.height) }
    fn tag(&self) -> tree::Tag { tree::Tag::of::<GameplayWidgetState>() }
    fn state(&self) -> tree::State { tree::State::new(GameplayWidgetState::default()) }

    fn layout(
        &self,
        _state: &mut tree::Tree,
        _renderer: &IcedRenderer,
        limits: &iced_core::layout::Limits,
    ) -> iced_core::layout::Node {
        let limits = limits
            .width(self.width)
            .height(self.height);

        iced_core::layout::Node::new(limits.max())
    }


    fn draw(
        &self,
        tree: &iced_core::widget::Tree,
        renderer: &mut IcedRenderer,
        _theme: &iced::Theme,
        _style: &iced_core::renderer::Style,
        layout: iced_core::Layout<'_>,
        _cursor: iced_core::mouse::Cursor,
        _viewport: &iced::Rectangle,
    ) {
        let state = tree.state.downcast_ref::<GameplayWidgetState>();
        let bounds = layout.bounds();
        let mut state_bounds = state.bounds.lock();
        if &*state_bounds != &bounds {
            *state_bounds = bounds;
            let _ = self.event_sender.try_send(bounds.into());
        }

        renderer.add_renderable(self.draw_data.lock().read().clone());
    }
}

impl Into<IcedElement> for GameplayPreviewWidget {
    fn into(self) -> IcedElement {
        IcedElement::new(self)
    }
}

#[derive(Default)]
struct GameplayWidgetState {
    bounds: Rc<Mutex<iced::Rectangle>>,
}


#[async_trait]
impl Widgetable for GameplayPreview {
    async fn handle_message(&mut self, message: &Message, _values: &mut ValueCollection) -> Vec<TatakuAction> { 
        let MessageTag::String(str) = &message.tag else { return self.actions.take() };
        if str != "gameplay_manager_create" { return self.actions.take() }

        let MessageType::GameplayManagerId(id) = &message.message_type else { return { error!("wrong type"); self.actions.take() } };
        self.manager = Some(id.clone());

        self.actions.push(GameAction::GameplayAction(id.clone(), GameplayAction::Resume));
        
        self.actions.take()
    }

    async fn update(&mut self, values: &mut ValueCollection, actions: &mut ActionQueue) {
        GameplayPreview::update(self, values, actions).await
        // (self as &mut GameplayPreview).update().await
    }
    fn view(&self, _owner: MessageOwner, _values: &mut ValueCollection) -> IcedElement {
        self.widget.clone().into()
    }
}

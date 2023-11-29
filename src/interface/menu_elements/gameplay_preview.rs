
use crate::prelude::*;



pub struct GameplayPreview {
    pub current_beatmap: CurrentBeatmapHelper,
    pub current_playmode: CurrentPlaymodeHelper,
    current_mods: ModManagerHelper,
    settings: SettingsHelper,
    pub manager: Option<IngameManager>,

    pub fit_to: Option<Bounds>,

    /// use bg game settings, or global gamemode?
    use_global_playmode: bool,
    apply_rate: bool,
    check_enabled: Box<dyn Fn(&Settings) -> bool + Send + Sync>,

    loader: Option<AsyncLoader<TatakuResult<IngameManager>>>,

    widget_sender: TripleBufferSender<Arc<dyn TatakuRenderable>>,
    event_receiver: AsyncReceiver<Bounds>,

    widget: GameplayPreviewWidget
}
impl GameplayPreview {
    pub fn new(use_global_playmode: bool, apply_rate: bool, check_enabled: Box<dyn Fn(&Settings) -> bool + Send + Sync>) -> Self {
        let a: Arc<dyn TatakuRenderable> = Arc::new(TransformGroup::new(Vector2::ZERO));   
        let (widget_sender, widget_receiver) = TripleBuffer::new(&a).split();
        let (event_sender, event_receiver) = async_channel(5);

        let widget = GameplayPreviewWidget::new(widget_receiver, event_sender);
        
        Self {
            current_beatmap: CurrentBeatmapHelper::new(),
            current_playmode: CurrentPlaymodeHelper::new(),
            current_mods: ModManagerHelper::new(),
            settings: SettingsHelper::new(),
            manager: None,
            fit_to: None,
            use_global_playmode,
            apply_rate,
            loader: None,
            check_enabled,

            widget_sender,
            event_receiver,
            widget
        }
    }

    pub async fn setup(&mut self) {
        self.current_playmode.update();
        self.current_beatmap.update();
        self.current_mods.update();
        self.settings.update();


        if !(self.check_enabled)(&self.settings) { return }

        // if !self.settings.background_game_settings.beatmap_select_enabled { return }

        let settings = self.settings.background_game_settings.clone();
        // if !settings.main_menu_enabled { return }

        let map = match &self.current_beatmap.0 {
            Some(map) => map,
            None => return trace!("manager no map")
        };

        let mode = if self.use_global_playmode { self.current_playmode.as_ref().0.clone() } else { settings.mode.clone() };

        if self.loader.is_some() {
            // stop it somehow?
        }

        let map = map.clone();
        let f = async move {manager_from_playmode(mode, &map).await};
        self.loader = Some(AsyncLoader::new(f));

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

    pub async fn update(&mut self) {
        if self.settings.update() {
            if let Some(manager) = &mut self.manager {
                manager.force_update_settings().await;
            }
        }

        if self.current_mods.update() {
            if let Some(manager) = &mut self.manager {
                manager.apply_mods(self.current_mods.as_ref().clone()).await;
            }
        }

        let mut refresh_map = self.current_playmode.update();
        refresh_map |= self.current_beatmap.update();
        if refresh_map { self.setup().await; }
        if let Ok(new_bounds) = self.event_receiver.try_recv() {
            if Some(new_bounds) != self.fit_to {
                self.fit_to_area(new_bounds).await;
            }
        }

        if let Some(manager) = &mut self.manager {
            manager.update().await;
            
            if manager.completed {
                manager.on_complete();
                self.manager = None;
            }
        }

        if let Some(loader) = &self.loader {
            if let Some(result) = loader.check().await {
                self.loader = None;
                
                match result {
                    Ok(mut manager) => {
                        manager.set_mode(GameplayMode::Preview);

                        if let Some(bounds) = self.fit_to {
                            manager.fit_to_area(bounds).await
                        }
                        
                        manager.start().await;
                        self.manager = Some(manager);
                    },
                    Err(e) => NotificationManager::add_error_notification("Error loading beatmap", e).await,
                }
            }
        }
        
        match AudioManager::get_song().await {
            Some(song) => {
                if !song.is_playing() && !song.is_paused() {
                    // restart the song at the preview point
                    if let Some(map) = &self.current_beatmap.clone().0 {
                        let _ = song.set_position(map.audio_preview);
                        if self.apply_rate { song.set_rate(self.current_mods.get_speed()); }
                        
                        song.play(false);
                        self.setup().await;
                    }
                }
            }

            // no value, try to set it to something
            _ => {
                if let Some(map) = &self.current_beatmap.clone().0 {
                    match AudioManager::play_song(map.audio_filename.clone(), true, map.audio_preview).await {
                        Ok(audio) => if self.apply_rate { audio.set_rate(self.current_mods.get_speed()); },
                        Err(_) => {error!("failed to set audio, crying")},
                    }
                }
            },
        }

        self.draw().await
    }

    async fn draw(&mut self) {
        let Some(manager) = &mut self.manager else { return };
        let mut list = RenderableCollection::new();
        manager.draw(&mut list).await;

        let mut group = TransformGroup::new(Vector2::ZERO);
        for i in list.take() {
            group.push_arced(i)
        }

        self.widget_sender.write(Arc::new(group));
    }

    pub async fn key_down(&mut self, key:Key, mods:KeyModifiers) {
        if let Some(manager) = &mut self.manager {
            manager.key_down(key, mods).await
        }
    }

    pub async fn window_size_changed(&mut self, window_size: Arc<WindowSize>) {
        if let Some(manager) = &mut self.manager {
            manager.window_size_changed(window_size).await
        }
    }

    pub async fn fit_to_area(&mut self, bounds: Bounds) {
        info!("fitting to area {bounds:?}");
        self.fit_to = Some(bounds);

        if let Some(manager) = &mut self.manager {
            manager.fit_to_area(bounds).await
        }
    } 


    pub fn widget(&self) -> IcedElement {
        self.widget.clone().into()
    }
}

impl Drop for GameplayPreview {
    fn drop(&mut self) {
        if let Some(manager) = &mut self.manager {
            manager.on_complete()
        }
    }
}


/// this is the widget that gets added to the ui
#[derive(Clone)]
pub struct GameplayPreviewWidget {
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

impl iced::advanced::Widget<Message, IcedRenderer> for GameplayPreviewWidget {
    fn width(&self) -> iced::Length { self.width }
    fn height(&self) -> iced::Length { self.height }

    fn layout(
        &self,
        _renderer: &IcedRenderer,
        limits: &iced_runtime::core::layout::Limits,
    ) -> iced_runtime::core::layout::Node {
        let limits = limits
            .width(self.width)
            .height(self.height);

        iced_runtime::core::layout::Node::new(limits.fill())
    }

    fn draw(
        &self,
        _state: &iced_runtime::core::widget::Tree,
        renderer: &mut IcedRenderer,
        _theme: &<IcedRenderer as iced_runtime::core::Renderer>::Theme,
        _style: &iced_runtime::core::renderer::Style,
        layout: iced_runtime::core::Layout<'_>,
        _cursor: iced_runtime::core::mouse::Cursor,
        _viewport: &iced::Rectangle,
    ) {
        let _  = self.event_sender.try_send(layout.bounds().into());
        renderer.draw_primitive(iced::advanced::graphics::Primitive::Custom(self.draw_data.lock().read().clone()));
    }
}

impl Into<IcedElement> for GameplayPreviewWidget {
    fn into(self) -> IcedElement {
        IcedElement::new(self)
    }
}
use crate::prelude::*;
use crate::prelude::ui::*;

#[derive(ChainableInitializer)]
pub struct GameplayPreview {
    beatmap: ValueChangeHelper<String>,
    playmode: ValueChangeHelper<String>,
    mods: ValueChangeHelper<ModManager>,
    song_time: ValueChangeHelper<f32>,

    manager: Option<GameplayId>,

    #[chain] pub visualization: Option<MenuVisualization>,

    /// area to fit to
    pub fit_to: Option<Bounds>,

    /// if a song ends, should we handle restarting it?
    pub handle_song_restart: bool,

    /// use bg game settings, or global gamemode?
    use_global_playmode: bool,
    apply_rate: bool,

    widget_sender: Arc<Mutex<TripleBufferSender<Arc<dyn TatakuRenderable>>>>,
    widget_receiver: TripleBufferReceiver<Arc<dyn TatakuRenderable>>,


    #[chain] blur: f32,
    #[chain] style: Style,
    node_id: NodeId,
}
impl GameplayPreview {
    pub fn new(
        use_global_playmode: bool, 
        apply_rate: bool, 
    ) -> Self {
        let a: Arc<dyn TatakuRenderable> = Arc::new(TransformGroup::new(Vector2::ZERO));
        let (widget_sender, widget_receiver) = TripleBuffer::new(&a).split();

        Self {
            // current_mods: ModManagerHelper::new(),
            beatmap: ValueChangeHelper::new("beatmaps.current_beatmap.map.file_path"),
            playmode: ValueChangeHelper::new("global.playmode_actual"),
            mods: ValueChangeHelper::new("global.mods"),
            song_time: ValueChangeHelper::new("song.position"),

            visualization: None,
            handle_song_restart: false,

            manager: None,
            fit_to: None,
            use_global_playmode,
            apply_rate,

            widget_sender: Arc::new(Mutex::new(widget_sender)),
            widget_receiver,

            blur: 0.0,
            style: Style {
                size: Size {
                    width: FILL,
                    height: FILL
                },
                ..Default::default()
            },
            node_id: EMPTY_NODE,
        }
    }

    pub fn setup(
        &mut self, 
        owner: MessageOwner,
        values: &dyn Reflect, 
        actions: &mut ActionQueue
    ) {
        let settings = values
            .reflect_get::<Settings>("settings")
            .unwrap();

        let draw_sender = self.widget_sender.clone();
        actions.push(GameAction::NewGameplayManager(NewManager {
            owner,
            playmode: (!self.use_global_playmode).then(|| settings.background_game_settings.mode.clone()),
            gameplay_mode: Some(GameplayMode::Preview),
            area: self.fit_to,
            draw_function: Some(Arc::new(move |group| {
                let mut lock = draw_sender.lock();
                *lock.input_buffer_mut() = Arc::new(group);
                lock.publish();
            })),

            ..Default::default()
        }));
    }

}
impl Widget for GameplayPreview {
    fn name(&self) -> Cow<'static, str> { "gameplay_preview_widget".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn layout(&mut self, shell: &mut LayoutShell) -> TaffyResult<NodeId> {
        self.node_id = shell.tree.new_leaf(self.style.clone())?;
        Ok(self.node_id)
    }

    fn handle_message(
        &mut self, 
        message: &Message, 
        shell: &mut MessageShell,
    ) {
        let MessageTag::String(str) = &message.tag else { return };
        if str != "gameplay_manager_create" { return }

        let MessageValue::GameplayManagerId(id) = &message.value 
        else { return error!("wrong type") };

        self.manager = Some(id.clone());
        shell.handled = true;

        shell.actions.push(
            GameAction::GameplayAction(id.clone(), GameplayAction::Resume)
        );
    }

    fn update(&mut self, shell: &mut UpdateShell) {
        self.widget_receiver.update();
        let settings = shell.values
            .reflect_get::<Settings>("settings")
            .unwrap();
        
        let last_song_time = self.song_time.unwrap_or_default();
        if let Ok(Some(time)) = self.song_time.update(shell.values) {
            if *time < last_song_time {
                self.setup(shell.owner, shell.values, shell.actions);
            }
        }


        // check for map/mode changes
        let a = self.beatmap.update(shell.values);
        let b = self.playmode.update(shell.values);
        match (a, b) {
            (Ok(Some(_)), _)
            | (_, Ok(Some(_))) => self.setup(shell.owner, shell.values, shell.actions),
            _=> {}
        }

        // check for new bounds
        let bounds = shell.tree.absolute_bounds(self.node_id);
        if let Some(bounds) = bounds {
            if self.fit_to != Some(bounds) {
                // info!("fitting to area {bounds:?}");
                self.fit_to = Some(bounds);

                if let Some(manager) = self.manager.clone() { 
                    shell.actions.push(GameAction::GameplayAction(manager, GameplayAction::FitToArea(bounds)));
                };
            }
        }

        // update vis
        if let Some((vis, bounds)) = self.visualization.as_mut().zip(bounds) {
            vis.update(bounds, shell.actions);
        }

        // check for state update
        if self.handle_song_restart {
            let stopped = shell.values.reflect_get::<bool>("song.stopped")
                .map(|i| *i)
                .unwrap_or_default();
            let playing = shell.values.reflect_get::<bool>("song.playing")
                .map(|i| *i)
                .unwrap_or_default();
            let paused = shell.values.reflect_get::<bool>("song.paused")
                .map(|i| *i)
                .unwrap_or_default();
            let exists = stopped || playing || paused;

            let speed = self
                .mods
                .as_ref()
                .map_or(1.0, |m| m.get_speed());

            if exists {
                if stopped {
                    if let Ok(preview) = shell.values.reflect_get::<f32>("beatmaps.current.map.preview") {
                        shell.actions.push(SongAction::SetPosition(*preview));
                        if self.apply_rate {
                            shell.actions.push(SongAction::SetRate(speed));
                        }

                        shell.actions.push(SongAction::Play);
                    }
                }
            } else {
                let preview_time = shell.values.reflect_get::<f32>("beatmaps.current.map.preview").ok();
                let audio_path = shell.values.reflect_get::<String>("beatmaps.current.map.audio_path").ok();
                
                if let Some((path, preview)) = audio_path.zip(preview_time) {
                    shell.actions.push(SongAction::Set(SongMenuSetAction::FromFile(path.deref().clone(), SongPlayData {
                        play: true,
                        position: Some(*preview),
                        rate: self.apply_rate.then_some(speed),
                        volume: Some(settings.get_music_vol()),

                        ..Default::default()
                    })));
                }
            }
        }
    }

    fn draw(&self, shell: &mut DrawShell) {
        // add gameplay
        shell.list.push_arced(self.widget_receiver.peek_output_buffer().clone());

        if self.blur > 0.0 {
            let bounds = shell.tree.absolute_bounds(self.node_id).unwrap();
            shell.list.push(Blur::new(bounds, self.blur));
        }

        // draw visualization
        if let Some(vis) = &self.visualization {
            vis.draw(shell.list);
        }

        // let bounds = shell.tree.absolute_bounds(self.node_id).unwrap();
        // shell.list.push(Rectangle::new_bounds(bounds, Color::TRANSPARENT_WHITE, Some(Border::new(Color::LIME, 2.0))));
    }


    fn reload_skin(&mut self, shell: &mut UpdateShell) {
        if let Some(vis) = &mut self.visualization {
            debug!("reloading vis skin");
            vis.reload_skin(shell.skin_manager);
        }
    }
}
impl core::fmt::Debug for GameplayPreview {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GameplayPreview")
    }
}

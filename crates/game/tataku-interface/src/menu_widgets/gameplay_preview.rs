use crate::prelude::*;

#[derive(ChainableInitializer)]
pub struct GameplayPreview {
    beatmap: ValueChangeHelper<String>,
    playmode: ValueChangeHelper<String>,
    song_time: ValueChangeHelper<f32>,

    manager: Option<GameplayId>,

    #[chain] visualization: Option<MenuVisualization>,

    /// area to fit to
    fit_to: Option<Bounds>,

    widget_receiver: TripleBufferReceiver<Option<RenderableCollection>>,
    gameplay: Mutex<Option<RenderableCollection>>,

    #[chain] blur: Option<BlurType>,
    node_id: NodeId,
}
impl GameplayPreview {
    pub fn new() -> Self {
        let (_, widget_receiver) = TripleBuffer::default().split();

        Self {
            // current_mods: ModManagerHelper::new(),
            beatmap: ValueChangeHelper::new("beatmaps.current_beatmap.map.file_path"),
            playmode: ValueChangeHelper::new("global.playmode_actual"),
            song_time: ValueChangeHelper::new("song.position"),

            visualization: None,

            manager: None,
            fit_to: None,

            widget_receiver,
            gameplay: Mutex::new(None),

            blur: None,
            node_id: EMPTY_NODE,
        }
    }

    pub fn setup(
        &mut self, 
        owner: MessageOwner,
        _values: &dyn Reflect, 
        actions: &mut ActionQueue
    ) {
        let (widget_sender, widget_receiver) = TripleBuffer::default().split();

        let widget_sender = Mutex::new(widget_sender);

        self.widget_receiver = widget_receiver;
        actions.push(GameAction::NewGameplayManager(NewManager {
            owner,
            playmode: None,
            gameplay_mode: Some(GameplayMode::Preview),
            area: self.fit_to,
            draw_function: Some(Arc::new(move |collection| {
                let Some(mut lock) = widget_sender.try_lock() else { return; };

                *lock.input_buffer_mut() = Some(collection);
                lock.publish();
            })),

            ..Default::default()
        }));
    }

}
impl Widget<TatakuAction> for GameplayPreview {
    fn name(&self) -> CowStr { "gameplay_preview_widget".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn layout(&mut self, shell: &mut LayoutShell<TatakuAction>) -> taffy::TaffyResult<NodeId> {
        self.node_id = shell.tree.new_leaf()?;
        Ok(self.node_id)
    }

    fn handle_message(
        &mut self, 
        message: &Message, 
        shell: &mut MessageShell<TatakuAction>,
    ) {
        if &**message.tag != "gameplay_manager_create" { return }

        let id = message.value.clone().downcast::<u32>();

        self.manager = Some(id.clone());
        shell.handled = true;

        shell.actions.push(
            GameAction::GameplayAction(id.clone(), GameplayAction::Resume)
        );
    }

    fn update(&mut self, shell: &mut UpdateShell<TatakuAction>) {
        self.widget_receiver.update();
        if let Some(gameplay) = self.widget_receiver.output_buffer_mut().take() {
            *self.gameplay.lock() = Some(gameplay);
        }
        
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
                    shell.actions.push(GameAction::GameplayAction(
                        manager, 
                        GameplayAction::FitToArea(bounds)
                    ));
                };
            }
        }

        // update vis
        if let Some((vis, bounds)) = self.visualization.as_mut().zip(bounds) {
            vis.update(bounds, shell.actions);
        }
    }

    fn draw(&self, shell: &mut DrawShell<TatakuAction>) {
        // add gameplay
        if let Some(gameplay) = self.gameplay.lock().take() {
            shell.list.list.extend(gameplay.list);
        }

        if let Some(blur) = self.blur {
            let bounds = shell.tree.absolute_bounds(self.node_id).unwrap();
            shell.list.push(Blur::new(bounds, blur));
        }

        // draw visualization
        if let Some(vis) = &self.visualization {
            vis.draw(shell.list);
        }

        // let bounds = shell.tree.absolute_bounds(self.node_id).unwrap();
        // shell.list.push(Rectangle::new_bounds(bounds, Color::TRANSPARENT_WHITE, Some(Border::new(Color::LIME, 2.0))));
    }


    fn reload_skin(&mut self, shell: &mut UpdateShell<TatakuAction>) {
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

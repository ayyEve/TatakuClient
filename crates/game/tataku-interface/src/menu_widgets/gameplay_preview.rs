use crate::prelude::*;
use common::reflect::*;
use tataku::Bounds;
use graphics::RenderableCollection;
use ui::{
    tree::*,
    widget::*,
    message::*,
};
use engine::{
    triple_buffer,
    data::ValueChangeHelper,
};

pub struct GameplayPreview {
    beatmap: ValueChangeHelper<common::Md5Hash>,
    playmode: ValueChangeHelper<String>,
    song_time: ValueChangeHelper<f32>,

    manager: Option<actions::game::GameplayId>,

    /// area to fit to
    fit_to: Option<Bounds>,

    widget_sender: Arc<Mutex<triple_buffer::Input<Option<RenderableCollection>>>>,
    widget_receiver: triple_buffer::Output<Option<RenderableCollection>>,
    gameplay: Mutex<Option<RenderableCollection>>,
    node_id: NodeId,
}
impl GameplayPreview {
    pub fn new() -> Self {
        let (
            widget_sender,
            widget_receiver
        ) = triple_buffer::TripleBuffer::default().split();

        Self {
            // current_mods: ModManagerHelper::new(),
            beatmap: ValueChangeHelper::new("beatmaps.current_beatmap"),
            playmode: ValueChangeHelper::new("global.playmode_actual"),
            song_time: ValueChangeHelper::new("song.position"),

            manager: None,
            fit_to: None,

            widget_sender: Arc::new(Mutex::new(widget_sender)),
            widget_receiver,
            gameplay: Mutex::new(None),

            node_id: ui::EMPTY_NODE,
        }
    }

    pub fn setup(
        &mut self,
        source: MessageSource,
        _values: &dyn Reflect,
        actions: &mut actions::ActionQueue
    ) {
        let widget_sender = self.widget_sender.clone();
        actions.push(actions::game::GameAction::NewGameplayManager(actions::game::NewManager {
            owner: source,
            playmode: None,
            gameplay_mode: Some(actions::game::GameplayTypeInfo::Preview),
            area: self.fit_to,
            draw_function: Some(Arc::new(move |collection| {
                let Some(mut lock) = widget_sender.try_lock()
                else { return };

                *lock.input_buffer_mut() = Some(collection);
                lock.publish();
            })),

            ..Default::default()
        }).into());
    }

}
impl Widget<actions::Action> for GameplayPreview {
    fn name(&self) -> CowStr { "gameplay_preview_widget".into() }
    fn node_id(&self) -> &NodeId { &self.node_id }

    fn layout(
        &mut self,
        shell: &mut LayoutShell<actions::Action>
    ) -> taffy::TaffyResult<NodeId> {
        self.node_id = shell.tree.new_leaf()?;
        Ok(self.node_id)
    }

    fn handle_message(
        &mut self,
        message: &Message,
        shell: &mut MessageShell<actions::Action>,
    ) {
        if &*message.tag != "gameplay_manager_create" { return }

        let Some(id) = message.value.downcast_ref::<actions::game::GameplayId>() else {
            error!("gameplay_manager_create is not GameplayId");
            return;
        };

        self.manager = Some(id.clone());
        shell.handled = true;

        shell.actions.push(actions::game::GameAction::GameplayAction(
            id.clone(),
            actions::gameplay::GameplayAction::Resume
        ).into());
    }

    fn update(&mut self, shell: &mut UpdateShell<actions::Action>) {
        self.widget_receiver.update();
        if let Some(gameplay) = self.widget_receiver.output_buffer_mut().take() {
            *self.gameplay.lock() = Some(gameplay);
        }

        // check for map/mode changes
        let a = self.beatmap.update(shell.values);
        let b = self.playmode.update(shell.values);

        let time_check = if let Ok(Some(time)) = self.song_time.update(shell.values)
        { *time < self.song_time.unwrap_or_default() } else { false };

        // check if time changed
        if time_check
        || matches!(a, Ok(Some(_)))
        || matches!(b, Ok(Some(_)))
        {
            self.setup(shell.source, shell.values, shell.actions);
        }
        // check for new bounds
        let bounds = shell.tree.absolute_bounds(&self.node_id);
        if let Some(bounds) = bounds
        && self.fit_to != Some(bounds) {
            // info!("fitting to area {bounds:?}");
            self.fit_to = Some(bounds);

            if let Some(manager) = self.manager.clone() {
                shell.actions.push(actions::game::GameAction::GameplayAction(
                    manager,
                    actions::gameplay::GameplayAction::FitToArea(bounds)
                ).into());
            };
        }
    }

    fn draw(&self, shell: &mut DrawShell<actions::Action>) {
        // add gameplay
        if let Some(gameplay) = self.gameplay.lock().take() {
            shell.list.list.extend(gameplay.list);
        }

        // let bounds = shell.tree.absolute_bounds(self.node_id).unwrap();
        // shell.list.push(Rectangle::new_bounds(bounds, Color::TRANSPARENT_WHITE, Some(Border::new(Color::LIME, 2.0))));
    }


}
impl core::fmt::Debug for GameplayPreview {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GameplayPreview")
    }
}

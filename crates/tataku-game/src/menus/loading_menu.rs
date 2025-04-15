use crate::prelude::*;
use crate::prelude::ui::*;

/// helper for when starting the game. will load beatmaps, settings, etc from storage
/// all while providing the user with its progress (relatively anyways)
pub struct LoadingMenu {
    actions: ActionQueue,
    pub statuses: Vec<Arc<RwLock<LoadingStatus>>>,
    node: Box<dyn Widget>,
    node_id: NodeId,
}

impl LoadingMenu {
    pub async fn new() -> Self {
        Self {
            actions: ActionQueue::new(),
            statuses: Vec::new(),
            node: EmptyWidget::new_boxed(),
            node_id: EMPTY_NODE
            // window_size: WindowSize::get(),
        }
    }
    pub async fn load(&mut self, _settings: &Settings) {
        macro_rules! add {
            ($fn: ident, $stage: expr) => {{
                let status = Arc::new(RwLock::new(LoadingStatus::new($stage)));
                self.statuses.push(status.clone());
                tokio::spawn(Self::$fn(status));
            }}
        }

        {
            let status = Arc::new(RwLock::new(LoadingStatus::new("Loading beatmaps")));
            self.actions.push(TaskAction::AddTask(Box::new(LoadBeatmapsTask::new(status.clone()))));
            self.statuses.push(status);
        }

        // init fonts
        add!(init_fonts, "Initializing fonts");
    }

    fn build_view(&self) -> Box<dyn Widget> {
        let elements = self.statuses.iter()
            .map(|status| {
            let status = status.read();

            let text;
            let mut color = Color::BLUE;

            if let Some(error) = &status.error {
                text = "Error: ".to_owned() + error;
                color = Color::RED;
            } else if status.complete {
                text = "Done".to_owned();
                color = Color::LIME;
            } else if !status.custom_message.is_empty() {
                text = status.custom_message.clone();
            } else {
                text = format!("{}/{}", status.items_complete, status.item_count);
            }
            
            row!(
                TextWidget::new(status.name.to_owned() + ": ").text_color(Color::WHITE).boxed(),
                TextWidget::new(text).text_color(color).width(FILL).boxed()
                ;
                width = FILL
            )
        }).collect::<Vec<_>>();

        row!(
            // Space::new(FILL, FILL).boxed(),

            col!(
                elements,
                width = FILL,
                height = FILL
                // spacing = 5.0
            )

            // Space::new(FILL, FILL).boxed()
            ;
            
            width = FILL,
            height = FILL,
            vertical_align = AlignContent::Center
        )
    }
    
    async fn init_fonts(status: Arc<RwLock<LoadingStatus>>) {
        status.write().item_count = 3;

        #[cfg(feature="graphics")]
        preload_fonts();

        status.write().complete = true;
    }
}

#[async_trait]
impl Widget for LoadingMenu {
    fn name(&self) -> Cow<'static, str> { Cow::Borrowed("loading_menu") }
    fn node_id(&self) -> NodeId { self.node_id }

    fn update_styles(&mut self, tree: &mut Tree, resolver: &mut CssResolver, display_override: Option<ui::Display>) {
        self.node.update_styles(tree, resolver, display_override);
    }

    fn layout(&mut self, shell: &mut LayoutShell<'_>) -> TaffyResult<NodeId> {
        self.node = self.build_view();

        let child = self.node.layout(shell)?;
        self.node_id = shell.tree.new_with_children(
            Style::default(), 
            &[child]
        )?;

        Ok(self.node_id)
    }

    fn update(
        &mut self, 
        _shell: &mut UpdateShell<'_>, 
        actions: &mut ActionQueue
    ) {
        actions.extend(self.actions.take());

        for status in self.statuses.iter() {
            let status = status.read();
            if !status.complete { return }
        }

        // loading complete, move to the main menu
        actions.push(BeatmapAction::Next);
        actions.push(MenuAction::set_menu("main_menu"));
    }

    fn draw(
        &self,
        shell: &mut DrawShell<'_>,
    ) {
        self.node.draw(shell);
    }
}


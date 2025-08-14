use crate::prelude::*;

#[derive(ChainableInitializer)]
pub struct TextWidget {
    text: WidgetText,
    node_id: NodeId,
}
impl TextWidget {
    pub fn new(text: impl Into<WidgetText>) -> Self {
        Self {
            text: text.into(),
            node_id: EMPTY_NODE,
        }
    }

    fn min_size(&self, tree: &Tree<TatakuAction>) -> [CssUnit; 2] {
        let text_size = tree
            .get_text_style(self.node_id)
            .unwrap()
            .measure_text(&self.text.get(), None)
            ;

        [
            CssUnit::Pixels(half::f16::from_f32(text_size.x)),
            CssUnit::Pixels(half::f16::from_f32(text_size.y)),
        ]
    }
}
impl Widget<TatakuAction> for TextWidget {
    fn name(&self) -> CowStr { "text_widget".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn layout(&mut self, shell: &mut LayoutShell<TatakuAction>) -> taffy::TaffyResult<NodeId> {
        // self.text_style.font_size *= shell.ui_scale;
        self.node_id = shell.tree.new_leaf()?;
        Ok(self.node_id)
    }
    fn init_style(&mut self, shell: &mut LayoutShell<TatakuAction>) {
        let min_size = self.min_size(shell.tree);
        shell.tree.update_style(self.node_id, |style| {
            style.min_width = min_size[0].into();
            style.min_height = min_size[1].into();
        });
    }

    fn update(&mut self, shell: &mut UpdateShell<TatakuAction>) {
        if self.text.update(shell.values) {
            let min = self.min_size(shell.tree);
            shell.actions.push(UiAction::new(
                self.node_id, 
                UiActionType::UpdateStyleWith(Arc::new(
                    move |style| {
                        style.min_width = min[0].into();
                        style.min_height = min[1].into();
                    }
                ))
            ));
            shell.actions.push(UiAction::new(
                self.node_id, 
                UiActionType::MarkDirty,
            ));
        }

        // self.text_style = shell
        //     .tree
        //     .get_context(self.node_id)
        //     .unwrap()
        //     .element_data
        //     .style()
        //     .0
        //     .text_style(shell.values);
    }
    
    fn draw(&self, shell: &mut DrawShell<TatakuAction>) {
        let Some(bounds) = shell.tree.absolute_bounds(self.node_id) 
        else { return };
        // let Some(ctx) = shell.tree.get_context(self.node_id) else { return };

        let text_style = shell.tree
            .get_text_style(self.node_id)
            .unwrap();
        
        let text = self.text.get();
        shell.list.push(text_style.create_text(
            text.into_owned(), 
            bounds
        ));
    }
}


// TODO: rename?
pub enum WidgetText {
    String(CowStr),
    Custom {
        custom: BuildableText,
        cached: String,
    },
}
impl WidgetText {
    pub fn get(&self) -> Cow<'_, str> {
        match self {
            Self::String(Cow::Borrowed(s)) => Cow::Borrowed(*s),
            Self::String(Cow::Owned(s)) => Cow::Borrowed(s),
            Self::Custom { cached, .. } => Cow::Borrowed(cached),
        }
    }
    pub fn set(&mut self, value: String) {
        match self {
            Self::String(cow) => *cow = Cow::Owned(value),
            Self::Custom { cached, .. } => *cached = value,
        }
    }

    pub fn update(
        &mut self,
        values: &dyn Reflect
    ) -> bool {
        let Self::Custom { custom, cached } = self else { return false };
        let new = custom.to_string(values);
        if *cached != new {
            *cached = new;
            true
        } else {
            false
        }
    }
}
impl From<&str> for WidgetText {
    fn from(value: &str) -> Self {
        Self::String(Cow::Owned(value.to_owned()))
    }
}
impl From<String> for WidgetText {
    fn from(value: String) -> Self {
        Self::String(value.into())
    }
}
impl From<BuildableText> for WidgetText {
    fn from(mut value: BuildableText) -> Self {
        if let Err(e) = value.compute() {
            error!("error parsing CustomElementText: {e:?}");
        }

        Self::Custom {
            custom: value,
            cached: String::new()
        }
    }
}

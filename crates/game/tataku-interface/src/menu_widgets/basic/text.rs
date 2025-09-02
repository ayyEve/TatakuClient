use crate::prelude::*;

#[derive(ChainableInitializer)]
pub struct TextWidget {
    text: WidgetText,
    node_id: NodeId,

    layout: parley::Layout<Color>,
    old_x: f32,
    old_width: f32,
}
impl TextWidget {
    pub fn new(text: impl Into<WidgetText>) -> Self {
        Self {
            text: text.into(),
            node_id: EMPTY_NODE,

            layout: parley::Layout::default(),
            old_x: 0.0,
            old_width: 0.0,
        }
    }

    fn recreate_layout(
        &mut self,
        tree: &mut Tree<TatakuAction>,
        text_layout_contexts: &mut TextLayoutContexts,
    ) {
        let text = self.text.get();
        let text_style = tree.get_text_style(self.node_id).unwrap();

        self.layout = text_layout_contexts.simple_text(
            &text,
            text_style,
        );

        if !text.is_empty() {
            let widths = self.layout.calculate_content_widths();

            tree.update_style(self.node_id, |style| {
                // fixme: there is an off-by-one somewhere
                style.min_width = CssUnit::Pixels(f16::from_f32(widths.min.ceil() + 1.0)).into();
                style.max_width = CssUnit::Pixels(f16::from_f32(widths.max.ceil() + 1.0)).into();
            });
        }
    }

    fn wrap_and_align(
        &mut self,
        tree: &mut Tree<TatakuAction>,
        container_width: f32,
    ) {
        let text = self.text.get();
        let text_style = tree.get_text_style(self.node_id).unwrap();

        if !text.is_empty() {
            self.layout.break_all_lines(Some(container_width));

            let alignment = match text_style.alignment {
                HorizontalAlign::Left => parley::Alignment::Start,
                HorizontalAlign::Center => parley::Alignment::Middle,
                HorizontalAlign::Right => parley::Alignment::End,
            };

            self.layout.align(
                None,
                alignment,
                parley::AlignmentOptions::default(),
            );

            let height = self.layout.height();

            tree.update_style(self.node_id, |style| {
                style.height = CssUnit::Pixels(f16::from_f32(height)).into();
            });
        }
    }
}
impl Widget<TatakuAction> for TextWidget {
    fn name(&self) -> CowStr { "text_widget".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn layout(
        &mut self,
        shell: &mut LayoutShell<TatakuAction>
    ) -> taffy::TaffyResult<NodeId> {
        self.node_id = shell.tree.new_leaf()?;
        Ok(self.node_id)
    }
    fn init_style(&mut self, shell: &mut LayoutShell<TatakuAction>) {
        self.recreate_layout(
            shell.tree,
            shell.text_layout_contexts
        );

        let text_style = shell.tree
            .get_text_style(self.node_id)
            .unwrap();
        let min_height = text_style.line_height;

        self.wrap_and_align(
            shell.tree,
            f32::MAX,
        );

        shell.tree.update_style(self.node_id, |style| {
            style.min_height = CssUnit::Pixels(f16::from_f32(min_height)).into();
        });
    }

    fn update(&mut self, shell: &mut UpdateShell<TatakuAction>) {
        let bounds = shell.tree.absolute_bounds(self.node_id).unwrap();

        let refresh_text = self.text.update(shell.values);

        if refresh_text {
            self.recreate_layout(
                shell.tree,
                shell.text_layout_contexts,
            );
        }

        let refresh_layout = refresh_text
            || bounds.pos.x != self.old_x
            || bounds.size.x != self.old_width;

        if refresh_layout {
            self.old_x = bounds.pos.x;
            self.old_width = bounds.size.x;

            self.wrap_and_align(
                shell.tree,
                bounds.size.x,
            );
        }
    }

    fn draw(&self, shell: &mut DrawShell<TatakuAction>) {
        // todo: handle this better
        let bounds = shell.tree.bounds(self.node_id).unwrap();
        let context = shell.tree.get_context(self.node_id).unwrap();
        let transform = context.global_transform
            * context.local_transform.matrix()
            * Matrix::identity().trans(bounds.pos);

        shell.list.push(Transformed {
            transform,
            drawable: Box::new(Text::new(self.layout.clone())),
        });
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

        match value {
            BuildableText::Text { text } => Self::String(text.to_string().into()),
            custom => Self::Custom {
                custom,
                cached: String::new()
            }
        }
    }
}

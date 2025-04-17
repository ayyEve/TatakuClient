use crate::prelude::*;
use crate::prelude::ui::*;

#[derive(ChainableInitializer)]
#[derive(Widget)]
#[widget(type("text"))]
pub struct TextWidget {
    #[chain] pub style: Style,
    #[chain] pub text_style: TextStyle,
    
    pub text: WidgetText,

    node_id: NodeId,
}
impl TextWidget {
    pub fn new(text: impl Into<WidgetText>) -> Self {
        Self {
            text: text.into(),
            style: Style::DEFAULT,
            text_style: TextStyle::default(),
            node_id: EMPTY_NODE,
        }
    }

    fn min_size(&self, tree: &Tree) -> Size<Dimension> {
        let text_size = tree.get_context(self.node_id).unwrap()
            .element_data.style()
            .0.text_style()
            .measure_text(&self.text.get(), None)
            ;

        Size {
            width: Dimension::Length(text_size.x),
            height: Dimension::Length(text_size.y),
        }
    }
}
impl Widget for TextWidget {
    fn name(&self) -> Cow<'static, str> { "text_widget".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn update_styles(&mut self, tree: &mut Tree, _resolver: &mut CssResolver, _display_override: Option<ui::Display>) {
        let mut style = tree.get_style(self.node_id).unwrap().clone();
        style.min_size = self.min_size(tree);
        tree.set_style(self.node_id, style);
    }
    // fn set_text_style(&mut self, style: TextStyle) {
    //     self.text_style = style;
    // }

    fn layout(&mut self, shell: &mut LayoutShell<'_>) -> TaffyResult<NodeId> {
        // self.text_style.font_size *= shell.ui_scale;
        self.node_id = shell.tree.new_leaf(Style::default())?;
        Ok(self.node_id)
    }

    fn update(
        &mut self, 
        shell: &mut UpdateShell<'_>, 
        actions: &mut ActionQueue
    ) {
        if self.text.update(shell.values) {
            let min = self.min_size(shell.tree);
            actions.push(UiAction::new(self.node_id, UiActionType::UpdateStyleWith(Box::new(move |style| style.min_size = min))));
            actions.push(UiAction::new(self.node_id, UiActionType::MarkDirty));
        }
    }
    
    fn draw(
        &self, 
        shell: &mut DrawShell<'_>, 
    ) {
        let Some(bounds) = shell.tree.absolute_bounds(self) else { return };
        let Some(ctx) = shell.tree.get_context(self.node_id) else { return };
        let style = ctx.element_data.style().0.text_style();

        shell.list.push(style.create_text(self.text.get().clone().into_owned(), bounds));
    }
}


// TODO: rename?
pub enum WidgetText {
    String(Cow<'static, str>),
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
            WidgetText::String(cow) => *cow = Cow::Owned(value),
            WidgetText::Custom { cached, .. } => *cached = value,
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
impl From<&'static str> for WidgetText {
    fn from(value: &'static str) -> Self {
        Self::String(value.into())
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
impl From<TextBuilderValue> for WidgetText {
    fn from(value: TextBuilderValue) -> Self {
        let val: BuildableText = value.into();
        val.into()
    }
}

impl From<TextBuilderValue> for BuildableTextInner {
    fn from(value: TextBuilderValue) -> Self {
        match value {
            TextBuilderValue::Static(s) => BuildableTextInner::Text(s),
            TextBuilderValue::Variable(v) => BuildableTextInner::Variable(v),
            TextBuilderValue::Calc(c) => BuildableTextInner::Calc(c),
            TextBuilderValue::List(_list, _join) => panic!("nested list is unsupported!"),
        }
    }
}
impl From<TextBuilderValue> for BuildableText {
    fn from(value: TextBuilderValue) -> Self {
        match value {
            TextBuilderValue::Static(s) => BuildableTextInner::Text(s).into(),
            TextBuilderValue::Variable(v) => BuildableTextInner::Variable(v).into(),
            TextBuilderValue::Calc(c) => BuildableTextInner::Calc(c).into(),
            TextBuilderValue::List(list, join) => BuildableText {
                join: Some(join),
                text: list.into_iter().map(|i| i.into()).collect()
            },
        }
    }
}

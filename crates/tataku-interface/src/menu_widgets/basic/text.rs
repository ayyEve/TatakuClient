use crate::prelude::*;
use crate::prelude::ui::*;

#[derive(ChainableInitializer)]
#[derive(Widget)]
#[widget(type("text"))]
pub struct TextWidget {
    #[chain] style: Style,
    #[chain] text_style: TextStyle,
    
    text: WidgetText,

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

    fn min_size(&self, tree: &Tree, values: &dyn Reflect) -> Size<Dimension> {
        let text_size = tree.get_context(self.node_id).unwrap()
            .element_data.style()
            .0.text_style(values)
            .measure_text(&self.text.get(), None)
            ;

        Size {
            width: Dimension::Length(text_size.x),
            height: Dimension::Length(text_size.y),
        }
    }
}
impl Widget for TextWidget {
    fn name(&self) -> CowStr { "text_widget".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn update_styles(
        &mut self, 
        shell: &mut StyleShell, 
        _display_override: Option<ui::Display>
    ) {
        let mut style = shell.tree.get_style(self.node_id).unwrap().clone();
        style.min_size = self.min_size(shell.tree, shell.values);
        shell.tree.set_style(self.node_id, style);
    }

    fn set_text_style(&mut self, style: TextStyle) {
        self.text_style = style;
    }

    fn layout(&mut self, shell: &mut LayoutShell) -> TaffyResult<NodeId> {
        // self.text_style.font_size *= shell.ui_scale;
        self.node_id = shell.tree.new_leaf(Style::default())?;
        Ok(self.node_id)
    }

    fn update(&mut self, shell: &mut UpdateShell) {
        if self.text.update(shell.values) {
            let min = self.min_size(shell.tree, shell.values);
            shell.actions.push(UiAction::new(
                self.node_id, 
                UiActionType::UpdateStyleWith(Box::new(
                    move |style| style.min_size = min
                ))
            ));
            shell.actions.push(UiAction::new(
                self.node_id, 
                UiActionType::MarkDirty,
            ));
        }

        self.text_style = shell
            .tree
            .get_context(self.node_id)
            .unwrap()
            .element_data
            .style()
            .0
            .text_style(shell.values);
    }
    
    fn draw(&self, shell: &mut DrawShell) {
        let Some(bounds) = shell.tree.absolute_bounds(self) else { return };
        // let Some(ctx) = shell.tree.get_context(self.node_id) else { return };
        
        let text = self.text.get();
        shell.list.push(self.text_style.create_text(
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
impl From<TextBuilderValue> for WidgetText {
    fn from(value: TextBuilderValue) -> Self {
        let val: BuildableText = value.into();
        val.into()
    }
}

impl From<TextBuilderValue> for BuildableText {
    fn from(value: TextBuilderValue) -> Self {
        match value {
            TextBuilderValue::Static(text) => BuildableText::Text { text },
            TextBuilderValue::Variable(variable) => BuildableText::Variable { 
                variable
            },
            TextBuilderValue::Calc(c) => BuildableText::Calc { 
                calc: Some(c), 
                var: None 
            },
            TextBuilderValue::List(
                list, 
                join
            ) => BuildableText::List {
                join,
                list: list.into_iter().map(|i| i.into()).collect()
            },
        }
    }
}

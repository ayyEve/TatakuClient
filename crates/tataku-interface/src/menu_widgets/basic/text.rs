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

    fn get_style(&self) -> Style {
        let text_size = self.text_style
            .measure_text(&self.text.get(), None)
            ;
        
        Style {
            min_size: Size {
                width: Dimension::Length(text_size.x),
                height: Dimension::Length(text_size.y),
            },

            ..self.style.clone()
        }
    }
}


#[async_trait]
impl Widget for TextWidget {
    fn name(&self) -> Cow<'static, str> { "text_widget".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn layout(&mut self, shell: &mut LayoutShell<'_>) -> TaffyResult<NodeId> {
        self.text_style.font_size *= shell.ui_scale;
        self.node_id = shell.tree.new_leaf(self.get_style())?;
        Ok(self.node_id)
    }

    fn update(
        &mut self, 
        shell: &mut UpdateShell<'_>, 
        actions: &mut ActionQueue
    ) {
        if self.text.update(shell.values) {
            actions.push(UiAction::new(self.node_id, UiActionType::UpdateStyle(Box::new(self.get_style()))));
            actions.push(UiAction::new(self.node_id, UiActionType::MarkDirty));
        }
    }
    
    fn draw(
        &self, 
        shell: &mut DrawShell<'_>, 
    ) {
        let Some(bounds) = shell.tree.absolute_bounds(self) else { return };
        shell.list.push(self.text_style.create_text(self.text.get().clone().into_owned(), bounds));
    }
}


#[derive(Clone)]
pub struct TextStyle {
    pub font: Font,
    pub font_size: f32,
    pub color: Color,
    pub line_height: f32,

    pub alignment: Alignment,
}
impl TextStyle {
    pub fn measure_text(
        &self, 
        text: &str, 
        scale: Option<Vector2>
    ) -> Vector2 {
        Text::measure_text_raw(
            &[self.font],
            self.font_size,
            text,
            scale.unwrap_or(Vector2::ONE),
            self.line_height - self.font_size
        )
    }

    /// create and layout some text within the provided bounds
    pub fn create_text(&self, text: String, bounds: Bounds) -> Text {
        let mut text = Text::new(
            Vector2::ZERO,
            self.font_size,
            text,
            self.color,
            self.font
        );
        text.line_spacing = self.line_height - self.font_size;

        let offset = self.alignment.resolve(
            &bounds, 
            text.measure_text(), 
            true, 
            true
        );

        text.pos = offset;

        text
    }

}

impl Default for TextStyle {
    fn default() -> Self {
        Self { 
            font: Font::Main, 
            font_size: 32.0, 
            color: Color::WHITE, 

            // idk what a sane default for this is
            line_height: 32.0,

            alignment: Alignment::CENTER_LEFT,
        }
    }
}

// TODO: rename?
pub enum WidgetText {
    String(Cow<'static, str>),
    Custom {
        custom: CustomElementText,
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
impl From<CustomElementText> for WidgetText {
    fn from(mut value: CustomElementText) -> Self {
        if let Err(e) = value.parse() {
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
        let val: CustomElementText = value.into();
        val.into()
    }
}

impl From<TextBuilderValue> for CustomElementText {
    fn from(value: TextBuilderValue) -> Self {
        match value {
            TextBuilderValue::Static(s) => Self::Text(s),
            TextBuilderValue::Variable(v) => Self::Variable(v),
            TextBuilderValue::Calc(c) => Self::Calc(c),
            TextBuilderValue::List(list, join) => 
                Self::List(list.into_iter().map(|i| i.into()).collect(), join),
        }
    }
}
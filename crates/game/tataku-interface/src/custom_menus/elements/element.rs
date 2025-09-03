use crate::prelude::*;

pub trait CustomElement {
    fn as_element(&self) -> Option<&dyn CustomElement> { None }
    fn build(&self) -> Box<dyn Widget<TatakuAction>>;
}

#[derive(Deserialize)]
#[serde(from="String")]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ClassList(pub Vec<ArcStr>);
impl ClassList {
    pub fn push(&mut self, s: impl Into<ArcStr>) {
        self.0.push(s.into());
    }
}
impl From<String> for ClassList {
    fn from(value: String) -> Self {
        Self(value
            .split(" ")
            .map(|i| i.trim().to_owned())
            .filter(|i| !i.is_empty())
            .map(ArcStr::from)
            .collect()
        )
    }
}
impl From<ArcStr> for ClassList {
    fn from(value: ArcStr) -> Self {
        Self(value
            .split(" ")
            .map(|i| i.trim().to_owned())
            .filter(|i| !i.is_empty())
            .map(ArcStr::from)
            .collect()
        )
    }
}
impl From<&str> for ClassList {
    fn from(value: &str) -> Self {
        value.to_owned().into()
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Default, PartialEq)]
pub enum Element {
    #[default] Empty,

    Row(Box<RowElement>),
    List(Box<ListElement>),
    Column(Box<ColumnElement>),
    Switch(Box<SwitchElement>),
    Animatable(Box<AnimatableElement>),
    #[serde(alias="cond", alias="if")]
    Conditional(Box<ConditionalElement>),

    Text(Box<TextElement>),
    GameplayPreview(Box<GameplayPreviewElement>),

    Button(Box<ButtonElement>),
    Slider(Box<SliderElement>),
    Checkbox(Box<CheckboxElement>),
    TextInput(Box<TextInputElement>),
    KeyButton(Box<KeyButtonElement>),
    GamepadButton(Box<GamepadButtonElement>),
    Dropdown(Box<DropdownElement>),
}
impl CustomElement for Element {
    fn as_element(&self) -> Option<&dyn CustomElement> {
        match self {
            Self::Empty => None,
            Self::Row(e) => Some(&**e as &dyn CustomElement),
            Self::List(e) => Some(&**e as &dyn CustomElement),
            Self::Column(e) => Some(&**e as &dyn CustomElement),
            Self::Switch(e) => Some(&**e as &dyn CustomElement),
            Self::Animatable(e) => Some(&**e as &dyn CustomElement),
            Self::Conditional(e) => Some(&**e as &dyn CustomElement),
            Self::Text(e) => Some(&**e as &dyn CustomElement),
            Self::GameplayPreview(e) => Some(&**e as &dyn CustomElement),
            Self::Slider(e) => Some(&**e as &dyn CustomElement),
            Self::Button(e) => Some(&**e as &dyn CustomElement),
            Self::Checkbox(e) => Some(&**e as &dyn CustomElement),
            Self::TextInput(e) => Some(&**e as &dyn CustomElement),
            Self::KeyButton(e) => Some(&**e as &dyn CustomElement),
            Self::GamepadButton(e) => Some(&**e as &dyn CustomElement),
            Self::Dropdown(e) => Some(&**e as &dyn CustomElement),
        }
    }
    fn build(&self) -> Box<dyn Widget<TatakuAction>> {
        match self {
            Self::Empty => EmptyWidget::new_boxed(),
            other => other.as_element().unwrap().build(),
        }
    }
}

// mainly used for testing
impl From<TextElement> for Element {
    fn from(value: TextElement) -> Self {
        Self::Text(Box::new(value))
    }
}

#[derive(Deserialize)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Wrapped<T> {
    #[serde(rename="$value")]
    pub inner: T
}

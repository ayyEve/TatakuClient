use crate::prelude::*;
use ui::widget::Widget;
use crate::custom_menus::elements;

pub trait CustomElement {
    fn as_element(&self) -> Option<&dyn CustomElement> { None }
    fn build(&self) -> Box<dyn Widget<actions::Action>>;
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

    Row(Box<elements::RowElement>),
    List(Box<elements::ListElement>),
    Column(Box<elements::ColumnElement>),
    Switch(Box<elements::SwitchElement>),
    Section(Box<elements::SectionElement>),

    Animatable(Box<elements::AnimatableElement>),
    #[serde(alias="cond", alias="if")]
    Conditional(Box<elements::ConditionalElement>),

    Text(Box<elements::TextElement>),
    GameplayPreview(Box<elements::GameplayPreviewElement>),

    Button(Box<elements::ButtonElement>),
    Slider(Box<elements::SliderElement>),
    Checkbox(Box<elements::CheckboxElement>),
    TextInput(Box<elements::TextInputElement>),
    KeyButton(Box<elements::KeyButtonElement>),
    GamepadButton(Box<elements::GamepadButtonElement>),
    Dropdown(Box<elements::DropdownElement>),
    Visualization(Box<elements::VisualizationElement>),
}
impl CustomElement for Element {
    fn as_element(&self) -> Option<&dyn CustomElement> {
        macro_rules! impl_as_element {
            ($($i: ident),*$(,)?) => {
                match self {
                    Self::Empty => None,
                    $(
                        Self::$i(e) => Some(&**e as &dyn CustomElement),
                    )*
                }
            }
        }
        
        impl_as_element!(
            Row,
            Column,
            Section,
            
            List,
            Switch,
            Animatable,
            Conditional,
            Text,
            TextInput,
            GameplayPreview,
            GamepadButton,
            Slider,
            Button,
            KeyButton,
            Dropdown,
            Checkbox,
            Visualization,
        )
    }
    fn build(&self) -> Box<dyn Widget<actions::Action>> {
        match self {
            Self::Empty => ui::widget::EmptyWidget::new_boxed(),
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


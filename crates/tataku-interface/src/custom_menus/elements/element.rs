use crate::prelude::*;

pub trait CustomElement {
    fn as_element(&self) -> Option<&dyn CustomElement> { None }
    fn build(&self) -> Box<dyn Widget>;
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[derive(Deserialize)]
#[serde(from="String")]
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

#[derive(Clone, Debug, Default, PartialEq)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Element {
    #[default] Empty,

    Row(Box<RowElement>),
    List(Box<ListElement>),
    Column(Box<ColumnElement>),
    Switch(Box<SwitchElement>),
    Animatable(Box<AnimatableElement>),
    #[serde(alias="cond")]
    Conditional(Box<ConditionalElement>),

    Text(Box<TextElement>),
    GameplayPreview(Box<GameplayPreviewElement>),

    Button(Box<ButtonElement>),
    Slider(Box<SliderElement>),
    Checkbox(Box<CheckboxElement>),
    TextInput(Box<TextInputElement>),
    KeyButton(Box<KeyButtonElement>),
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
            Self::Dropdown(e) => Some(&**e as &dyn CustomElement),
        }
    }
    fn build(&self) -> Box<dyn Widget> {
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


#[derive(Clone, Debug, Default, PartialEq)]
#[derive(Deserialize)]
pub struct ElementTag {
    #[serde(rename="$value")] pub element: Element
}
crate::impl_tag!(ElementTag, Element, element);

#[derive(Clone, Debug, Default, PartialEq)]
#[derive(Deserialize)]
pub struct ElementList {
    #[serde(rename="$value")] pub list: Vec<Element>,
}
impl ElementList {
    pub fn build(&self) -> Vec<Box<dyn Widget>> {
        self.list
            .iter()
            .map(|i| i.build())
            .collect()
    }
}
crate::impl_tag!(ElementList, Vec<Element>, list);

#[macro_export]
macro_rules! impl_tag {
    ($struct: ident, $ty: ty, $field: ident) => {
        impl $struct {
            #[allow(unused)]
            pub fn new($field: impl Into<$ty>) -> Self {
                Self { $field: $field.into() }
            }
        }

        impl Deref for $struct {
            type Target = $ty;
            fn deref(&self) -> &$ty {
                &self.$field
            }
        }
        impl DerefMut for $struct {
            fn deref_mut(&mut self) -> &mut $ty {
                &mut self.$field
            }
        }
        impl From<$ty> for $struct {
            fn from(value: $ty) -> Self {
                Self {
                    $field: value
                }
            }
        }
    }
}

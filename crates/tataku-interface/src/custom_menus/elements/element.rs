use crate::prelude::*;

pub trait CustomElement {
    fn build(&self) -> Box<dyn Widget>;
    fn boxed(self) -> Box<dyn CustomElement> where Self:Sized + 'static {
        Box::new(self)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[derive(Deserialize)]
#[serde(from="String")]
pub struct ClassList(pub Vec<String>);
impl ClassList {
    pub fn push(&mut self, s: impl Into<String>) {
        self.0.push(s.into());
    }
}
impl From<String> for ClassList {
    fn from(value: String) -> Self {
        Self(value
            .split(" ")
            .map(|i| i.trim().to_owned())
            .filter(|i| !i.is_empty())
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
    Column(Box<ColumnElement>),
    Button(Box<ButtonElement>),
    Checkbox(Box<CheckboxElement>),

    GameplayPreview(Box<GameplayPreviewElement>),
    Text(Box<TextElement>),
    TextInput(Box<TextInputElement>),
    Animatable(Box<AnimatableElement>),

    Dropdown(Box<DropdownElement>),
    List(Box<ListElement>),

    #[serde(alias="cond")]
    Conditional(Box<ConditionalElement>),
}
impl CustomElement for Element {
    fn build(&self) -> Box<dyn Widget> {
        match self {
            Self::Empty => EmptyWidget::new_boxed(),
            Self::Row(e) => e.build(),
            Self::Column(e) => e.build(),
            Self::Button(e) => e.build(),
            Self::Checkbox(e) => e.build(),
            Self::GameplayPreview(e) => e.build(),
            Self::Text(e) => e.build(),
            Self::TextInput(e) => e.build(),
            Self::Animatable(e) => e.build(),
            Self::Conditional(e) => e.build(),
            Self::List(e) => e.build(),
            Self::Dropdown(e) => e.build(),
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

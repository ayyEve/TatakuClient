use crate::prelude::*;

pub trait CustomElement {
    fn build(&self, shell: &mut ElementBuildShell<'_>) -> Box<dyn Widget>;
    fn boxed(self) -> Box<dyn CustomElement> where Self:Sized + 'static {
        Box::new(self)
    }
}

/// might want to add more to this in the future so i made it easy for myself
pub struct ElementBuildShell<'a> {
    pub owner: MessageOwner,
    pub _empty: std::marker::PhantomData<&'a ()>
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[derive(Deserialize)]
#[serde(from="String")]
pub struct ClassList(pub Vec<String>);
impl ClassList {
    pub fn push(&mut self, s: impl Into<String>) {
        self.0.push(s.into())
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
    fn build(&self, shell: &mut ElementBuildShell<'_>) -> Box<dyn Widget> {
        match self {
            Self::Empty => EmptyWidget::new_boxed(),
            Self::Row(e) => e.build(shell),
            Self::Column(e) => e.build(shell),
            Self::Button(e) => e.build(shell),
            Self::GameplayPreview(e) => e.build(shell),
            Self::Text(e) => e.build(shell),
            Self::TextInput(e) => e.build(shell),
            Self::Animatable(e) => e.build(shell),
            Self::Conditional(e) => e.build(shell),
            Self::List(e) => e.build(shell),
            Self::Dropdown(e) => e.build(shell),
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
    pub fn build(&self, shell: &mut ElementBuildShell<'_>) -> Vec<Box<dyn Widget>> {
        self.list
            .iter()
            .map(|i| i.build(shell))
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

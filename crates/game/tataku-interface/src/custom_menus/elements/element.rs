use crate::prelude::*;
use ui::widget::Widget;
use crate::custom_menus::elements;

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

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Element {
    pub name: ArcStr,
    pub id: Option<ArcStr>,
    pub class_list: ClassList,
    pub style: ArcStr,

    pub inner: ElementType,
}

impl<'de> Deserialize<'de> for Element {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        use serde::de::{
            self, DeserializeSeed,
            VariantAccess, EnumAccess,
            MapAccess, Error,
            value::{
                EnumAccessDeserializer,
                MapAccessDeserializer,
                StrDeserializer,
                UnitDeserializer,
            }
        };

        struct Enum<'name, 'de: 'name, A>
        where
            A: MapAccess<'de>
        {
            name: &'name str,
            attributes: Vec<(String, tataku::FromString<'static>)>,
            next_key: Option<String>,
            current_attribute: usize,
            map: Option<A>,
            lifetime: std::marker::PhantomData<&'de ()>,
        }
        impl<'name, 'de: 'name, A> EnumAccess<'de> for Enum<'name, 'de, A>
        where
            A: MapAccess<'de>
        {
            type Error = A::Error;
            type Variant = Self;

            fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
            where
                V: DeserializeSeed<'de>
            {
                Ok((
                    seed.deserialize(StrDeserializer::new(self.name))?,
                    self,
                ))
            }
        }
        impl<'name, 'de: 'name, A> VariantAccess<'de> for Enum<'name, 'de, A>
        where
            A: MapAccess<'de>
        {
            type Error = A::Error;

            fn unit_variant(self) -> Result<(), Self::Error> {
                debug_assert!(self.attributes.is_empty(), "unit variants can't have attributes");
                Ok(())
            }

            fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
            where
                T: DeserializeSeed<'de>
            {
                if self.attributes.is_empty() && self.map.is_none() {
                    seed.deserialize(UnitDeserializer::new())
                } else {
                    seed.deserialize(MapAccessDeserializer::new(self))
                }
            }

            fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
            where
                V: serde::de::Visitor<'de>
            {
                unimplemented!()
            }

            fn struct_variant<V>(
                self,
                _fields: &'static [&'static str],
                _visitor: V,
            ) -> Result<V::Value, Self::Error>
            where
                V: serde::de::Visitor<'de>
            {
                unimplemented!("struct");
            }
        }
        impl<'name, 'de: 'name, A> MapAccess<'de> for Enum<'name, 'de, A>
        where
            A: MapAccess<'de>
        {
            type Error = A::Error;

            fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
            where
                K: DeserializeSeed<'de>
            {
                if let Some((attribute, _)) = self.attributes.get(self.current_attribute) {
                    seed.deserialize(StrDeserializer::new(attribute))
                        .map(Some)
                } else if let Some(map) = &mut self.map {
                    // Avoid running `next_key_seed` twice on the inner map
                    // to prevent panic from debug_assert
                    if let Some(next_key) = self.next_key.take() {
                        seed.deserialize(StrDeserializer::new(&next_key))
                            .map(Some)
                    } else {
                        map.next_key_seed(seed)
                    }
                } else {
                    Ok(None)
                }
            }

            fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
            where
                V: DeserializeSeed<'de>
            {
                if let Some((_, value)) = self.attributes.get_mut(self.current_attribute) {
                    self.current_attribute += 1;
                    let value = std::mem::replace(value, tataku::FromString::EMPTY);

                    seed.deserialize(value)
                        .map_err(Error::custom)
                } else if let Some(map) = &mut self.map {
                    map.next_value_seed(seed)
                } else {
                    unreachable!("called next_value_seed despite no next_key");
                }
            }
        }
        #[derive(Default)]
        struct Visitor {
            name: String,
        }
        impl<'de> de::Visitor<'de> for Visitor {
            type Value = Element;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "an element")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>
            {
                let mut id = None;
                let mut class_list = None;
                let mut style = None;

                let mut attributes = Vec::new();
                let mut next_key = None;

                while let Some(key) = map.next_key::<String>()? {
                    match &*key {
                        "@id" => {
                            if id.is_some() {
                                return Err(Error::duplicate_field("@id"));
                            }

                            id = Some(map.next_value()?);
                        },
                        "@class" => {
                            if class_list.is_some() {
                                return Err(Error::duplicate_field("@class"));
                            }

                            class_list = Some(map.next_value()?);
                        },
                        "@style" => {
                            if style.is_some() {
                                return Err(Error::duplicate_field("@style"));
                            }

                            style = Some(map.next_value()?);
                        },
                        _ => {
                            if &key[..1] == "@" {
                                // attributes have to be deserializable as strings
                                let value: String = map.next_value()?;

                                attributes.push((key, tataku::FromString::from(value)));
                            } else {
                                next_key = Some(key);
                                break;
                            }
                        }
                    }
                }

                // if there is no next key, then the tag end has been consumed.
                // avoid subsequent calls as they will peak the next token.
                let map = next_key.is_some().then_some(map);

                let Self { name } = self;

                let element = {
                    let de = EnumAccessDeserializer::new(Enum {
                        name: &name,
                        attributes,
                        next_key,
                        current_attribute: 0,
                        map,
                        lifetime: std::marker::PhantomData,
                    });
                    ElementType::deserialize(de)?
                };

                Ok(Element {
                    name: name.into(),
                    id,
                    class_list: class_list.unwrap_or_default(),
                    style: style.unwrap_or_default(),
                    inner: element,
                })
            }

            fn visit_enum<A>(mut self, data: A) -> Result<Self::Value, A::Error>
            where
                A: de::EnumAccess<'de>
            {
                let (name, variant): (String, _) = data.variant()?;

                self.name = name;

                struct QueryFields<'a, E> {
                    name: &'a str,
                    fields: Option<&'static [&'static str]>,
                    error: std::marker::PhantomData<E>,
                }
                impl<'de, E> serde::Deserializer<'de> for &mut QueryFields<'de, E>
                where
                    E: Error
                {
                    type Error = E;

                    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
                    where
                        V: de::Visitor<'de>
                    {
                        Err(Error::custom("expected struct or struct variant"))
                    }

                    serde::forward_to_deserialize_any! {
                        bool
                        i8 i16 i32 i64
                        u8 u16 u32 u64
                        f32 f64
                        char str string
                        bytes byte_buf
                        unit
                        seq tuple tuple_struct map
                        identifier ignored_any
                    }

                    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
                    where
                        V: de::Visitor<'de>
                    {
                        visitor.visit_some(self)
                    }

                    fn deserialize_unit_struct<V>(
                        self,
                        _name: &'static str,
                        _visitor: V,
                    ) -> Result<V::Value, Self::Error>
                    where
                        V: de::Visitor<'de>
                    {
                        // Use fields == None
                        Err(Error::custom("end"))
                    }

                    fn deserialize_newtype_struct<V>(
                        self,
                        _name: &'static str,
                        visitor: V,
                    ) -> Result<V::Value, Self::Error>
                    where
                        V: de::Visitor<'de>
                    {
                        visitor.visit_newtype_struct(self)
                    }

                    fn deserialize_struct<V>(
                        self,
                        _name: &'static str,
                        fields: &'static [&'static str],
                        _visitor: V,
                    ) -> Result<V::Value, Self::Error>
                    where
                        V: de::Visitor<'de>
                    {
                        self.fields = Some(fields);
                        Err(Error::custom("end"))
                    }

                    fn deserialize_enum<V>(
                        self,
                        _name: &'static str,
                        _variants: &'static [&'static str],
                        visitor: V,
                    ) -> Result<V::Value, Self::Error>
                    where
                        V: de::Visitor<'de>
                    {
                        visitor.visit_enum(self)
                    }
                }
                impl<'de, E> EnumAccess<'de> for &mut QueryFields<'de, E>
                where
                    E: Error
                {
                    type Error = E;

                    type Variant = Self;

                    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
                    where
                        V: DeserializeSeed<'de>
                    {
                        Ok((
                            seed.deserialize(StrDeserializer::new(self.name))?,
                            self,
                        ))
                    }
                }
                impl<'de, E> VariantAccess<'de> for &mut QueryFields<'de, E>
                where
                    E: Error
                {
                    type Error = E;

                    fn unit_variant(self) -> Result<(), Self::Error> {
                        Err(Error::custom("expected struct or struct variant"))
                    }

                    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
                    where
                        T: DeserializeSeed<'de>
                    {
                        seed.deserialize(self)
                    }

                    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
                    where
                        V: de::Visitor<'de>
                    {
                        Err(Error::custom("expected struct or struct variant"))
                    }

                    fn struct_variant<V>(
                        self,
                        fields: &'static [&'static str],
                        _visitor: V,
                    ) -> Result<V::Value, Self::Error>
                    where
                        V: de::Visitor<'de>
                    {
                        self.fields = Some(fields);
                        Err(Error::custom("end"))
                    }
                }

                let mut query = QueryFields {
                    name: &self.name,
                    fields: None,
                    error: std::marker::PhantomData,
                };

                let error: A::Error = ElementType::deserialize(&mut query).unwrap_err();

                let string = error.to_string();

                if string == "end" {
                    variant.struct_variant(
                        query.fields.unwrap_or_default(),
                        self
                    )
                } else {
                    Err(error)
                }

            }
        }

        deserializer.deserialize_enum(
            "",
            &[],
            Visitor::default()
        )
    }
}

impl Element {
    pub fn build(&self) -> widgets::WidgetBase {
        widgets::WidgetBase::new(
            self.style.clone(),
            self.name.clone(),
            self.id.clone(),
            self.class_list.clone(),
            self.inner.build()
        )
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Default, PartialEq)]
pub enum ElementType {
    #[default] Empty,

    Section(Box<elements::SectionElement>),
    Row(Box<elements::SectionElement>),
    Column(Box<elements::SectionElement>),
    List(Box<elements::ListElement>),
    Switch(Box<elements::SwitchElement>),

    Animatable(Box<elements::AnimatableElement>),
    #[serde(alias="cond", alias="if")]
    Conditional(Box<elements::ConditionalElement>),

    Text(Box<elements::TextElement>),
    GameplayPreview(Box<elements::GameplayPreviewElement>),

    Button(Box<elements::ButtonElement>),
    Slider(Box<elements::SliderElement>),
    Checkbox(Box<elements::CheckboxElement>),
    TextInput(Box<elements::TextInputElement>),
    KeyButton(Box<elements::InputButtonElement<input::Key>>),
    GamepadButton(Box<elements::InputButtonElement<input::Key>>),
    Dropdown(Box<elements::DropdownElement>),
    Visualization(Box<elements::VisualizationElement>),
}
impl ElementType {
    pub fn build(&self) -> Box<dyn Widget<actions::Action>> {
        macro_rules! build {
            ($($i: ident),*$(,)?) => {
                match self {
                    Self::Empty => ui::EmptyWidget::new_boxed(),
                    $(
                        Self::$i(e) => e.build().boxed(),
                    )*
                }
            }
        }
        
        build!(
            Section,
            Row,
            Column,
            
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
}

#[derive(Deserialize)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Wrapped<T> {
    #[serde(rename="$value")]
    pub inner: T
}

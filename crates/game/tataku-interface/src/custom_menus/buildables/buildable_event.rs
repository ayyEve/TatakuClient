use crate::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct BuildableEvent {
    pub event: TatakuEvent<CustomEvent>,

    pub actions: Vec<BuildableAction>,
}

#[derive(Deserialize)]
#[derive(Clone, Debug, PartialEq)]
pub struct CustomEvent {
    #[serde(rename="@event")]
    pub event: ArcStr,
}

impl<'de> Deserialize<'de> for BuildableEvent {
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
                MapDeserializer,
                StrDeserializer,
            }
        };

        struct Enum<E> {
            name: String,
            attributes: Vec<(String, FromString)>,
            error: std::marker::PhantomData<E>,
        }
        impl<'de, E: Error> EnumAccess<'de> for Enum<E> {
            type Error = E;
            type Variant = Self;

            fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
            where
                V: DeserializeSeed<'de>
            {
                Ok((
                    seed.deserialize(StrDeserializer::new(&self.name))?,
                    self
                ))
            }
        }
        impl<'de, E: Error> VariantAccess<'de> for Enum<E> {
            type Error = E;

            fn unit_variant(self) -> Result<(), Self::Error> {
                debug_assert!(self.attributes.is_empty(), "unit variants can't have attributes");
                Ok(())
            }

            fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
            where
                T: DeserializeSeed<'de>
            {
                seed.deserialize(MapDeserializer::new(self.attributes.into_iter()))
            }

            fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
            where
                V: de::Visitor<'de>
            {
                unimplemented!()
            }

            fn struct_variant<V>(
                self,
                _fields: &'static [&'static str],
                visitor: V,
            ) -> Result<V::Value, Self::Error>
            where
                V: de::Visitor<'de>
            {
                visitor.visit_map(MapDeserializer::new(self.attributes.into_iter()))
            }
        }

        #[derive(Default)]
        struct Visitor {
            name: String,
        }
        impl<'de> de::Visitor<'de> for Visitor {
            type Value = BuildableEvent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "a TatakuEvent and optionally some actions")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>
            {
                let mut attributes = Vec::new();
                let mut actions = Vec::new();

                while let Some(key) = map.next_key::<String>()? {
                    if &key[..1] == "@" {
                        // attributes have to be deserializable as strings
                        let value: String = map.next_value()?;

                        attributes.push((key, FromString::from(value)));
                    } else {
                        actions.push(map.next_value()?);
                    }
                }

                let de = EnumAccessDeserializer::new(Enum {
                    name: self.name,
                    attributes,
                    error: std::marker::PhantomData,
                });
                let event = TatakuEvent::deserialize(de)?;

                Ok(BuildableEvent {
                    event,
                    actions,
                })
            }

            fn visit_enum<A>(mut self, data: A) -> Result<Self::Value, A::Error>
            where
                A: de::EnumAccess<'de>
            {
                let (name, variant): (String, _) = data.variant()?;

                self.name = name;

                variant.struct_variant(
                    &["$value"],
                    self
                )
            }
        }

        deserializer.deserialize_enum(
            "",
            &[],
            Visitor::default()
        )
    }
}

impl BuildableEvent {
    pub fn resolve(
        event: &TatakuEvent<CustomEvent>,
    ) -> TatakuEvent {
        match event {
            // These are necessary because technically the
            // types differ at the generic, even if not used here.
            TatakuEvent::SongStart => TatakuEvent::SongStart,
            TatakuEvent::SongPause => TatakuEvent::SongPause,
            TatakuEvent::SongEnd   => TatakuEvent::SongEnd,

            TatakuEvent::MenuEnter => TatakuEvent::MenuEnter,
            TatakuEvent::MenuLeave => TatakuEvent::MenuLeave,

            TatakuEvent::MapAdded => TatakuEvent::MapAdded,

            TatakuEvent::KeyPress(k)   => TatakuEvent::KeyPress(*k),
            TatakuEvent::KeyRelease(k) => TatakuEvent::KeyRelease(*k),

            TatakuEvent::ControllerPress(k) => TatakuEvent::ControllerPress(*k),
            TatakuEvent::ControllerRelease(k) => TatakuEvent::ControllerRelease(*k),

            TatakuEvent::CustomEvent(event) => TatakuEvent::CustomEvent(
                event.event.clone()
            )
        }
    }
}

#[test]
fn test() {
    quick_xml::de::from_str::<BuildableEvent>(r#"
        <event>
            <event><songEnd/></event>
            <actions>
                <map><next/></map>
            </actions>
        </event>
    "#)
    .map_err(|e| TatakuError::String(format!("{e}")))
    .unwrap();
}

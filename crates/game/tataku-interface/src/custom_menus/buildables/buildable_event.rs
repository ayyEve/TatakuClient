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
    pub event: BuildableValue,
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
        values: &dyn Reflect,
        // passed_in: Option<&TatakuValue>,
    ) -> Option<TatakuEvent> {
        match event {
            // These are necessary because technically the
            // types differ at the generic, even if not used here.
            TatakuEvent::SongStart => Some(TatakuEvent::SongStart),
            TatakuEvent::SongPause => Some(TatakuEvent::SongPause),
            TatakuEvent::SongEnd   => Some(TatakuEvent::SongEnd),

            TatakuEvent::MenuEnter => Some(TatakuEvent::MenuEnter),
            TatakuEvent::MenuLeave => Some(TatakuEvent::MenuLeave),

            TatakuEvent::MapAdded => Some(TatakuEvent::MapAdded),

            TatakuEvent::KeyPress(k)   => Some(TatakuEvent::KeyPress(*k)),
            TatakuEvent::KeyRelease(k) => Some(TatakuEvent::KeyRelease(*k)),

            TatakuEvent::ControllerPress(k) => Some(TatakuEvent::ControllerPress(*k)),
            TatakuEvent::ControllerRelease(k) => Some(TatakuEvent::ControllerRelease(*k)),

            TatakuEvent::CustomEvent(event) => event.event
                .resolve(values, None)
                .map(|i| i.as_string())
                .map(TatakuEvent::CustomEvent),
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

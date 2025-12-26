use crate::prelude::*;
use common::reflect::*;
use ui::message::MessageTarget;

#[derive(Deserialize)]
#[derive(Clone, Debug, PartialEq)]
pub struct BuildableMessage {
    pub tag: BuildableText,
    pub target: Option<BuildableMessageTarget>,
    pub value: BuildableValue,
}
#[cfg(feature="graphics")]
impl BuildableMessage {
    pub fn build(&mut self) {
        if let Err(e) = self.tag.compute() {
            error!("Error building text {:?}: {e:?}", self.tag);
        }
        if let Some(target) = self.target.as_mut() {
            target.build();
        }

        self.value.build();
    } 

    pub fn resolve(
        &self, 
        node_id: ui::tree::NodeId,
        source: ui::MessageSource,
        values: &dyn Reflect,
        passed_in: Option<&tataku::TatakuValue>,
    ) -> Option<ui::Message> {
        Some(ui::Message::new(
            source, 
            self.tag.to_string(values), 
            self.target.as_ref().and_then(|t| t.resolve(node_id, values)), 
            Box::new(self.value.resolve(values, passed_in)?.into_owned())
        ))
    }
}

#[derive(Deserialize)]
#[derive(Clone, Debug, PartialEq)]
pub enum BuildableMessageTarget {
    This,

    Id {
        #[serde(rename="$value")]
        id: BuildableText,
    },

    Class{
        #[serde(rename="$value")]
        class: BuildableText,
    }
}

#[cfg(feature="graphics")]
impl BuildableMessageTarget {
    pub fn build(&mut self) {
        let res = match self {
            Self::This => Ok(()),

            Self::Id { id } 
                => id.compute().map_err(|e| (e, id)),
            
            Self::Class { class } 
                => class.compute().map_err(|e| (e, class)),
        };

        if let Err((e, txt)) = res {
            error!("Error building text {txt:?}: {e:?}");
        }
    } 

    pub fn resolve(
        &self, 
        node_id: ui::tree::NodeId,
        values: &dyn Reflect,
    ) -> Option<MessageTarget> {
        match self {
            Self::This => Some(MessageTarget::Node(node_id)),
            Self::Id { id } 
                => Some(MessageTarget::ElementId(id.to_string(values).into())),
            Self::Class { class } 
                => Some(MessageTarget::ElementClass(class.to_string(values).into())),
        }
    }
}
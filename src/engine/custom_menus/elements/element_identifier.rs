use crate::prelude::*;

#[derive(Clone, Debug)]
pub enum ElementIdentifier {
    Row {
        elements: Vec<ElementDef>,
        padding: Option<ElementPadding>,
        margin: Option<f32>,
    },
    Column {
        elements: Vec<ElementDef>,
        padding: Option<ElementPadding>,
        margin: Option<f32>,
    },
    Space,
    Button {
        text: CustomElementText,
        color: Option<Color>,
        font_size: Option<f32>,

        action: ButtonAction,
        padding: Option<ElementPadding>,
    },
    Text {
        text: CustomElementText,
        color: Option<Color>,
        font_size: Option<f32>,
    },
    TextInput {
        placeholder: CustomElementText,
        variable: String,
        is_password: bool,
    },

    SongDisplay,
    GameplayPreview {
        visualization: Option<String>,
    },
    Animatable {
        triggers: Vec<AnimatableTrigger>,
        actions: HashMap<String, Vec<AnimatableAction>>,
        element: Box<ElementDef>,
    },

    // TODO: !!!
    Custom {

    }
}

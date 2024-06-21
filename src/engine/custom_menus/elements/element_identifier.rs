use crate::prelude::*;

#[derive(Clone, Debug)]
pub enum ElementIdentifier {
    /// id = row
    Row {
        elements: Vec<ElementDef>,
        padding: Option<ElementPadding>,
        margin: Option<f32>,
    },
    /// id = col
    Column {
        elements: Vec<ElementDef>,
        padding: Option<ElementPadding>,
        margin: Option<f32>,
    },
    /// id = space
    Space,
    /// id = button
    Button {
        element: Box<ElementDef>,
        action: ButtonAction,
        padding: Option<ElementPadding>,
    },
    /// id = text
    Text {
        text: CustomElementText,
        color: Option<Color>,
        font_size: Option<f32>,
        font: Option<String>,
    },
    /// id = text_input
    TextInput {
        placeholder: CustomElementText,
        variable: String,
        on_input: Option<ButtonAction>,
        on_submit: Option<ButtonAction>,
        is_password: bool,
    },
    /// id = gameplay_preview
    GameplayPreview {
        visualization: Option<String>,
    },
    /// id = animatable
    Animatable {
        triggers: Vec<AnimatableTrigger>,
        actions: HashMap<String, Vec<AnimatableAction>>,
        element: Box<ElementDef>,
    },
    /// id = styled_content
    StyledContent {
        element: Box<ElementDef>,
        padding: Option<ElementPadding>,

        color: Option<Color>,
        border: Option<Border>,
        image: Option<String>,
        built_image: Option<Image>,
        shape: Option<Shape>,
    },

    /// id = conditional
    Conditional {
        cond: ElementCondition,
        if_true: Box<ElementDef>,
        if_false: Option<Box<ElementDef>>,
    },

    /// id = list
    List {
        list_var: String,
        scrollable: bool,
        element: Box<ElementDef>,
        variable: Option<String>,
    },

    /// id = dropdown
    Dropdown {
        options_key: String,
        options_display_key: Option<String>,
        selected_key: String,
        
        on_select: Option<ButtonAction>,

        padding: Option<ElementPadding>,
        placeholder: Option<String>,
        font_size: Option<f32>,
        font: Option<String>,
    },

}






#[derive(Clone, Debug)]
pub enum ElementCondition {
    Unbuilt(String),
    Built(Arc<CustomElementCalc>, String),
    Failed,
}
impl ElementCondition {
    pub fn build(&mut self) {
        let ElementCondition::Unbuilt(s) = self else { unreachable!() };
        match CustomElementCalc::parse(format!("{s} == true")) {
            Ok(built) => *self = ElementCondition::Built(Arc::new(built), s.clone()),
            Err(e) => {
                error!("Error building conditional: {e:?}");
                *self = ElementCondition::Failed;
            }
        }
    }

    pub fn resolve<'a>(&'a self, values: &ValueCollection) -> ElementResolve<'a> {
        match self {
            Self::Failed => ElementResolve::Failed,
            Self::Unbuilt(calc_str) => ElementResolve::Unbuilt(calc_str),
            Self::Built(calc, calc_str) => {
                match calc.resolve(values).map(|n| n.as_bool()) {
                    Ok(true) => ElementResolve::True,
                    Ok(false) => ElementResolve::False,
                    Err(e) => {
                        error!("Error with shunting yard calc. calc: '{calc_str}', error: {e:?}");
                        println!("");
                        ElementResolve::Error(e)
                    }
                }
            }
        }
    }
}
pub enum ElementResolve<'a> {
    Failed,
    Unbuilt(&'a String),
    True,
    False,
    Error(ShuntingYardError)
}

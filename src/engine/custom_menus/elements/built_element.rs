use crate::prelude::*;
use crate::prelude::iced_elements::*;

pub struct BuiltElementDef {
    pub element: ElementDef,
    pub children: Vec<Box<dyn Widgetable>>,
}
impl BuiltElementDef {
    /// get the view from the first child, or an empty view if none exist
    fn first_child_view(&self, owner: MessageOwner, values: &mut ShuntingYardValues) -> IcedElement {
        let Some(child) = self.children.first() else { return EmptyElement.into_element() };
        child.view(owner, values)
    }
}


#[async_trait]
impl Widgetable for BuiltElementDef {
    async fn update(&mut self) {
        for i in self.children.iter_mut() {
            i.update().await;
        }
    }
    
    fn view(&self, owner: MessageOwner, values: &mut ShuntingYardValues) -> IcedElement {
        match &self.element.id {
            ElementIdentifier::Space => Space::new(self.element.width, self.element.height).into_element(),
            ElementIdentifier::Button { padding,  action, ..} => {
                Button::new(self.first_child_view(owner, values))
                    .on_press_maybe(action.into_message(owner))
                    .width(self.element.width)
                    .height(self.element.height)
                    .chain_maybe(*padding, |s, p| s.padding(p))
                    .into_element()
            }
            ElementIdentifier::Text { text, color, font_size, font }  => {
                iced_elements::Text::new(text.to_string(values))
                    .width(self.element.width)
                    .height(self.element.height)
                    .chain_maybe(*color, |s, c| s.color(c))
                    .chain_maybe(*font_size, |s, f| s.size(f))
                    .chain_maybe(font.as_ref(), |s, font| match &**font {
                        "main" => s.font(iced::Font::with_name("main")),
                        "fallback" => s.font(iced::Font::with_name("fallback")),
                        "fa"|"font_awesome" => s.font(iced::Font::with_name("font_awesome")),
                        _ => s
                    })
                    .into_element()
            }
            ElementIdentifier::TextInput { placeholder, variable, is_password } => {
                let placeholder = placeholder.to_string(values);
                let value = values.get_string(&variable).unwrap_or_default();
                let variable = variable.clone();

                iced_elements::TextInput::new(&placeholder, &value)
                    .on_input(move |t| Message::new(owner, &variable, MessageType::Text(t)))
                    .width(self.element.width)
                    .chain_bool(*is_password, |s| s.password())
                    .into_element()
            }

            ElementIdentifier::Row { padding, margin, .. } => {
                Row::with_children(self.children.iter().map(|e|e.view(owner, values)).collect())
                    .width(self.element.width)
                    .height(self.element.height)
                    .chain_maybe(*padding, |s, p| s.padding(p))
                    .chain_maybe(*margin, |s, m| s.spacing(m))
                    .into_element()
            }
            ElementIdentifier::Column { padding, margin, .. } => {
                Column::with_children(self.children.iter().map(|e|e.view(owner, values)).collect())
                    .width(self.element.width)
                    .height(self.element.height)
                    .chain_maybe(*padding, |s, p| s.padding(p))
                    .chain_maybe(*margin, |s, m| s.spacing(m))
                    .into_element()
            }

            ElementIdentifier::StyledContent { padding, color, border, shape, .. } => {
                ContentBackground::new(self.first_child_view(owner, values))
                    .width(self.element.width)
                    .height(self.element.height)
                    .border(*border)
                    .color(*color)
                    .shape_maybe(*shape)
                    .padding_maybe(*padding)
                    .into_element()
            }

            ElementIdentifier::KeyHandler { events } => {
                KeyEventsHandler::new(events, owner)
                    .into_element()
            }

            ElementIdentifier::Custom {} => {
                todo!()
            }


            _ => {
                // warn!("missed object? {:?}", self.element.id);
                self.first_child_view(owner, values)
                // panic!("you missed something")
            }
        }
    }
}

// TODO: come up with a better name for this
#[async_trait]
pub trait Widgetable: Send + Sync {
    async fn update(&mut self) {}
    fn view(&self, owner: MessageOwner, values: &mut ShuntingYardValues) -> IcedElement;
}


pub trait ChainMaybe:Sized {
    fn chain_bool(self, check: bool, f: impl Fn(Self) -> Self) -> Self;
    fn chain_maybe<T>(self, op: Option<T>, f: impl Fn(Self, T) -> Self) -> Self;
}
impl<S> ChainMaybe for S {
    fn chain_bool(self, check: bool, f: impl Fn(Self) -> Self) -> Self {
        if !check { return self }
        f(self)
    }
    fn chain_maybe<T>(self, op: Option<T>, f: impl Fn(Self, T) -> Self) -> Self {
        let Some(op) = op else { return self };
        f(self, op)
    }
}

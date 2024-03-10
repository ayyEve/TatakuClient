use crate::prelude::*;
use crate::prelude::iced_elements::*;

pub struct BuiltElementDef {
    pub element: ElementDef,
    pub special: BuiltElementData
}

impl BuiltElementDef {
    #[async_recursion::async_recursion]
    pub async fn build_elements(elements: &Vec<ElementDef>) -> Vec<BuiltElementDef> {
        let mut list = Vec::new();
        for i in elements.iter() {
            list.push(i.build().await);
        }
        list
    }

    #[async_recursion::async_recursion]
    pub async fn update(&mut self) {
        match &mut self.special {
            BuiltElementData::None => {}
            BuiltElementData::GameplayPreview(gameplay) => gameplay.update().await,
            BuiltElementData::Element(e) => e.update().await,
            BuiltElementData::Elements(e) => for i in e { i.update().await },
        }
    }

    pub fn view(&self, owner: MessageOwner, values: &ShuntingYardValues) -> IcedElement {
        match (&self.element.id, &self.special) {
            (ElementIdentifier::Space, _) => Space::new(self.element.width, self.element.height).into_element(),
            (ElementIdentifier::Button { 
                text, 
                color,
                font_size, 
                padding, 

                action ,
            }, _) => {
                Button::new(iced_elements::Text::new(text.to_string(values)))
                    .on_press_maybe(action.into_message(owner))
                    .width(self.element.width)
                    .height(self.element.height)
                    .into_element()
            }
            (ElementIdentifier::Text { text, color, font_size, font }, _)  => {
                let mut text = iced_elements::Text::new(text.to_string(values))
                    .width(self.element.width)
                    .height(self.element.height);

                if let Some(font) = font {
                    match &**font {
                        "main" => text = text.font(iced::Font::with_name("main")),
                        "fallback" => text = text.font(iced::Font::with_name("fallback")),
                        "fa"|"font_awesome" => text = text.font(iced::Font::with_name("font_awesome")),
                        _ => {}
                    }
                }
                if let Some(color) = color { text = text.color(*color) }
                if let Some(font_size) = font_size { text = text.size(*font_size) }

                text.into_element()
            }
            (ElementIdentifier::TextInput { placeholder, variable, is_password }, _) => {
                let placeholder = placeholder.to_string(values);
                let value = values.get_string(&variable).unwrap_or_default();
                let variable = variable.clone();

                let mut text = iced_elements::TextInput::new(&placeholder, &value)
                    .on_input(move |t|Message::new(owner, &variable, MessageType::Text(t)))
                    .width(self.element.width)
                    ;
                if *is_password { text = text.password(); }

                text.into_element()
            }

            (ElementIdentifier::Row { padding, margin, .. }, BuiltElementData::Elements(elements)) => {
                let mut row = Row::with_children(elements.iter().map(|e|e.view(owner, values)).collect())
                    .width(self.element.width)
                    .height(self.element.height);

                if let Some(padding) = padding { row = row.padding(*padding) }
                if let Some(margin) = margin { row = row.spacing(*margin) }
                
                row.into_element()
            }
            (ElementIdentifier::Column { padding, margin, .. }, BuiltElementData::Elements(elements)) => {
                let mut col = Column::with_children(elements.iter().map(|e|e.view(owner, values)).collect())
                    .width(self.element.width)
                    .height(self.element.height);
                
                if let Some(padding) = padding { col = col.padding(*padding) }
                if let Some(margin) = margin { col = col.spacing(*margin) }

                col.into_element()
            }

            (ElementIdentifier::StyledContent { padding, color, border, shape, .. }, BuiltElementData::Element(element)) => {
                ContentBackground::new(element.view(owner, values))
                    .width(self.element.width)
                    .height(self.element.height)
                    .border(*border)
                    .color(*color)
                    .shape_maybe(*shape)
                    .padding_maybe(*padding)
                    .into_element()
            }

            (ElementIdentifier::KeyHandler { events }, _) => {
                KeyEventsHandler::new(events, owner).into_element()
            }
            // specials
            (_, BuiltElementData::Element(e)) => e.view(owner, values),
            (_, BuiltElementData::GameplayPreview (gameplay)) => gameplay.widget(),

            (ElementIdentifier::Custom {}, _) => {
                todo!()
            }

            _ => panic!("you missed something")
        }
    }
}

pub enum BuiltElementData {
    None,
    Element(Box<BuiltElementDef>),
    Elements(Vec<BuiltElementDef>),

    GameplayPreview(GameplayPreview),
}



// pub trait Widgetable: Send + Sync + iced::widget::Widget<Message, IcedRenderer> {
//     async fn update(&mut self) {}
// }
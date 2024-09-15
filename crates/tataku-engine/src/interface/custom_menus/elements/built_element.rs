use crate::prelude::*;
use crate::prelude::iced_elements::*;

pub struct BuiltElementDef {
    pub element: ElementDef,
    pub children: Vec<Box<dyn Widgetable>>,
}
impl BuiltElementDef {
    /// get the view from the nth child, or an empty view if none exist
    fn nth_child_view(&self, n:usize, owner: MessageOwner, values: &mut dyn Reflect) -> IcedElement {
        let Some(child) = self.children.get(n) else { return EmptyElement.into_element() };
        child.view(owner, values)
    }

    /// get the view from the first child, or an empty view if none exist
    fn first_child_view(&self, owner: MessageOwner, values: &mut dyn Reflect) -> IcedElement {
        let Some(child) = self.children.first() else { return EmptyElement.into_element() };
        child.view(owner, values)
    }
}


#[async_trait]
impl Widgetable for BuiltElementDef {
    async fn update(&mut self, values: &mut dyn Reflect, actions: &mut ActionQueue) {
        for i in self.children.iter_mut() {
            i.update(values, actions).await;
        }
    }

    async fn handle_message(&mut self, message: &Message, values: &mut dyn Reflect) -> Vec<TatakuAction> { 
        let mut actions = Vec::new();

        for i in self.children.iter_mut() {
            actions = i.handle_message(message, values).await;
            if !actions.is_empty() {
                return actions
            }
        }
        
        actions
    }
    
    fn view(&self, owner: MessageOwner, values: &mut dyn Reflect) -> IcedElement {
        match &self.element.element {
            ElementIdentifier::Space => Space::new(self.element.width, self.element.height).into_element(),
            ElementIdentifier::Button { padding,  action, ..} => {
                Button::new(self.first_child_view(owner, values))
                    .on_press_maybe(action.resolve(owner, values, None))
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
                    .chain_maybe(font.as_ref().and_then(map_font), |s, font| s.font(font))
                    .into_element()
            }
            ElementIdentifier::TextInput { placeholder, variable,  on_input, on_submit,is_password } => {
                let placeholder = placeholder.to_string(values);
                let value = values.reflect_get::<String>(variable).cloned().unwrap_or_default();
                // let value = values.get_string(&variable).unwrap_or_default();.unwrap_or_default();
                let variable = variable.clone();

                let mut input_action = on_input.clone();
                input_action.as_mut().map(|a| a.resolve_variables(values)); 

                let on_input:Box<dyn Fn(String) -> Message> = match input_action {
                    Some(on_input) => {
                        Box::new(move |t| {
                            let value = TatakuValue::String(t);
                            let m = Message::new(owner, "", MessageType::CustomMenuAction(CustomMenuAction::SetValue(variable.clone(), value.clone()), None));

                            on_input
                                .resolve_passed_in(owner, Some(value))
                                .map(|resolved| Message::new(owner, "", MessageType::Multi(vec![ m.clone(), resolved ])))
                                .unwrap_or_else(|| m)
                        })
                    }
                    None => {
                        Box::new(move |t| Message::new(owner, "", MessageType::CustomMenuAction(CustomMenuAction::SetValue(variable.clone(), TatakuValue::String(t)), None)))
                    }
                };


                iced_elements::TextInput::new(&placeholder, &value)
                    .on_input(on_input)
                    .width(self.element.width)
                    .secure(*is_password)
                    .chain_maybe(on_submit.as_ref().and_then(|e| e.resolve(owner, values, None)), |t, m| t.on_submit(m))
                    .into_element()
            }

            ElementIdentifier::Row { padding, margin, .. } => {
                Row::with_children(self.children.iter().map(|e| e.view(owner, values)).collect::<Vec<_>>())
                    .width(self.element.width)
                    .height(self.element.height)
                    .chain_maybe(*padding, |s, p| s.padding(p))
                    .chain_maybe(*margin, |s, m| s.spacing(m))
                    .into_element()
            }
            ElementIdentifier::Column { padding, margin, .. } => {
                Column::with_children(self.children.iter().map(|e| e.view(owner, values)).collect::<Vec<_>>())
                    .width(self.element.width)
                    .height(self.element.height)
                    .chain_maybe(*padding, |s, p| s.padding(p))
                    .chain_maybe(*margin, |s, m| s.spacing(m))
                    .into_element()
            }

            ElementIdentifier::StyledContent { padding, color, border, shape, built_image, .. } => {
                ContentBackground::new(self.first_child_view(owner, values))
                    .width(self.element.width)
                    .height(self.element.height)
                    .border(*border)
                    .color(*color)
                    .image(built_image.clone())
                    .shape_maybe(*shape)
                    .padding_maybe(*padding)
                    .into_element()
            }

            ElementIdentifier::Conditional { cond, .. } => {
                match cond.resolve(values) {
                    ElementResolve::Failed | ElementResolve::Error(_) => EmptyElement.into_element(),
                    ElementResolve::Unbuilt(_) => panic!("conditional element not built!"),
                    ElementResolve::True => self.nth_child_view(0, owner, values),
                    ElementResolve::False => self.nth_child_view(1, owner, values),
                }
            }

            ElementIdentifier::List { list_var, scrollable, variable, .. } => {
                // info!("using variable: {variable}");

                let ele = self.children.first().unwrap();
                let mut children = Vec::new();

                let Ok(iter) = values.reflect_iter(list_var) else {
                    error!("list variable doesnt exist! {list_var}");
                    return EmptyElement.into_element() 
                };
                // let iter = iter.enumerate().map(|(n, _)| n).collect::<Vec<_>>();
                let iter = iter
                    .filter_map(|i| i.duplicate()
                ).collect::<Vec<_>>();

                // TODO:!!!!
                for value in iter {
                    if let Err(e) = values.reflect_insert(variable, value) {
                        error!("{e:?}");
                        return EmptyElement.into_element() 
                    }
                    children.push(ele.view(owner, values));
                }
                // values.remove(&var);

                if *scrollable {
                    iced::widget::Scrollable::new(
                        iced_elements::Column::with_children(children)
                        .spacing(5.0)
                        // .clip(true)
                        .into_element()
                    )
                    .id(iced::widget::scrollable::Id::new("a"))
                    .width(self.element.width)
                    .height(self.element.height)
                    .into_element()
                } else {
                    Column::with_children(children)
                        .width(self.element.width)
                        .height(self.element.height)
                        .into_element()
                }
            }

            ElementIdentifier::Dropdown { 
                options_key, 
                options_display_key: _, 
                selected_key, 

                on_select,

                padding,
                placeholder, 
                font_size,
                font
            } => {
                // let options_display_key = options_display_key.as_ref().unwrap_or(options_key);

                struct Test {
                    id: String,
                    value: Box<dyn Reflect>,
                    display: String,
                }
                impl ToString for Test {
                    fn to_string(&self) -> String {
                        self.display.clone()
                    }
                }
                impl PartialEq for Test {
                    fn eq(&self, other: &Self) -> bool {
                        self.id == other.id
                    }
                }
                impl Eq for Test {}
                impl Clone for Test {
                    fn clone(&self) -> Self {
                        Test {
                            id: self.id.clone(),
                            value: self.value.duplicate().expect("Value in dropdown not clonable"),
                            display: self.display.clone()
                        }
                    }
                }

                let Ok(iter) = values.reflect_iter(options_key) else {
                    error!("Value not exist for {options_key}");
                    return EmptyElement.into_element()
                };
                let list2 = iter.filter_map(|value| {
                    let a = TatakuValue::from_reflection(value).ok()?.as_string();
                    Some(Test { 
                        id: a.clone(),
                        value: value.duplicate().expect("Value in dropdown not clonable"),
                        // id.downcast_ref::<String>()?.clone(), 
                        display: a // TODO: 
                    })
                }).collect::<Vec<_>>();

                // let Ok(Some(list)) = values.get_raw(options_key).map(|s| s.list_maybe()) else { 
                //     error!("Value not exist for {options_key}");
                //     return EmptyElement.into_element() 
                // };
                // let Ok(Some(display_list)) = values.get_raw(options_display_key).map(|s| s.list_maybe()) else { 
                //     error!("Value not exist for {options_display_key}");
                //     return EmptyElement.into_element()
                // };

                // let list2 = list
                //     .iter()
                //     // .zip(display_list.iter())
                //     .map(|id| Test { id: id.as_string(), display: id.get_display()} )
                //     .collect::<Vec<_>>();


                let selected = match values.impl_get(ReflectPath::new(selected_key)) {
                    Ok(s) => match TatakuValue::from_reflection(s).map(|s| s.as_string()) {
                        Ok(s) => Some(s),
                        Err(ReflectError::OptionIsNone) => None,
                        Err(e) => {
                            error!("dropdown error: {e:?}");
                            return EmptyElement.into_element()
                        }
                    }, 
                    Err(ReflectError::EntryNotExist { .. }) => None,
                    Err(e) => {
                        error!("dropdown error: {e:?}");
                        return EmptyElement.into_element()
                    }
                };

                let selected = selected
                    .as_ref()
                    .and_then(|s| list2.iter().find(|i| &i.id == s))
                    .cloned();
                let selected_key = selected_key.clone();

                // println!("a: {on_select:?}");
                let mut on_select = on_select.clone();
                on_select.as_mut().map(|on_select| on_select.resolve_variables(values));

                let on_select:Box<dyn Fn(Test)->Message> = match on_select.clone() {
                    Some(action) => Box::new(move |t: Test| 
                        action.resolve_passed_in(
                            owner, 
                            t.value.duplicate().map(TatakuValue::Reflect)
                        )
                        .unwrap_or(Message::new(owner, selected_key.clone(), MessageType::Value(TatakuValue::None)))
                        // Message::new(owner, format!("dropdown.{selected_key}"), MessageType),
                    ),
                    
                    None => Box::new(move |t:Test| Message::new(owner, &selected_key, MessageType::Value(t.value.duplicate().map(TatakuValue::Reflect).unwrap_or_default())))
                };

                iced_elements::Dropdown::new(
                    list2, 
                    selected,
                    on_select,
                )
                .width(self.element.width)
                .chain_maybe(padding.as_ref(), |t, p| t.padding(*p))
                .chain_maybe(placeholder.as_ref(), |t, p| t.placeholder(p))
                .chain_maybe(font_size.as_ref(), |t, f| t.text_size(*f))
                .chain_maybe(font.as_ref().and_then(map_font), |s, font| s.font(font))
                
                .into_element()
            }

            ElementIdentifier::PanelScroll { padding, margin, .. } => {
                make_panel_scroll(
                    self.children.iter()
                        .map(|e| e.view(owner, values))
                        .collect(),
                    Cow::Owned(self.element.id.clone())
                )
                .width(self.element.width)
                .height(self.element.height)
                .chain_maybe(padding.as_ref(), |panel, pad| panel.padding(*pad))
                .chain_maybe(margin.as_ref(), |panel, margin| panel.padding(*margin))
                .into_element()
            }


            _ => {
                // warn!("missed object? {:?}", self.element.id);
                self.first_child_view(owner, values)
                // panic!("you missed something")
            }
        }

        // debug color
        .chain_maybe(self.element.debug_color, move |e, color| e.explain(color).into_element())
    }
}

// TODO: come up with a better name for this
#[async_trait]
pub trait Widgetable: Send + Sync {
    async fn update(&mut self, _values: &mut dyn Reflect, _actions: &mut ActionQueue) {}
    fn view(&self, _owner: MessageOwner, _values: &mut dyn Reflect) -> IcedElement { EmptyElement.into_element() }

    async fn handle_message(&mut self, _message: &Message, _values: &mut dyn Reflect) -> Vec<TatakuAction> { Vec::new() }

    async fn reload_skin(&mut self, _skin_manager: &mut dyn SkinProvider) {}
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


fn map_font(font: &String) -> Option<Font> {
    match &**font {
        "main" => Some(Font::Main),
        "fallback" => Some(Font::Fallback),
        "fa"|"font_awesome" => Some(Font::FontAwesome),
        _ => None
    }
}

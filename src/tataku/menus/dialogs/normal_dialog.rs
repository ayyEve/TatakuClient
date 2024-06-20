#![allow(unused, dead_code)]

use crate::prelude::*;
const Y_PADDING:f32 = 5.0;
const BUTTON_SIZE:Vector2 = Vector2::new(100.0, 30.0);

// pub type ClickFn = Box<dyn Fn(&mut GenericDialog, &mut Game) + Send + Sync>;
pub type ClickFn = Arc<dyn Fn(&mut GenericDialog) -> Option<TatakuAction> + Send + Sync>;

pub struct GenericDialog {
    num: usize,
    actions: Vec<TatakuAction>,


    bounds: Rectangle,
    // buttons: Vec<MenuButton>,
    button_actions: HashMap<String, ClickFn>,
    pub should_close: bool,
    window_size: Arc<WindowSize>
}
impl GenericDialog {
    pub fn new(_title: impl AsRef<str>) -> Self {
        let window_size = WindowSize::get();

        let bounds = Rectangle::new(
            Vector2::ZERO,
            window_size.0,
            Color::BLACK.alpha(0.7),
            Some(Border::new(
                Color::BLACK, 
                1.5
            ))
        );
        
        Self {
            num: 0,
            actions: Vec::new(),

            bounds,
            // buttons: Vec::new(),
            button_actions: HashMap::new(),

            should_close: false,
            window_size
        }
    }

    pub fn add_button(&mut self, text: impl ToString, on_click: ClickFn) {
        let text = text.to_string();

        // let y_pos = 100.0 + (BUTTON_SIZE.y + Y_PADDING) * self.buttons.len() as f32;

        // let mut button = MenuButton::new(
        //     Vector2::new((self.window_size.x - BUTTON_SIZE.x) / 2.0, y_pos),
        //     BUTTON_SIZE,
        //     &text,
        //     Font::Main,
        // );
        // button.set_tag(&text);
        // self.buttons.push(button);
        self.button_actions.insert(text, on_click);
    }

    pub fn add_action(&mut self, action: impl Into<TatakuAction>) {
        self.actions.push(action.into());
    }
}

#[async_trait]
impl Dialog for GenericDialog {
    fn name(&self) -> &'static str { "generic_dialog" }
    fn get_num(&self) -> usize { self.num }
    fn set_num(&mut self, num: usize) { self.num = num }

    fn should_close(&self) -> bool { self.should_close }
    // fn get_bounds(&self) -> Bounds { *self.bounds }
    async fn force_close(&mut self) { self.should_close = true; }
    
    async fn handle_message(&mut self, message: Message, values: &mut ValueCollection) {
        let Some(tag) = message.tag.as_string() else { return }; 

        if let Some(action) = self.button_actions.get(&tag).cloned() {
            if let Some(action2) = (action)(self) {
                self.actions.push(action2)
            }
        }
    }

    async fn update(&mut self, _values: &mut ValueCollection) -> Vec<TatakuAction> { self.actions.take() }

    fn view(&self, _values: &mut ValueCollection) -> IcedElement {
        use iced_elements::*;

        col!(
            self.button_actions.keys().map(|s|Button::new(Text::new(s.clone())).on_press(Message::new_dialog(self, s, MessageType::Click)).into_element()).collect::<Vec<_>>(),
            height = Fill
        )
    }

    // async fn on_mouse_down(&mut self, pos:Vector2, button:MouseButton, mods:&KeyModifiers, game:&mut Game) -> bool {
    //     let mut buttons = std::mem::take(&mut self.buttons);
    //     let actions = std::mem::take(&mut self.actions);

    //     for m_button in buttons.iter_mut() {
    //         if m_button.on_click(pos, button, *mods) {
    //             let tag = m_button.get_tag();
    //             let action = actions.get(&tag).unwrap();
    //             action(self, game);
    //             // self.should_close = true;
    //             break
    //         }
    //     }
    //     self.buttons = buttons;
    //     self.actions = actions;

    //     true
    // }

    // async fn draw(&mut self, offset: Vector2, list: &mut RenderableCollection) {
    //     // background and border
    //     let mut bounds = self.bounds;
    //     bounds.pos += offset;
    //     list.push(bounds);

    //     // draw buttons
    //     for button in self.buttons.iter_mut() {
    //         button.draw(offset, list);
    //     }
    // }
}

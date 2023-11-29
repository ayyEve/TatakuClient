use crate::prelude::*;
use tokio::sync::mpsc::{Sender, Receiver, channel};

const FONT_SIZE: f32 = 30.0;
const MARGIN: f32 = 5.0;

// const BUTTON_HEIGHT:f32 = 50.0;

// const BUTTON_SIZE:Vector2 = Vector2::new(100.0, BUTTON_HEIGHT);
// const BUTTON_MARGIN:Vector2 = Vector2::new(5.0, 5.0);
// const FONT_SIZE:f32 = BUTTON_HEIGHT * 0.8;

// const MAX_WIDTH:f32 = 500.0;

pub struct YesNoDialog {
    num: usize,

    should_close: bool,
    title: &'static str,
    prompt: String,

    // yes_button: MenuButton,
    // no_button: MenuButton,
    // cancel_button: Option<MenuButton>,

    show_cancel: bool,

    sender: Sender<YesNoResult>,

    // prompt_size: Vector2,
}
impl YesNoDialog {
    pub fn new(title: &'static str, prompt: impl ToString, show_cancel: bool) -> (Receiver<YesNoResult>, Self) {
        let prompt = prompt.to_string();
        // let zero = Vector2::ZERO;

        // // create buttons
        // let mut yes_button = MenuButton::new(zero, BUTTON_SIZE, "Yes", Font::Main);
        // let mut no_button = MenuButton::new(zero, BUTTON_SIZE, "No", Font::Main);
        // let mut cancel_button = if show_cancel { Some(MenuButton::new(zero, BUTTON_SIZE, "Cancel", Font::Main)) } else { None };
        
        // // how many buttons do we have (since it can be 2 or 3 depending if the cancel button is included or not)
        // let button_count =  if cancel_button.is_some() { 3 } else { 2 };
        // // how much width the buttons need
        // let button_widths = BUTTON_MARGIN.x + (BUTTON_SIZE.x + BUTTON_MARGIN.x) * button_count as f32;

        // // create prompt text and measure it
        // let prompt_size = Text::new(zero, FONT_SIZE, prompt.clone(), Color::BLACK, Font::Main).measure_text();

        // // get the total width of the dialog
        // // it must be at least the width of all the buttons, but at most MAX_WIDTH
        // let total_width = prompt_size.x.clamp(button_widths, MAX_WIDTH);

        // // reposition buttons

        // // space between buttons
        // let x_margin = (total_width - button_widths) / button_count as f32;

        // let mut pos = Vector2::new(x_margin + BUTTON_MARGIN.x, prompt_size.y + BUTTON_MARGIN.y * 2.0);
        // for mut btn in [Some(&mut yes_button), Some(&mut no_button), cancel_button.as_mut()] {
        //     btn.ok_do_mut(|b|b.set_pos(pos));
        //     pos.x += x_margin + BUTTON_SIZE.x + BUTTON_MARGIN.x;
        // }

        // create the sender and receiver to send the result of this dialog
        let (sender, receiver) = channel(1);

        // create the dialog
        (receiver, Self {
            num: 0,
            should_close: false,
            title,
            prompt,
            show_cancel,
            // yes_button,
            // no_button,
            // cancel_button,
            sender,
            // prompt_size: Vector2::new(total_width, prompt_size.y)
        })
    }
}

#[async_trait]
impl Dialog for YesNoDialog {
    fn name(&self) -> &'static str { "yes_no_dialog" }
    fn title(&self) -> &'static str { self.title }
    fn get_num(&self) -> usize { self.num }
    fn set_num(&mut self, num: usize) { self.num = num }

    fn should_close(&self) -> bool { self.should_close }
    // fn get_bounds(&self) -> Bounds {
    //     Bounds::new(Vector2::ZERO, self.prompt_size + Vector2::with_y(BUTTON_SIZE.y + BUTTON_MARGIN.y * 2.0))
    // }
    
    async fn force_close(&mut self) { 
        self.sender.try_send(YesNoResult::Cancel).unwrap();
        self.should_close = true; 
    }
    
    async fn handle_message(&mut self, message: Message) {
        let Some(tag) = message.tag.as_string() else { return }; 

        match &*tag {
            "yes" => {
                self.sender.try_send(YesNoResult::Yes).unwrap();
                self.should_close = true;
            }
            "no" => {
                self.sender.try_send(YesNoResult::No).unwrap();
                self.should_close = true;
            }
            "cancel" => {
                self.sender.try_send(YesNoResult::Cancel).unwrap();
                self.should_close = true;
            }

            _ => {}
        }
    }

    fn view(&self) -> IcedElement {
        use iced_elements::*;
        col!(
            // prompt
            Text::new(self.prompt.clone()).size(FONT_SIZE),
            row!(
                // yes button
                Button::new(Text::new("Yes").size(FONT_SIZE)).on_press(Message::new_dialog(self, "yes", MessageType::Click)),
                // no button
                Button::new(Text::new("No").size(FONT_SIZE)).on_press(Message::new_dialog(self, "no", MessageType::Click)),
                // cancel
                self.show_cancel.then(||Button::new(Text::new("Cancel").size(FONT_SIZE)).on_press(Message::new_dialog(self, "cancel", MessageType::Click)).into_element()).unwrap_or_else(||EmptyElement.into_element());
                width = Fill,
                spacing = MARGIN
            )
            ;

            width = Fill,
            height = Fill
        )
    }


    // async fn draw(&mut self, offset: Vector2, list: &mut RenderableCollection) {
    //     // background
    //     self.draw_background(Color::GRAY, offset, list);

    //     // prompt text
    //     let mut text = Text::new(offset, FONT_SIZE, self.prompt.clone(), Color::BLACK, Font::Main);
    //     text.center_text(&Bounds::new(offset, self.prompt_size));
    //     list.push(text);

    //     // buttons
    //     self.yes_button.draw(offset, list);
    //     self.no_button.draw(offset, list);
    //     if let Some(cancel_button) = &mut self.cancel_button {
    //         cancel_button.draw(offset, list);
    //     }
    // }
}


#[derive(Copy, Clone, Debug)]
pub enum YesNoResult {
    Yes,
    No,
    Cancel
}
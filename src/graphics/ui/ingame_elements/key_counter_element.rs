

use crate::prelude::*;

const BOX_SIZE:Vector2 = Vector2::new(40.0, 40.0);

pub struct KeyCounterElement {
    counter: KeyCounter,
    // background_image: Option<Image>,
    button_image: Option<Image>
}
impl KeyCounterElement {
    pub fn new() -> Self {
        // let mut background_image = SKIN_MANAGER.write().get_texture("inputoverlay-background", true);
        // if let Some(image) = &mut background_image {
        //     image.current_rotation = 90f64.to_radians();
        //     image.origin = Vector2::new(image.size().x, 0.0);
        //     // image.current_pos = pos - Vector2::new(image.size().x, 0.0);
        //     image.depth = -100.0;
        // }

        let mut button_image= SKIN_MANAGER.write().get_texture("inputoverlay-key", true);
        if let Some(image) = &mut button_image {
            image.depth = -100.1;
        }

        Self {
            counter: KeyCounter::default(),
            
            // background_image,
            button_image,
        }
    }
}
impl InnerUIElement for KeyCounterElement {
    fn get_bounds(&self) -> Rectangle {
        Rectangle::bounds_only(
            -BOX_SIZE,
            Vector2::new(BOX_SIZE.x, BOX_SIZE.y * self.counter.key_order.len() as f64)
        )
    }
    
    fn update(&mut self, manager: &mut IngameManager) {
        self.counter = manager.key_counter.clone();
    }

    fn draw(&mut self, pos_offset: Vector2, scale: Vector2, list: &mut Vec<Box<dyn Renderable>>) {
        let font = get_font();

        // let pad = if let Some((btn, bg)) = self.button_image.as_ref().zip(self.background_image.as_ref()) {
        //     let btn_size = btn.size();
        //     let btn_count = self.key_order.len() as f64;
        //     Vector2::new(
        //         btn_size.x,
        //         (bg.size().y - btn_size.y * btn_count) / btn_count
        //     )
        // } else {
        //     BOX_SIZE
        // };
        let pad = BOX_SIZE;
        
        let base_pos = pos_offset - pad;

        // if let Some(bg) = &self.background_image {
        //     let mut bg = bg.clone();
        //     bg.current_pos = base_pos + Vector2::new(pad.x, pad.y * self.key_order.len() as f64);
        //     list.push(Box::new(bg));
        // }

        //TODO: center properly somehow
        for i in 0..self.counter.key_order.len() {
            let info = &self.counter.keys[&self.counter.key_order[i]];
            let pos = base_pos + Vector2::new(0.0, pad.y * i as f64);

            if let Some(btn) = &self.button_image {
                let mut btn = btn.clone();
                btn.current_pos = pos + pad / 2.0;
                btn.current_scale = scale;
                if info.held {
                    btn.current_scale = Vector2::new(1.1, 1.1) * scale;
                }
                
                list.push(Box::new(btn));
            } else {
                // draw bg box
                list.push(Box::new(Rectangle::new(
                    if info.held {
                        Color::new(0.8, 0.0, 0.8, 0.8)
                    } else {
                        Color::new(0.0, 0.0, 0.0, 0.8)
                    },
                    -100.0,
                    pos,
                    BOX_SIZE * scale,
                    Some(Border::new(Color::BLACK, 2.0))
                )));
            }

            // draw key
            let mut text = Text::new(
                Color::WHITE,
                -100.1,
                pos,
                (20.0 * scale.x) as u32,
                if info.count == 0 {info.label.clone()} else {format!("{}", info.count)},
                font.clone()
            );
            text.center_text(Rectangle::bounds_only(pos, BOX_SIZE));
            list.push(Box::new(text));
        }

    }
}
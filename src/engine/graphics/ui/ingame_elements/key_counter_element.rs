

use crate::prelude::*;

const BOX_SIZE:Vector2 = Vector2::new(40.0, 40.0);

pub struct KeyCounterElement {
    counter: KeyCounter,
    // background_image: Option<Image>,
    button_image: Option<Image>
}
impl KeyCounterElement {
    pub async fn new() -> Self {
        // let mut background_image = SKIN_MANAGER.write().get_texture("inputoverlay-background", true);
        // if let Some(image) = &mut background_image {
        //     image.current_rotation = 90f64.to_radians();
        //     image.origin = Vector2::new(image.size().x, 0.0);
        //     // image.current_pos = pos - Vector2::new(image.size().x, 0.0);
        //     image.depth = -100.0;
        // }
        let mut button_image= SkinManager::get_texture("inputoverlay-key", true).await;
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
    fn display_name(&self) -> &'static str { "Key Counter" }

    fn get_bounds(&self) -> Rectangle {
        let box_size = self.button_image.as_ref().map(|b|b.size()).unwrap_or(BOX_SIZE);
        Rectangle::bounds_only(
            -box_size,
            Vector2::new(box_size.x, box_size.y * self.counter.key_order.len() as f64)
        )
    }
    
    fn update(&mut self, manager: &mut IngameManager) {
        self.counter = manager.key_counter.clone();
    }

    fn draw(&mut self, pos_offset: Vector2, scale: Vector2, list: &mut RenderableCollection) {
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
        let box_size = self.button_image.as_ref().map(|b|b.size()).unwrap_or(BOX_SIZE) * scale;
        
        let base_pos = pos_offset - BOX_SIZE;

        // if let Some(bg) = &self.background_image {
        //     let mut bg = bg.clone();
        //     bg.current_pos = base_pos + Vector2::new(pad.x, pad.y * self.key_order.len() as f64);
        //     list.push(Box::new(bg));
        // }

        //TODO: center properly somehow
        for i in 0..self.counter.key_order.len() {
            let info = &self.counter.keys[&self.counter.key_order[i]];
            let pos = base_pos + Vector2::new(0.0, box_size.y * i as f64);
            let box_width;

            if let Some(btn) = &self.button_image {
                let mut btn = btn.clone();
                btn.current_pos = pos + box_size / 2.0;
                btn.current_scale = scale;
                if info.held {
                    btn.current_scale = Vector2::new(1.1, 1.1) * scale;
                }
                
                box_width = btn.size().x * scale.x;
                list.push(btn);
            } else {
                box_width = (BOX_SIZE * scale).x;

                // draw bg box
                list.push(Rectangle::new(
                    if info.held {
                        Color::new(0.8, 0.0, 0.8, 0.8)
                    } else {
                        Color::new(0.0, 0.0, 0.0, 0.8)
                    },
                    -100.0,
                    pos,
                    BOX_SIZE * scale,
                    Some(Border::new(Color::BLACK, 2.0))
                ));
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
            
            let text_size = text.measure_text();
            let max_width = box_width - 10.0; // padding of 10
            if text_size.x >= max_width {
                text.font_size = FontSize::new((20.0 * scale.x * max_width / text_size.x) as f32).unwrap();
            }

            text.center_text(&Rectangle::bounds_only(pos, box_size));
            list.push(text);
        }

    }
}
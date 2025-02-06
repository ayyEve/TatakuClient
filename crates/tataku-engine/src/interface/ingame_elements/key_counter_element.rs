use crate::prelude::*;

const BOX_SIZE:Vector2 = Vector2::new(40.0, 40.0);

pub struct KeyCounterElement {
    counter: KeyCounter,
    // background_image: Option<Image>,
    button_image: Option<Image>
}
impl KeyCounterElement {
    pub fn new() -> Self {
        Self {
            counter: KeyCounter::default(),
            
            // background_image,
            button_image: None,
        }
    }
}

#[async_trait]
impl InnerUIElement for KeyCounterElement {
    fn display_name(&self) -> &'static str { "Key Counter" }

    fn max_size(&self) -> Vector2 {
        let box_size = self.button_image.as_ref()
            .map(Image::size)
            .unwrap_or(BOX_SIZE);
        Vector2::new(box_size.x, box_size.y * self.counter.key_order.len() as f32)
    }
    
    fn update(&mut self, manager: &mut GameplayManager) {
        self.counter = manager.key_counter.clone();
    }

    #[cfg(feature="graphics")]
    fn draw(
        &mut self, 
        pos_offset: Vector2, 
        scale: Vector2, 
        _align: Alignment,
        list: &mut RenderableCollection
    ) {
        let box_size = self.button_image.as_ref()
            .map(Image::size)
            .unwrap_or(BOX_SIZE) * scale;

        // if let Some(bg) = &self.background_image {
        //     let mut bg = bg.clone();
        //     bg.current_pos = base_pos + Vector2::new(pad.x, pad.y * self.key_order.len() as f64);
        //     list.push(Box::new(bg));
        // }

        for i in 0..self.counter.key_order.len() {
            let info = &self.counter.keys[&self.counter.key_order[i]];
            let pos = pos_offset + Vector2::new(0.0, box_size.y * i as f32);
            let box_width;

            if let Some(mut btn) = self.button_image.clone() {
                btn.pos = pos + box_size / 2.0;
                btn.scale = scale;
                if info.held {
                    btn.scale *= 1.1;
                }
                
                box_width = btn.size().x * scale.x;
                list.push(btn);
            } else {
                box_width = (BOX_SIZE * scale).x;

                // draw bg box
                list.push(Rectangle::new(
                    pos,
                    BOX_SIZE * scale,
                    if info.held {
                        Color::new(0.8, 0.0, 0.8, 0.8)
                    } else {
                        Color::new(0.0, 0.0, 0.0, 0.8)
                    },
                    Some(Border::new(Color::BLACK, 2.0))
                ));
            }

            // draw key
            let mut text = Text::new(
                pos,
                20.0 * scale.x,
                if info.count == 0 {info.label.clone()} else {format!("{}", info.count)},
                Color::WHITE,
                Font::Main
            );
            
            let text_size = text.measure_text();
            let max_width = box_width - 10.0; // padding of 10
            if text_size.x >= max_width {
                text.set_font_size(20.0 * scale.x * max_width / text_size.x)
            }

            text.center_text(&Bounds::new(pos, box_size));
            list.push(text);
        }

    }

    #[cfg(feature="graphics")]
    async fn reload_skin(&mut self, source: &TextureSource, skin_manager: &mut dyn SkinProvider) {
        // let mut background_image = SKIN_MANAGER.write().get_texture("inputoverlay-background", false;
        // if let Some(image) = &mut background_image {
        //     image.current_rotation = 90f64.to_radians();
        //     image.origin = Vector2::new(image.size().x, 0.0);
        //     // image.current_pos = pos - Vector2::new(image.size().x, 0.0);
        //     image.depth = -100.0;
        // }

        self.button_image = skin_manager.get_texture("inputoverlay-key", source, SkinUsage::Gamemode, false).await;
    }
}

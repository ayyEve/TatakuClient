use crate::prelude::*;

macro_rules! number_element {
    (
        $name: ident, $display: expr,
        $t: ty, $precision: expr,
        $format: expr,
        $property: expr,
        $symbol: expr,
        $tex_name: expr,
        $max_number: expr
    ) => {
        pub struct $name {
            image: Option<SkinnedNumber>,
            text: Text,
        
            number: $t,
            max_size: Vector2,
        }
        impl $name {
            pub fn new() -> Self {
                let text = Text::new(Vector2::ZERO, 30.0, format!("{}{}", $format($max_number), $symbol.map(|c| c.to_string()).unwrap_or_default()), Color::WHITE, Font::Main);

                Self {
                    image: None,
                    // combo_size: Text::measure_text_raw(&[Font::Main], 30.0, &format!("{NUMBER}x"), Vector2::ONE, 0.0),
                    number: $max_number as $t,
                    max_size: text.measure_text(),
                    text,
                }
            }
        }


        #[async_trait]
        impl InnerUIElement for $name {
            fn display_name(&self) -> &'static str { $display }
            fn max_size(&self) -> Vector2 { self.max_size }

            fn update(&mut self, manager: &mut GameplayManager) {
                let old_number = self.number;
                self.number = ($property)(manager) as $t;
                // self.combo = manager.score.score.combo;

                if self.number == old_number { return }

                if let Some(image) = &mut self.image {
                    image.number = self.number as f64;
                    // self.size = image.measure_text();
                } else {
                    self.text.text = ($format)(self.number);
                    // self.size = self.text.measure_text();
                }
            }

            #[cfg(feature="graphics")]
            fn draw(
                &mut self, 
                pos_offset: Vector2, 
                scale: Vector2, 
                align: Alignment,
                list: &mut RenderableCollection
            ) {
                if let Some(mut image) = self.image.clone() {
                    image.scale = scale;

                    image.pos = align.resolve(
                        &Bounds::new(pos_offset, self.max_size),
                        image.measure_text(),
                        true,
                        true
                    );

                    list.push(image);
                } else {
                    let mut text = self.text.clone();
                    text.pos = align.resolve(
                        &Bounds::new(pos_offset, self.max_size),
                        text.measure_text(),
                        true,
                        true
                    );
                    text.set_font_size(30.0 * scale.y);
                    list.push(text);
                }
            }

            #[cfg(feature="graphics")]
            async fn reload_skin(&mut self, source: &TextureSource, skin_manager: &mut dyn SkinProvider) {
                self.image = SkinnedNumber::new(
                    Vector2::ZERO, 
                    $max_number as f64, 
                    Color::WHITE, 
                    $tex_name, 
                    $symbol, 
                    $precision, 
                    skin_manager, 
                    source, 
                    SkinUsage::Gamemode
                ).await.ok();
                
                if let Some(image) = &mut self.image {
                    self.max_size = image.measure_text();
                    image.number = self.number as f64;
                }
            }
        }

    }
}


// Score
number_element!(
    ScoreElement, "Score",
    u64, 0,
    format_number,
    |m: &mut GameplayManager| m.score.score.score,
    None::<char>,
    "score",
    1_000_000_000
);

// Combo
number_element!(
    ComboElement, "Combo",
    u16, 0,
    format_number,
    |m: &mut GameplayManager| m.score.score.combo,
    Some('x'),
    "combo",
    5_000
);

// Accuracy
number_element!(
    AccuracyElement, "Accuracy",
    f32, 2,
    |a: f32| format_float(a, 2),
    |m: &mut GameplayManager| m.score.score.accuracy * 100.0,
    Some('%'),
    "score",
    100.0
);

// Performance
number_element!(
    PerformanceElement, "Performance",
    f32, 2,
    |a: f32| format_float(a, 2),
    |m: &mut GameplayManager| m.score.score.performance,
    None::<char>,
    "score",
    10_000.0
);
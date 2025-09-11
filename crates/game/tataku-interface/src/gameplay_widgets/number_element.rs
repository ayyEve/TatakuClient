use crate::prelude::*;
use graphics::SkinnedNumber;
use tataku::{
    Alignment,
    Bounds,
    Vector2,
    Color,
};
use engine::{
    settings::common_gameplay::CommonGameplaySettings,
    gameplay::{
        widgets::*,
        GamemodeInfo,
        gameplay_manager::GameplayManagerTrait,
    },
};

macro_rules! number_element {
    (
        $name: ident, $display: expr,
        $t: ty, $precision: expr,
        $format: expr,
        $property: expr,
        $symbol: expr,
        $tex_name: expr,
        $max_number: expr,

        $id: expr,
        $layout: expr,
        $const_name: ident,

    ) => {
        struct $name {
            image: Option<SkinnedNumber>,
            // text: String,
        
            number: $t,
            max_size: Vector2,
            layout: Option<Arc<parley::Layout<Color>>>,
            layout_size: Vector2,
        }
        impl $name {
            fn layout(
                text: &str,
                scale: Vector2,
                context: &mut ui::widget::TextLayoutContexts,
            ) -> (Arc<parley::Layout<Color>>, Vector2) {
                let style = ui::style::TextStyle {
                    font_size: 30.0 * scale.y,
                    ..Default::default()
                };

                let mut layout = context.simple_text(
                    text,
                    &style,
                );
                layout.break_all_lines(None);

                let size = Vector2::new(
                    layout.width(),
                    layout.height(),
                );

                (Arc::new(layout), size)
            }

            pub fn build(
                _: &GamemodeInfo,
                _: &Arc<CommonGameplaySettings>
            ) -> Box<dyn GameplayWidget> {
                Box::new(Self {
                    image: None,
                    // combo_size: Text::measure_text_raw(&[Font::Main], 30.0, &format!("{NUMBER}x"), Vector2::ONE, 0.0),
                    number: $max_number as $t,

                    layout: None,
                    max_size: Vector2::ZERO,
                    layout_size: Vector2::ZERO,
                    // text,
                })
            }
        }

        impl GameplayWidget for $name {
            fn display_name(&self) -> &'static str { $display }
            fn max_size(&self) -> Vector2 { self.max_size }

            fn update(&mut self, shell: &mut GameplayWidgetUpdateShell) {
                if self.max_size == Vector2::ZERO {
                    let text = format!(
                        "{}{}", 
                        $format($max_number), 
                        $symbol.map(|c| c.to_string()).unwrap_or_default()
                    );

                    let (_, size) = Self::layout(
                        &text,
                        shell.scale,
                        shell.font_context,
                    );
                    self.max_size = size;
                }

                let old_number = self.number;
                self.number = ($property)(shell.manager) as $t;
                if self.number == old_number { return }

                if let Some(image) = &mut self.image {
                    image.number = self.number as f64;
                    self.layout_size = image.measure_text();
                } else {
                    // self.text = ($format)(self.number);
                    // self.size = self.text.measure_text();

                    let (layout, size) = Self::layout(
                        &($format)(self.number),
                        shell.scale,
                        shell.font_context,
                    );

                    self.layout_size = size;
                    self.layout = Some(layout);
                }
            }

            fn draw(&mut self, shell: &mut GameplayWidgetDrawShell) {
                let bounds = Bounds::new(shell.pos_offset, self.max_size * shell.scale);

                if let Some(mut image) = self.image.clone() {
                    image.scale = shell.scale;

                    image.pos = shell.align.resolve(
                        &bounds,
                        image.measure_text(),
                        true,
                        true
                    );

                    shell.list.push(image);
                } else {
                    let Some(layout) = self.layout.clone() 
                    else { return };

                    shell.list.push(graphics::Transformed::new(
                        graphics::Transform::default().translate(shell.align.resolve(
                            &bounds,
                            self.layout_size,
                            true,
                            true
                        )),
                        Box::new(graphics::Text::new(layout))
                    ));

                    // let mut text = self.text.clone();
                    // text.pos = align.resolve(
                    //     &Bounds::new(shell.pos_offset, self.max_size),
                    //     text.measure_text(),
                    //     true,
                    //     true
                    // );
                    // text.set_font_size(30.0 * scale.y);
                    // list.push(text);
                }
            }

            fn reload_skin(
                &mut self, 
                shell: &mut GameplayWidgetReloadSkinShell
            ) {
                self.image = SkinnedNumber::new(
                    Vector2::ZERO, 
                    $max_number as f64, 
                    Color::WHITE, 
                    $tex_name, 
                    $symbol, 
                    $precision, 
                    shell.skin_manager, 
                    shell.source, 
                    graphics::SkinUsage::Gamemode
                ).ok();
                
                if let Some(image) = &mut self.image {
                    // self.max_size = image.measure_text();
                    image.number = self.number as f64;
                }
            }
        }

        pub const $const_name: GameplayWidgetBuilder = GameplayWidgetBuilder {
            name: $id,
            default_layout: $layout,
            build: $name::build,
        };
    }
}

use tataku::{
    format_number,
    format_float,
};

// Score
number_element!(
    ScoreElement, "Score",
    u64, 0,
    format_number,
    |m: &mut dyn GameplayManagerTrait| m.score().score.score,
    None::<char>,
    "score",
    1_000_000_000,

    "score",
    GameplayWidgetLayout::new_default(
        GameplayWidgetAnchor::Screen,
        Alignment::TOP_RIGHT,
        None,
        None,
    ),
    SCORE,
);

// Combo
number_element!(
    ComboElement, "Combo",
    u16, 0,
    format_number,
    |m: &mut dyn GameplayManagerTrait| m.score().score.combo,
    Some('x'),
    "combo",
    5_000,

    "combo",
    GameplayWidgetLayout::new_default(
        GameplayWidgetAnchor::element("duration_bar", GameplayWidgetAlign::Above),
        Alignment::TOP_LEFT,
        None,
        None,
    ),
    COMBO,
);

// Accuracy
number_element!(
    AccuracyElement, "Accuracy",
    f32, 2,
    |a: f32| format_float(a, 2),
    |m: &mut dyn GameplayManagerTrait| m.score().score.accuracy * 100.0,
    Some('%'),
    "score",
    100.0,

    "accuracy", 
    // below score
    GameplayWidgetLayout::new_default(
        GameplayWidgetAnchor::element("score", GameplayWidgetAlign::Below), 
        Alignment::BOTTOM_RIGHT,
        None,
        None,
    ),
    ACCURACY,
);

// Performance
number_element!(
    PerformanceElement, "Performance",
    f32, 2,
    |a: f32| format_float(a, 2),
    |m: &mut dyn GameplayManagerTrait| m.score().score.performance,
    None::<char>,
    "score",
    10_000.0,


    "performance",
    // below acc
    GameplayWidgetLayout::new_default(
        GameplayWidgetAnchor::element("accuracy", GameplayWidgetAlign::Below), 
        Alignment::BOTTOM_RIGHT,
        None,
        None,
    ),
    PERFORMANCE,
);

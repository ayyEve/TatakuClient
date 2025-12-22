use crate::prelude::*;
use graphics::SkinnedNumber;
use tataku::{
    Color,
    Bounds,
    Vector2,
    Alignment,
    format_float,
    format_number,
};
use engine::gameplay::{
    widgets::*,
    gameplay_manager::GameplayManagerTrait,
};

pub struct Number<T: _CanNum> {
    config: NumberConfig<T>,
    image: Option<SkinnedNumber>,
    number: T,
    max_size: Vector2,
    layout: Option<Arc<parley::Layout<Color>>>,
    layout_size: Vector2,
}
impl<T: _CanNum> Number<T> {
    fn new(config: NumberConfig<T>) -> Self {
        Self {
            config,
            number: config.max_number,

            image: None,
            layout: None,
            max_size: Vector2::ZERO,
            layout_size: Vector2::ZERO,
        }
    }

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
}
impl<T: _CanNum> GameplayWidget for Number<T> {
    fn display_name(&self) -> &'static str { self.config.display }
    fn preferred_size(&self) -> Vector2 { self.max_size }

    fn update(&mut self, shell: &mut GameplayWidgetUpdateShell) {
        if self.max_size == Vector2::ZERO {
            let text = format!(
                "{}{}",
                (self.config.format)(self.config.max_number),
                self.config.symbol.map(|c| c.to_string()).unwrap_or_default()
            );

            let (_, size) = Self::layout(
                &text,
                shell.scale,
                shell.font_context,
            );
            self.max_size = size;
        }

        let old_number = self.number;
        self.number = (self.config.property)(shell.manager);
        if self.number == old_number { return }

        if let Some(image) = &mut self.image {
            image.number = self.number.as_f64();
            self.layout_size = image.measure_text();
        } else {
            // self.text = ($format)(self.number);
            // self.size = self.text.measure_text();

            let (layout, size) = Self::layout(
                &(self.config.format)(self.number),
                shell.scale,
                shell.font_context,
            );

            self.layout_size = size;
            self.layout = Some(layout);
        }
    }

    fn draw(&mut self, shell: &mut GameplayWidgetDrawShell) {
        if let Some(image) = self.image.clone() {
            shell.list.push(graphics::Transformed {
                transform: shell.transform,
                drawable: Box::new(image)
            });
        } else {
            let Some(layout) = self.layout.clone()
            else { return };

            shell.list.push(graphics::Transformed {
                transform: shell.transform,
                drawable: Box::new(graphics::Text::new(layout))
            });

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
            self.config.max_number.as_f64(),
            Color::WHITE,
            self.config.tex_name,
            self.config.symbol,
            self.config.precision,
            shell.skin_manager,
            shell.source,
            graphics::SkinUsage::Gamemode
        ).ok();

        if let Some(image) = &mut self.image {
            // self.max_size = image.measure_text();
            image.number = self.number.as_f64();
        }
    }
}

#[derive(Copy, Clone)]
struct NumberConfig<T: _CanNum> {
    display: &'static str,
    tex_name: &'static str,
    max_number: T,
    precision: usize,
    symbol: Option<char>,

    format: fn(T) -> String,
    property: fn(&mut dyn GameplayManagerTrait) -> T,
}

// Score
const SCORE_CONF: NumberConfig<u64> = NumberConfig {
    display: "Score",
    tex_name: "score",
    max_number: 1_000_000_000,
    precision: 0,
    symbol: None,
    format: |n| format_number(&n),
    property: |m: &mut dyn GameplayManagerTrait| m.score().score.score,
};
pub const SCORE:GameplayWidgetBuilder = GameplayWidgetBuilder {
    name: "score",
    default_layout: GameplayWidgetLayout {
        anchor: GameplayWidgetAnchor::Screen,
        align: Alignment::TOP_RIGHT,
        transform: graphics::Transform::identity(),
    },
    build: |_,_| Box::new(Number::new(SCORE_CONF)),
};


// Combo
const COMBO_CONF:NumberConfig<u16> = NumberConfig {
    display: "Combo",
    tex_name: "combo",
    max_number: 10_000,
    precision: 0,
    symbol: Some('x'),
    format: |n| format_number(&n),
    property: |m: &mut dyn GameplayManagerTrait| m.score().score.combo,
};
pub const COMBO:GameplayWidgetBuilder = GameplayWidgetBuilder {
    name: "combo",
    default_layout: GameplayWidgetLayout {
        anchor: GameplayWidgetAnchor::Element {
            element: Cow::Borrowed("duration_bar"),
            horizontal_side: Side::Inside,
            vertical_side: Side::Outside,
        },
        align: Alignment::TOP_LEFT,
        transform: graphics::Transform::identity(),
    },
    build: |_,_| Box::new(Number::new(COMBO_CONF)),
};


// Accuracy
const ACCURACY_CONF: NumberConfig<f32> = NumberConfig {
    display: "Accuracy",
    tex_name: "score",
    max_number: 100.0,
    precision: 2,
    symbol: Some('%'),
    format: |acc: f32| format_float(&acc, 2),
    property: |m: &mut dyn GameplayManagerTrait| m.score().score.accuracy * 100.0,
};
pub const ACCURACY: GameplayWidgetBuilder = GameplayWidgetBuilder {
    name: "accuracy",
    build: |_,_| Box::new(Number::new(ACCURACY_CONF)),
    // below score
    default_layout: GameplayWidgetLayout {
        anchor: GameplayWidgetAnchor::Element {
            element: Cow::Borrowed("score"),
            horizontal_side: Side::Inside,
            vertical_side: Side::Outside,
        },
        align: Alignment::BOTTOM_RIGHT,
        transform: graphics::Transform::identity(),
    },
};


// Performance
const PERFORMANCE_CONF: NumberConfig<f32> = NumberConfig {
    display: "Performance",
    tex_name: "score",
    max_number: 10_000.0,
    precision: 2,
    symbol: None,
    format: |perf: f32| format_float(&perf, 2),
    property: |m: &mut dyn GameplayManagerTrait| m.score().score.performance,
};
pub const PERFORMANCE: GameplayWidgetBuilder = GameplayWidgetBuilder {
    name: "performance",
    build: |_,_| Box::new(Number::new(PERFORMANCE_CONF)),
    // below acc
    default_layout: GameplayWidgetLayout {
        anchor: GameplayWidgetAnchor::Element {
            element: Cow::Borrowed("accuracy"),
            horizontal_side: Side::Inside,
            vertical_side: Side::Outside,
        },
        align: Alignment::BOTTOM_RIGHT,
        transform: graphics::Transform::identity(),
    },
};




pub trait _CanNum: Copy + PartialEq + Send + Sync {
    fn as_f64(self) -> f64;
}
macro_rules! ugh {
    ($($t:ty),*) => {
        $(impl _CanNum for $t {
            fn as_f64(self) -> f64 { self as f64 }
        })*
    };
}
ugh![
    u8, u16, u32, u64, usize,
    i8, i16, i32, i64, isize,
    f32, f64
];

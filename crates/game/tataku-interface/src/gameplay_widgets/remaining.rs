use crate::prelude::*;
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
    },
};

const FONT_SIZE:f32 = 30.0;
const MAX_CHARS:usize = 7; // 1 for neg, 1 for colon, 2 for secs, 3 for mins
const SIZE:Vector2 = Vector2::new(FONT_SIZE * MAX_CHARS as f32, FONT_SIZE);

struct RemainingElement {
    // remaining_image: Option<SkinnedNumber>,
    // remaining_bounds: Bounds,

    speed: f32,
    start_time: f32,
    end_time: f32,

    elapsed: f32,

    mins: i16,
    secs: i16,

    layout: Option<Arc<parley::Layout<Color>>>
}
impl RemainingElement {
    fn build(
        _: &GamemodeInfo,
        _: &Arc<CommonGameplaySettings>
    ) -> Box<dyn GameplayWidget> {
        Box::new(Self {
            // elapsed_image: SkinnedNumber::new(Color::WHITE, -5000.0, Vector2::ZERO, 0.0, "normal", None, 0).await.ok(),
            // elapsed_bounds: Bounds::new(Vector2::ZERO, SIZE),

            speed: 1.0,
            start_time: -1.0,
            end_time: -1.0,
            elapsed: 0.0,

            mins: 0,
            secs: 0,

            layout: None,
        })
    }
}

impl GameplayWidget for RemainingElement {
    fn display_name(&self) -> &'static str { "Time Elapsed" }
    fn max_size(&self) -> Vector2 { SIZE }

    fn update(&mut self, shell: &mut GameplayWidgetUpdateShell) {
        // if the values arent set yet, set them
        if self.start_time == -1.0 {
            self.speed = shell.manager.mods().get_speed();
            self.end_time = shell.manager.end_time() / self.speed;
            self.start_time = self.end_time - shell.manager.metadata().duration / self.speed;
        }

        self.elapsed = shell.manager.time() / self.speed;


        let diff = self.elapsed - self.end_time;
        let sign = if diff < 0.0 {"-"} else {""};
        let secs = (diff / 1000.0).floor().abs();
        let mins = (secs / 60.0).floor() as i16;
        let secs = secs as i16 % 60;

        if self.mins != mins || self.secs != secs {
            self.mins = mins;
            self.secs = secs;

            let mut layout = shell.font_context.simple_text(
                &format!("{sign}{mins:02}:{secs:02}"), 
                &ui::style::TextStyle {
                    color: Color::WHITE,
                    font_size: 30.0 * shell.scale.y,
                    ..Default::default()
                }
            );

            layout.break_all_lines(None);
            self.layout = Some(Arc::new(layout));
        }
    }

    fn draw(&mut self, shell: &mut GameplayWidgetDrawShell) {
        let Some(layout) = self.layout.clone() 
        else { return };

        let bounds = Bounds::new(
            shell.pos_offset,
            SIZE * shell.scale
        );

        shell.list.push(graphics::Transformed::new(
            graphics::Transform::default()
            .translate(Alignment::CENTER.resolve(
                &bounds, 
                Vector2::new(
                    layout.width(),
                    layout.height(),
                ), 
                true, 
                true
            )),
            Box::new(graphics::Text::new(layout))
        ));
    }
}


pub const REMAINING_ELEMENT: GameplayWidgetBuilder = GameplayWidgetBuilder {
    name: "remaining_timer",
    default_layout: GameplayWidgetLayout::new_default(
        GameplayWidgetAnchor::element(
            "judgement_bar", 
            GameplayWidgetAlign::Right
        ),
        Alignment::CENTER_RIGHT,
        Some(Alignment::CENTER_LEFT),
        None,
    ),
    build: RemainingElement::build,
};

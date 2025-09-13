use crate::prelude::*;
use tataku::{
    Alignment,
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

const HIT_TIMING_BAR_SIZE:Vector2 = Vector2::new(300.0, 30.0);
// const HIT_TIMING_BAR_POS:Vector2 = Vector2::new(200.0 - HIT_TIMING_BAR_SIZE.x() / 2.0, -(DURATION_HEIGHT + 3.0 + HIT_TIMING_BAR_SIZE.y() + 5.0));
/// how long should a hit timing line last
pub const HIT_TIMING_DURATION:f32 = 1_500.0;
/// how long to fade out for
const HIT_TIMING_FADE:f32 = 300.0;
/// hit timing bar color
const HIT_TIMING_BAR_COLOR:Color = Color::new(0.0, 0.0, 0.0, 1.0);

struct JudgementBarElement {
    hitbar_timings: Vec<(f32, f32)>,
    judgment_colors: Vec<(f32, Color)>,

    /// not so much miss as it is the largest window
    miss_window: f32,

    game_time: f32
}
impl JudgementBarElement {
    fn build(
        _: &GamemodeInfo,
        _: &Arc<CommonGameplaySettings>
    ) -> Box<dyn GameplayWidget> {
        Box::new(Self {
            judgment_colors: Vec::new(),
            hitbar_timings: Vec::new(),
            miss_window: 9999.0,
            game_time: 0.0,
        })
    }
}
impl GameplayWidget for JudgementBarElement {
    fn display_name(&self) -> &'static str { "Judgement Bar" }

    fn max_size(&self) -> Vector2 {
        // let items_width = HIT_TIMING_BAR_SIZE.x; // * (self.timing_bar_things.0.len() + 1) as f64;
        // Vector2::new(items_width, HIT_TIMING_BAR_SIZE.y)
        HIT_TIMING_BAR_SIZE
    }

    fn update(&mut self, shell: &mut GameplayWidgetUpdateShell) {
        self.game_time = shell.manager.time();
        self.hitbar_timings = shell.manager.hitbar_timings().clone();

        if self.judgment_colors.is_empty() {
            self.judgment_colors = shell.manager.properties().timing_bar_things.clone();

            self.judgment_colors.sort_by(
                |(a, _), (b, _)| b.partial_cmp(a).unwrap()
            );
            self.miss_window = self.judgment_colors.iter()
                .map(|(n,_)| *n)
                .reduce(f32::max)
                .unwrap_or_default();
        }

    }

    fn draw(&mut self, shell: &mut GameplayWidgetDrawShell) {
        // TODO: rework this garbage lmao
        let timing_bar_size = HIT_TIMING_BAR_SIZE * shell.scale;
        
        // draw hit windows
        for (window, color) in &self.judgment_colors {
            let width = (window / self.miss_window) * timing_bar_size.x;
            
            shell.list.push(graphics::Rectangle::new(
                shell.pos_offset + Vector2::new(
                    (timing_bar_size.x - width) / 2.0, 
                    0.0
                ),
                Vector2::new(width, timing_bar_size.y),
                *color,
            ));
        }
        
        // draw hit timings
        for &(hit_time, mut diff) in self.hitbar_timings.iter() {
            diff = if diff < 0.0 { 
                diff.max(-self.miss_window) 
            } else { 
                diff.min(self.miss_window) 
            };

            let pos = (timing_bar_size.x / 2.0) 
                + (diff / self.miss_window) 
                * (timing_bar_size.x / 2.0);


            // draw diff line
            let diff = self.game_time - hit_time;
            let alpha = if diff > HIT_TIMING_DURATION - HIT_TIMING_FADE {
                1.0 - (diff - (HIT_TIMING_DURATION - HIT_TIMING_FADE)) / HIT_TIMING_FADE
            } else { 1.0 };

            shell.list.push(graphics::Rectangle::new(
                shell.pos_offset + Vector2::new(pos, 0.0),
                Vector2::new(2.0, timing_bar_size.y),
                HIT_TIMING_BAR_COLOR.alpha(alpha),
            ));
        }
    }
}


pub const JUDGMENT_BAR: GameplayWidgetBuilder = GameplayWidgetBuilder {
    name: "judgement_bar",
    default_layout: GameplayWidgetLayout::new_default(
        GameplayWidgetAnchor::element(
            "duration_bar", 
            GameplayWidgetAlign::Above
        ), 
        Alignment::TOP_CENTER,
        None,
        None,
    ),
    build: JudgementBarElement::build,
};

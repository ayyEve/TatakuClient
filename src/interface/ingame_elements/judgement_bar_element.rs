use crate::prelude::*;



const HIT_TIMING_BAR_SIZE:Vector2 = Vector2::new(300.0, 30.0);
const HIT_TIMING_BAR_POS:Vector2 = Vector2::new(200.0 - HIT_TIMING_BAR_SIZE.x() / 2.0, -(DURATION_HEIGHT + 3.0 + HIT_TIMING_BAR_SIZE.y() + 5.0));
/// how long should a hit timing line last
pub const HIT_TIMING_DURATION:f32 = 1_000.0;
/// how long to fade out for
const HIT_TIMING_FADE:f32 = 300.0;
/// hit timing bar color
const HIT_TIMING_BAR_COLOR:Color = Color::new(0.0, 0.0, 0.0, 1.0);

pub struct JudgementBarElement {
    hitbar_timings: Vec<(f32, f32)>,
    judgment_colors: Vec<(f32, Color)>,

    /// not so much miss as it is the largest window
    miss_window: f32,

    game_time: f32
}
impl JudgementBarElement {
    pub fn new(mut judgment_colors: Vec<(f32, Color)>) -> Self {
        judgment_colors.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        let miss_window = judgment_colors.iter().fold(0f32, |biggest, (w, _)| biggest.max(*w));

        Self {
            judgment_colors,
            hitbar_timings: Vec::new(),
            miss_window,
            game_time: 0.0,
        }
    }
}
#[async_trait]
impl InnerUIElement for JudgementBarElement {
    fn display_name(&self) -> &'static str { "Judgement Bar" }

    fn get_bounds(&self) -> Bounds {
        let items_width = HIT_TIMING_BAR_SIZE.x; // * (self.timing_bar_things.0.len() + 1) as f64;

        Bounds::new(
            Vector2::new(-items_width/2.0, HIT_TIMING_BAR_POS.y),
            Vector2::new(items_width, HIT_TIMING_BAR_SIZE.y)
        )
    }

    fn update(&mut self, manager: &mut GameplayManager) {
        self.game_time = manager.time();
        self.hitbar_timings = manager.hitbar_timings.clone()
    }

    fn draw(&mut self, pos_offset: Vector2, scale: Vector2, list: &mut RenderableCollection) {
        // TODO: rework this garbage lmao
        // // draw hit timings bar
        // // draw hit timing colors below the bar
        // let (windows, (miss, miss_color)) = ;
        // // draw miss window first
        // list.push(Box::new(Rectangle::new(
        //     *miss_color,
        //     17.1,
        //     pos_offset + Vector2::new(-HIT_TIMING_BAR_SIZE.x/2.0, HIT_TIMING_BAR_POS.y),
        //     HIT_TIMING_BAR_SIZE * scale,
        //     None // for now
        // )));
        let timing_bar_size = HIT_TIMING_BAR_SIZE * scale;

        // since the calcs scale the x, but the x pos does not actually scale, we need to offset it
        let x_offset = Vector2::with_x(timing_bar_size.x - HIT_TIMING_BAR_SIZE.x) / 2.0;
        let pos_offset = pos_offset + x_offset;
        
        // draw other hit windows
        for (window, color) in &self.judgment_colors {
            let width = (window / self.miss_window) * timing_bar_size.x;
            
            list.push(Rectangle::new(
                pos_offset + Vector2::new(-width/2.0, HIT_TIMING_BAR_POS.y),
                Vector2::new(width, timing_bar_size.y),
                *color,
                None // for now
            ));
        }
        
        // draw hit timings
        for &(hit_time, mut diff) in self.hitbar_timings.iter() {
            diff = if diff < 0.0 { diff.max(-self.miss_window) } else { diff.min(self.miss_window) };

            let pos = (diff / self.miss_window) * (timing_bar_size.x / 2.0);
            // let pos = (diff / self.miss_window) as f64 * (HIT_TIMING_BAR_SIZE.x / 2.0);


            // draw diff line
            let diff = self.game_time - hit_time;
            let alpha = if diff > HIT_TIMING_DURATION - HIT_TIMING_FADE {
                1.0 - (diff - (HIT_TIMING_DURATION - HIT_TIMING_FADE)) / HIT_TIMING_FADE
            } else { 1.0 };

            list.push(Rectangle::new(
                pos_offset + Vector2::new(pos, HIT_TIMING_BAR_POS.y),
                Vector2::new(2.0, timing_bar_size.y),
                HIT_TIMING_BAR_COLOR.alpha(alpha),
                None // for now
            ));
        }

        
    }


    async fn reload_skin(&mut self, _skin_manager: &mut SkinManager) {}
}
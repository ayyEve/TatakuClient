use crate::prelude::*;



const HIT_TIMING_BAR_SIZE:Vector2 = Vector2::new(300.0, 30.0);
const HIT_TIMING_BAR_POS:Vector2 = Vector2::new(200.0 - HIT_TIMING_BAR_SIZE.x / 2.0, -(DURATION_HEIGHT + 3.0 + HIT_TIMING_BAR_SIZE.y + 5.0));
/// how long should a hit timing line last
pub const HIT_TIMING_DURATION:f32 = 1_000.0;
/// how long to fade out for
const HIT_TIMING_FADE:f32 = 300.0;
/// hit timing bar color
const HIT_TIMING_BAR_COLOR:Color = Color::new(0.0, 0.0, 0.0, 1.0);

pub struct JudgementBarElement {
    hitbar_timings: Vec<(f32, f32)>,
    timing_bar_things: (Vec<(f32,Color)>, (f32,Color)),

    game_time: f32
}
impl JudgementBarElement {
    pub fn new(timing_bar_things: (Vec<(f32,Color)>, (f32,Color))) -> Self {
        Self {
            timing_bar_things,
            hitbar_timings: Vec::new(),
            game_time: 0.0
        }
    }
}
impl InnerUIElement for JudgementBarElement {
    fn get_bounds(&self) -> Rectangle {
        let window_size = Settings::window_size();
        let items_width = HIT_TIMING_BAR_SIZE.x * (self.timing_bar_things.0.len() + 1) as f64;

        Rectangle::bounds_only(
            Vector2::new((window_size.x-items_width)/2.0, HIT_TIMING_BAR_POS.y),
            Vector2::new(items_width, HIT_TIMING_BAR_SIZE.y)
        )
    }

    fn update(&mut self, manager: &mut IngameManager) {
        self.game_time = manager.time();
        self.hitbar_timings = manager.hitbar_timings.clone()
    }

    fn draw(&mut self, pos_offset: Vector2, scale: Vector2, list: &mut Vec<Box<dyn Renderable>>) {
        let window_size = Settings::window_size();

        // TODO: rework this garbage lmao
        // draw hit timings bar
        // draw hit timing colors below the bar
        let (windows, (miss, miss_color)) = &self.timing_bar_things;
        // draw miss window first
        list.push(Box::new(Rectangle::new(
            *miss_color,
            17.1,
            pos_offset + Vector2::new((window_size.x-HIT_TIMING_BAR_SIZE.x)/2.0, HIT_TIMING_BAR_POS.y),
            HIT_TIMING_BAR_SIZE * scale,
            None // for now
        )));
        // draw other hit windows
        for (window, color) in windows {
            let width = (window / miss) as f64 * HIT_TIMING_BAR_SIZE.x;
            list.push(Box::new(Rectangle::new(
                *color,
                17.0,
                pos_offset + Vector2::new((window_size.x - width)/2.0, HIT_TIMING_BAR_POS.y),
                Vector2::new(width, HIT_TIMING_BAR_SIZE.y) * scale,
                None // for now
            )));
        }
        
        // draw hit timings
        for (hit_time, diff) in self.hitbar_timings.as_slice() {
            let hit_time = hit_time.clone();
            let mut diff = diff.clone();
            if diff < 0.0 {
                diff = diff.max(-miss);
            } else {
                diff = diff.min(*miss);
            }

            let pos = (diff / miss) as f64 * (HIT_TIMING_BAR_SIZE.x / 2.0);

            // draw diff line
            let diff = self.game_time - hit_time;
            let alpha = if diff > HIT_TIMING_DURATION - HIT_TIMING_FADE {
                1.0 - (diff - (HIT_TIMING_DURATION - HIT_TIMING_FADE)) / HIT_TIMING_FADE
            } else {1.0};

            list.push(Box::new(Rectangle::new(
                HIT_TIMING_BAR_COLOR.alpha(alpha),
                10.0,
                pos_offset + Vector2::new(window_size.x / 2.0 + pos, HIT_TIMING_BAR_POS.y),
                Vector2::new(2.0, HIT_TIMING_BAR_SIZE.y) * scale,
                None // for now
            )));
        }

        
    }
}
use crate::prelude::*;
use tataku::{ Vector2, Color };
use std::sync::atomic::{ AtomicU32, Ordering };

const SIZE:Vector2 = Vector2::new(180.0, 20.0);
const TEXT_PADDING:Vector2 = Vector2::new(0.0, 2.0);

/// fps display helper, cleans up some of the code in game
pub struct FpsDisplay {
    name: String,
    pos: Vector2,
    pos_count: u8,
    // number_image: Option<SkinnedNumber>,

    count: CountProvider,
    last: f32, 
    timer: tataku::Instant,

    frametime: CountProvider,
    frametime_last: f32,
    frametime_timer: tataku::Instant,
}
impl FpsDisplay {
    /// name is what to display in text, count is which fps counter is this (only affects position)
    pub fn new_atomic(
        name: &str,
        pos_count: u8,
        count: Arc<AtomicU32>,
        frametime_last: Arc<AtomicU32>
    ) -> Self {
        Self::new(
            name,
            pos_count,
            count.into(),
            frametime_last.into()
        )
    }
    /// name is what to display in text, count is which fps counter is this (only affects position)
    pub fn new_counter(
        name: &str,
        pos_count: u8,
    ) -> Self {
        Self::new(
            name,
            pos_count,
            CountProvider::U32(0),
            CountProvider::F32(0.0),
        )
    }

    fn new(
        name: &str,
        pos_count: u8,
        count: CountProvider,
        frametime: CountProvider
    ) -> Self {
        Self {
            count,
            frametime,
            pos_count,
            // number_image: SkinnedNumber::new(Color::BLACK, 0.0, pos, 0.0, "fps", None, 2).await.ok(),

            last: 0.0,
            frametime_last: 0.0,
            frametime_timer: tataku::Instant::now(),
            timer: tataku::Instant::now(),
            name: name.to_owned(),
            pos: Vector2::ZERO,
        }
    }

    pub fn increment(&mut self) {
        let CountProvider::U32(count) = &mut self.count 
        else { return };
        *count += 1;

        let CountProvider::F32(frametime) = &mut self.frametime 
        else { return };

        *frametime = self.frametime_timer.as_millis().max(*frametime);
        self.frametime_timer = tataku::Instant::now();
    }

    pub fn window_size_changed(&mut self, window_size: Vector2) {
        self.pos = window_size - Vector2::new(
            SIZE.x,
            SIZE.y * (self.pos_count+1) as f32
        );
    }

    pub fn update(&mut self) {
        let elapsed = self.timer.as_millis();
        if elapsed >= 100.0 {
            // reset timer
            self.timer = tataku::Instant::now();

            // update frametime and last updates/s
            let frametime = self.frametime.get_and_reset();
            if matches!(self.frametime, CountProvider::Atomic(_)) {
                self.frametime_last = frametime / 100.0; // restore 2 decimal places
            } else {
                self.frametime_last = frametime;
            }
            self.last = self.count.get_and_reset() / elapsed * 1000.0;
            
            
            // if let Some(n) = &mut self.number_image {
            //     n.number = self.frametime_last_draw as f64;
            // }
        }
    }

    pub fn draw(
        &self,
        list: &mut graphics::RenderableCollection,
        text_layout_contexts: &mut ui::widget::TextLayoutContexts,
    ) {
        list.push(graphics::Rectangle::new(
            self.pos,
            SIZE,
            Color::WHITE.alpha(0.8),
        ));

        let text = format!("{:.2} {} ({:.2}ms)", self.last, self.name, self.frametime_last);

        let mut layout = text_layout_contexts.simple_text(
            &text,
            &ui::style::TextStyle {
                font_size: 12.0,
                color: Color::BLACK,
                ..Default::default()
            },
        );

        layout.break_all_lines(Some(SIZE.x));
        let transform = graphics::Transform::default()
            .translate(self.pos + TEXT_PADDING);

        list.push(graphics::Transformed::new(
            transform,
            Box::new(graphics::Text::new(layout.clone())),
        ));
        
    }
}

enum CountProvider {
    U32(u32),
    F32(f32),
    Atomic(Arc<AtomicU32>),
}
impl CountProvider {
    fn get_and_reset(&mut self) -> f32 {
        match self {
            Self::F32(n) => n.take(),
            Self::U32(n) => n.take() as f32,
            Self::Atomic(a) => a.swap(0, Ordering::Acquire) as f32,
        }
    }
}
impl From<Arc<AtomicU32>> for CountProvider {
    fn from(value: Arc<AtomicU32>) -> Self {
        Self::Atomic(value)
    }
}


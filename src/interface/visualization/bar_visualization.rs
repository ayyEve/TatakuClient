use crate::prelude::*;
use super::Visualization;

const CUTOFF:f32 = 0.1;

pub struct BarVisualization {
    data: Vec<FFTData>,
    // bars: Vec<Bar>,

    timer: Instant, // external use only

    min_index: usize,
    max_index: usize,

    bounds: Bounds,
}
impl BarVisualization {
    pub async fn new(min_index: usize, max_index: usize, bounds: Bounds) -> Self {
        Self {
            data: Vec::new(),
            // bars: Vec::new(),
            timer: Instant::now(),

            min_index,
            max_index,
            bounds
        }
    }

    pub async fn update(&mut self) {

    }

    fn get_color(&self, _i: usize) -> (Color, Color) {
        (Color::BABY_BLUE, Color::BLUE)
    }

    #[inline]
    fn max_height(&self) -> f32 {
        self.bounds.size.y
    }
}

#[async_trait]
impl Visualization for BarVisualization {
    fn lerp_factor(&self) -> f32 {10.0} // 15
    fn data(&mut self) -> &mut Vec<FFTData> { &mut self.data }
    fn timer(&mut self) -> &mut Instant { &mut self.timer }

    async fn draw(&mut self, pos_offset:Vector2, list: &mut RenderableCollection) {
        // let since_last = self.timer.as_millis() * 1000.0; // not ms
        self.update_data().await;
        if self.data.len() < 3 { return }
        
        let bar_width = self.bounds.size.x / (self.max_index - self.min_index) as f32;
        let pos = self.bounds.pos + pos_offset;

        for i in self.min_index..self.max_index {
            let Some(data) = self.data.get(i) else { return };
            let (fill, border) = self.get_color(i);

            let amplitude = data.amplitude() / 500.0;
            if amplitude <= CUTOFF { continue }

            let factor = (i as f32 + 2.0).log10();

            let pos = pos + Vector2::with_x(bar_width * i as f32);
            let size = Vector2::new(bar_width, amplitude * factor * self.max_height());

            list.push(Rectangle::new(pos, size, fill, Some(Border::new(border, 2.0))));
        }
        
    }

    fn reset(&mut self) {
        self.data.clear();
        // self.timer = Instant::now();
    }
}


// struct Bar {
//     height: f32,
//     color: Color,
// }

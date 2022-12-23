use crate::prelude::*;

pub struct PieGraph {
    // min: f32,
    sum: f32,
    data: Arc<Vec<MenuStatsEntry>>
}
impl PieGraph {
    pub fn new(data: Arc<Vec<MenuStatsEntry>>) -> Self {
        // let mut min = f32::MAX;
        let mut sum = 0.0;

        for i in data.iter() {
            sum += i.get_value()
        }

        Self {
            sum, 
            data
        }
    }

}

impl StatsGraph for PieGraph {
    fn draw(&self, bounds: &Rectangle, depth: f64, list: &mut Vec<Box<dyn Renderable>>) {
        let pos = bounds.current_pos;
        let size = bounds.size;
        let radius = size.x / 2.0;

        // background
        list.push(Box::new(Rectangle::new(
            Color::new(0.2, 0.2, 0.2, 0.7),
            depth,
            pos,
            size,
            Some(Border::new(Color::RED, 1.5))
        )));

        // // mid
        // list.push(Box::new(Line::new(
        //     pos + Vector2::new(0.0, size.y / 2.0),
        //     pos + Vector2::new(size.x, size.y / 2.0),
        //     LINE_WIDTH,
        //     parent_depth,
        //     Color::WHITE
        // )));

        let center = pos + size / 2.0;
        let mut last_theta = -PI / 2.0;

        for i in self.data.iter().rev() {
            let theta = (i.get_value() / self.sum) as f64 * 2.0 * PI;

            // arc
            list.push(Box::new(Sector::new(
                center, 
                radius,
                last_theta,
                last_theta + theta,
                i.color,
                depth,
                None
            )));

            last_theta += theta
        }

    }

}


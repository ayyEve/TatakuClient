use crate::prelude::*;

#[derive(Clone)]
pub struct PieGraph {
    // min: f32,
    sum: f32,
    data: Arc<Vec<MenuStatsEntry>>,
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

    
    pub fn draw(&self, bounds: &Bounds) -> TransformGroup {
        let mut group = TransformGroup::new(bounds.pos);
        let size = bounds.size;
        let radius = size.x / 2.0;

        // background
        group.push(Rectangle::new(
            Vector2::ZERO,
            size,
            Color::new(0.2, 0.2, 0.2, 0.7),
            Some(Border::new(Color::RED, 1.5))
        ));

        // // mid
        // group.push(Box::new(Line::new(
        //     Vector2::new(0.0, size.y / 2.0),
        //     Vector2::new(size.x, size.y / 2.0),
        //     LINE_WIDTH,
        //     parent_depth,
        //     Color::WHITE
        // )));

        let center = size / 2.0;
        let mut last_theta = -PI / 2.0;

        for i in self.data.iter().rev() {
            let theta = (i.get_value() / self.sum) * 2.0 * PI;

            // arc
            group.push(Sector::new(
                center, 
                radius,
                last_theta,
                last_theta + theta,
                i.color,
                None
            ));

            last_theta += theta
        }

        group
    }

}

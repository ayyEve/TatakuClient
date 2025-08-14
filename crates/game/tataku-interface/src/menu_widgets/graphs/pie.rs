use crate::prelude::*;

#[derive(Clone)]
pub struct PieGraph {
    // min: f32,
    sum: f32,
    data: Arc<Vec<StatsEntry>>,
}
impl PieGraph {
    pub fn new(data: Arc<Vec<StatsEntry>>) -> Self {
        // let mut min = f32::MAX;
        let mut sum = 0.0;

        for i in data.iter() {
            sum += i.get_value();
        }

        Self {
            sum, 
            data
        }
    }

    pub fn draw(&self, bounds: &Bounds) -> RenderableCollection {
        let mut collection = RenderableCollection::new();
        let size = bounds.size;
        let radius = size.x / 2.0;

        // background
        collection.push(
            Rectangle::new(
                bounds.pos,
                size,
                Color::new(0.2, 0.2, 0.2, 0.7),
            )
            .border(Border::new(Color::RED, 1.5))
        );

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
            collection.push(Sector::new(
                bounds.pos + center,
                radius,
                last_theta,
                last_theta + theta,
                i.color,
                None
            ));

            last_theta += theta;
        }

        collection
    }

}

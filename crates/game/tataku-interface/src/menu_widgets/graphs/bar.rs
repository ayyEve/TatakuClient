use crate::prelude::*;
use tataku::{
    Color,
    Border,
    Bounds,
    Vector2,
};
use engine::gameplay::stats::{
    StatsValue,
    StatsEntry,
};

#[derive(Clone)]
pub struct BarGraph {
    min: f32,
    max: f32,
    data: Arc<Vec<StatsEntry>>
}
impl BarGraph {
    pub fn new(data: Arc<Vec<StatsEntry>>) -> Self {
        let mut min = f32::MAX;
        let mut max = f32::MIN;

        for i in data.iter() {
            match &i.value {
                StatsValue::Single(val) => {
                    min = min.min(*val);
                    max = max.max(*val);
                }
                StatsValue::List(list) => {
                    for val in list {
                        min = min.min(*val);
                        max = max.max(*val);
                    }
                }
            }
        }

        Self {
            min, 
            max, 
            data
        }
    }

    fn map_point(&self, point: f32, size: Vector2) -> f32 {
        (self.max - point.clamp(self.min, self.max)) * size.y / (self.max - self.min).abs()
    }
    fn map_points(&self, data: &[f32], size: Vector2) -> Vec<f32> {
        data.iter().map(|x| (self.max - x.clamp(self.min, self.max)) * size.y / (self.max - self.min).abs()).collect()
    }

    
    pub fn draw(&self, bounds: &Bounds) -> graphics::RenderableCollection {
        let mut collection = graphics::RenderableCollection::default();

        let size = bounds.size;

        // background
        collection.push(
            graphics::Rectangle::new(
                size,
                Color::new(0.2, 0.2, 0.2, 0.7),
            )
            .border(Border::new(Color::RED, 1.5))
            .with_transform(tataku::Matrix::identity()
                .trans(bounds.pos)
            )
        );

        // // mid
        // list.push(Line::new(
        //     pos + Vector2::new(0.0, size.y / 2.0),
        //     pos + Vector2::new(size.x, size.y / 2.0),
        //     LINE_WIDTH,
        //     parent_depth,
        //     Color::WHITE
        // ));

        for i in self.data.iter() {
            match &i.value {
                StatsValue::Single(v) => {
                    let v = self.map_point(*v, size);

                    collection.push(graphics::Line::new(
                        size.x_portion(),
                        2.0,
                        i.color,
                    ).with_transform(tataku::Matrix::identity()
                        .trans(bounds.pos + Vector2::with_y(v))
                    ));
                }
                StatsValue::List(points) => {
                    let mapped_points = self.map_points(points, size);
                    
                    let mut prev_y = mapped_points[0];
                    let x_step = size.x / mapped_points.len() as f32;

                    for (n, new_y) in mapped_points.iter().copied().enumerate().skip(1) {
                        let start = Vector2::new(x_step * (n-1) as f32, prev_y);
                        let end = Vector2::new(x_step * n as f32, new_y);

                        collection.push(graphics::Line::new(
                            end - start,
                            2.0,
                            i.color
                        ).with_transform(tataku::Matrix::identity()
                            .trans(bounds.pos + start)
                        ));

                        prev_y = new_y;
                    }
                    
                }
            }
            
        }

        collection
    }

}

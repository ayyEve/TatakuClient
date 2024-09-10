use crate::prelude::*;

#[derive(Clone)]
pub struct BarGraph {
    min: f32,
    max: f32,
    data: Arc<Vec<MenuStatsEntry>>
}
impl BarGraph {
    pub fn new(data: Arc<Vec<MenuStatsEntry>>) -> Self {
        let mut min = f32::MAX;
        let mut max = f32::MIN;

        for i in data.iter() {
            match &i.value {
                MenuStatsValue::Single(val) => {
                    min = min.min(*val);
                    max = max.max(*val);
                }
                MenuStatsValue::List(list) => {
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
    fn map_points(&self, data: &Vec<f32>, size: Vector2) -> Vec<f32> {
        data.iter().map(|x| (self.max - x.clamp(self.min, self.max)) * size.y / (self.max - self.min).abs()).collect()
    }

    
    pub fn draw(&self, bounds: &Bounds) -> TransformGroup {
        let mut group = TransformGroup::new(bounds.pos);
        let size = bounds.size;

        // background
        group.push(Rectangle::new(
            Vector2::ZERO,
            size,
            Color::new(0.2, 0.2, 0.2, 0.7),
            Some(Border::new(Color::RED, 1.5))
        ));

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
                MenuStatsValue::Single(v) => {
                    let v = self.map_point(*v, size);

                    group.push(Line::new(
                        Vector2::with_y(v),
                        size.x_portion() + Vector2::with_y(v),
                        2.0,
                        i.color,
                    ))
                }
                MenuStatsValue::List(points) => {
                    let mapped_points = self.map_points(&points, size);
                    
                    let mut prev_y = mapped_points[0];
                    let x_step = size.x / mapped_points.len() as f32;

                    for n in 1..mapped_points.len() {
                        let new_y = mapped_points[n];
                        group.push(Line::new(
                            Vector2::new(x_step * (n-1) as f32, prev_y),
                            Vector2::new(x_step * n as f32, new_y),
                            2.0,
                            i.color
                        ));

                        prev_y = new_y;
                    }
                    
                }
            }
            
        }

        group
    }

}

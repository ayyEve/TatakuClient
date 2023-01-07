use crate::prelude::*;

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

    fn map_point(&self, point: f32, size: Vector2) -> f64 {
        (self.max - point.clamp(self.min, self.max)) as f64 * size.y / (self.max - self.min).abs() as f64
    }
    fn map_points(&self, data: &Vec<f32>, size: Vector2) -> Vec<f64> {
        data.iter().map(|x| (self.max - x.clamp(self.min, self.max)) as f64 * size.y / (self.max - self.min).abs() as f64).collect()
    }


}

impl StatsGraph for BarGraph {
    fn draw(&self, bounds: &Rectangle, depth: f64, list: &mut RenderableCollection) {
        let pos = bounds.pos;
        let size = bounds.size;

        // background
        list.push(Rectangle::new(
            Color::new(0.2, 0.2, 0.2, 0.7),
            depth,
            pos,
            size,
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

                    list.push(Line::new(
                        pos + Vector2::with_y(v),
                        pos + size.x_portion() + Vector2::with_y(v),
                        2.0,
                        depth,
                        i.color,
                    ))
                }
                MenuStatsValue::List(points) => {
                    let mapped_points = self.map_points(&points, size);
                    
                    let mut prev_y = mapped_points[0];
                    let x_step = size.x / mapped_points.len() as f64;

                    for n in 1..mapped_points.len() {
                        let new_y = mapped_points[n];
                        list.push(Line::new(
                            pos + Vector2::new(x_step * (n-1) as f64, prev_y),
                            pos + Vector2::new(x_step * n as f64, new_y),
                            2.0,
                            depth + 1.0,
                            i.color
                        ));

                        prev_y = new_y;
                    }
                    
                }
            }
            
        }

    }

}
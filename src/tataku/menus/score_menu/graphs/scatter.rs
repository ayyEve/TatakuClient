use crate::prelude::*;

pub struct ScatterGraph {
    min: f32,
    max: f32,
    data: Arc<Vec<MenuStatsEntry>>
}
impl ScatterGraph {
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


}


impl StatsGraph for ScatterGraph {
    fn draw(&self, bounds: &Rectangle, depth: f32, list: &mut RenderableCollection) {
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
        
        // 0 line
        let zero_pos = Vector2::with_y(self.map_point(0.0, size));
        list.push(Line::new(
            pos + zero_pos,
            pos + size.x_portion() + zero_pos,
            1.5,
            depth,
            Color::WHITE,
        ));

        for i in self.data.iter() {
            match &i.value {
                MenuStatsValue::Single(v) => {
                    let v = self.map_point(*v, size);

                    list.push(Line::new(
                        pos + Vector2::with_y(v),
                        pos + size.x_portion() + Vector2::with_y(v),
                        1.5,
                        depth,
                        i.color,
                    ))
                }
                MenuStatsValue::List(points) => {
                    let mapped_points = self.map_points(&points, size);
                    let x_step = size.x / mapped_points.len() as f32;

                    for (n, &y) in mapped_points.iter().enumerate() {
                        let mut c = Circle::new(
                            i.color,
                            depth,
                            pos + Vector2::new(x_step * n as f32, y),
                            2.0,
                            None
                        );
                        c.resolution = 32;
                        list.push(c);
                    }
                    
                }
            }
            
        }

    }

}
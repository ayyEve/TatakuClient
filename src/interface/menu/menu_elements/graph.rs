use crate::prelude::*;

const LINE_WIDTH:f32 = 1.0;

#[derive(ScrollableGettersSetters)]
pub struct Graph {
    pos: Vector2,
    size: Vector2,
    hover: bool,

    pub data_points: Vec<f32>,
    mapped_points: Vec<f32>,
    mouse_pos: Vector2,

    min: f32,
    max: f32,


    pub font: Font,
    pub font_size: f32,
}
impl Graph {
    pub fn new(pos: Vector2, size: Vector2, data_points: Vec<f32>, min: f32, max: f32, font: Font) -> Self {
        let mapped_points = data_points.iter().map(|x| (max - x.clamp(min, max)) * size.y / (max - min).abs()).collect();
        Self {
            pos,
            size,
            hover: false,

            data_points,
            mapped_points,
            mouse_pos: Vector2::ONE * -10.0,
            min,
            max,

            font,
            font_size: 12.0,
        }
    }
}

impl ScrollableItem for Graph {
    fn draw(&mut self, pos_offset:Vector2, parent_depth:f32, list: &mut RenderableCollection) {
        // list.reserve(self.data_points.len() + 2);
        
        // background
        list.push(Rectangle::new(
            Color::new(0.2, 0.2, 0.2, 0.7),
            parent_depth,
            self.pos + pos_offset,
            self.size,
            Some(Border::new(Color::RED, 1.5))
        ));
        // mid
        list.push(Line::new(
            self.pos + pos_offset + Vector2::new(0.0, self.size.y / 2.0),
            self.pos + pos_offset + Vector2::new(self.size.x, self.size.y / 2.0),
            LINE_WIDTH,
            parent_depth,
            Color::WHITE
        ));

        // if theres no data points to draw, return
        if self.data_points.len() == 0 {return}

        let mut prev_y = self.mapped_points[0];
        let x_step = self.size.x / self.data_points.len() as f32;

        for i in 1..self.mapped_points.len() {
            let new_y = self.mapped_points[i];
            list.push(Line::new(
                self.pos + pos_offset + Vector2::new(x_step * (i-1) as f32, prev_y),
                self.pos + pos_offset + Vector2::new(x_step * i as f32, new_y),
                LINE_WIDTH,
                parent_depth + 1.0,
                Color::BLACK
            ));

            if self.get_hover() {
                // draw circles on the points
                list.push(Circle::new(
                    Color::BLUE,
                    parent_depth + 0.5,
                    self.pos + pos_offset + Vector2::new(x_step * i as f32, new_y),
                    LINE_WIDTH * 2.0,
                    None
                ));
            }

            prev_y = new_y;
        }

        if self.get_hover() {
            // draw vertical line at mouse pos
            list.push(Line::new(
                Vector2::new(self.mouse_pos.x, self.pos.y +pos_offset.y),
                Vector2::new(self.mouse_pos.x, self.pos.y + pos_offset.y + self.size.y),
                LINE_WIDTH,
                parent_depth - 1.0,
                Color::RED
            ));
        }
    }
    fn on_mouse_move(&mut self, p:Vector2) {
        self.mouse_pos = p;
        self.check_hover(p);
    }
}

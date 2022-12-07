use crate::prelude::*;

const X_OFFSET:f64 = 10.0;
const ITEM_PADDING:usize = 2;

#[allow(dead_code)]
#[derive(Clone)]
pub struct MainMenuButton {
    pos: Vector2,
    size: Vector2,
    hover: bool,
    selected: bool,
    text: String,

    shapes: TransformGroup,
    disposable_shapes: Vec<TransformGroup>,
    pub visible: bool,
    timer: Instant,

    pub window_size: Vector2,

    last_num: usize,
    last_count: usize,

    hide_time: f32,
}
impl MainMenuButton {
    pub fn new(_pos: Vector2, size: Vector2, text:&str) -> MainMenuButton {
        let pos = Vector2::zero();
        let shapes = TransformGroup::new();
        let window_size = WindowSize::get().0;

        MainMenuButton {
            pos, 
            size, 
            text: text.to_owned(),

            hover: false,
            selected: false,

            shapes,
            disposable_shapes: Vec::new(),
            visible: true,
            timer: Instant::now(),

            window_size,

            last_num: 0,
            last_count: 0,

            hide_time: -1.0
        }
    }

    /// num: this button number, count: number of buttons
    pub fn show(&mut self, num: usize, count: usize, do_transform: bool) {
        if self.visible { return }

        let time = self.time();
        self.visible = true;
        self.last_num = num;
        self.last_count = count;
        self.hide_time = -1.0;


        let radius = (self.window_size.y / 6.0) * VISUALIZATION_SIZE_FACTOR + X_OFFSET;
        let center = self.window_size / 2.0;


        let height = self.size.y;
        let angle = (PI / (count + 2 * ITEM_PADDING - 1) as f64) * (num + ITEM_PADDING) as f64 - PI / 2.0;
        let end = center + Vector2::new(
            angle.cos() * radius,
            angle.sin() * radius,
        ) - Vector2::new(0.0, height / 2.0);

        let duration = if do_transform { 500.0 } else { 1.0 };
        let start = Vector2::new(
            center.x,
            end.y
        );

        let t1 = Transformation::new(
            0.0,
            duration,
            TransformType::Position {start, end},
            TransformEasing::Linear,
            time
        );
        let t2 = Transformation::new(
            0.0,
            duration,
            TransformType::Transparency { start: 0.0, end: 1.0 },
            TransformEasing::Linear,
            time
        );

        self.shapes.transforms.push(t1);
        self.shapes.transforms.push(t2);

        for i in self.disposable_shapes.iter_mut() {
            i.transforms.push(t1);
            i.transforms.push(t2);
        }

    }


    /// num: this button number, count: number of buttons
    pub fn hide(&mut self, num: usize, count: usize, do_transform: bool) {
        if !self.visible { return }

        let time = self.time();
        self.visible = true;
        self.last_num = num;
        self.last_count = count;
        self.hide_time = time as f32;


        let radius = (self.window_size.y / 6.0) * VISUALIZATION_SIZE_FACTOR + X_OFFSET;
        let center = self.window_size / 2.0;


        let height = self.size.y;
        let angle = (PI / (count + 2 * ITEM_PADDING - 1) as f64) * (num + ITEM_PADDING) as f64 - PI / 2.0;
        let mut end = center + Vector2::new(
            angle.cos() * radius,
            angle.sin() * radius,
        ) - Vector2::new(0.0, height / 2.0);

        let duration = if do_transform { 500.0 } else { 1.0 };
        let mut start = Vector2::new(
            center.x,
            end.y
        );

        std::mem::swap(&mut end, &mut start);

        let t1 = Transformation::new(
            0.0,
            duration,
            TransformType::Position {start, end},
            TransformEasing::Linear,
            time
        );
        

        let t2 = Transformation::new(
            0.0,
            duration,
            TransformType::Transparency { start: 1.0, end: 0.0 },
            TransformEasing::Linear,
            time
        );

        self.shapes.transforms.push(t1);
        self.shapes.transforms.push(t2);

        for i in self.disposable_shapes.iter_mut() {
            i.transforms.push(t1);
            i.transforms.push(t2);
        }

    }
    

    pub fn time(&self) -> f64 {
        self.timer.as_millis64()
    }

    pub fn window_size_changed(&mut self, window_size: &Arc<WindowSize>) {
        self.window_size = window_size.0;

        if self.visible {
            self.visible = false;
            self.show(self.last_num, self.last_count, false);
        }
    }
}
impl ScrollableItemGettersSetters for MainMenuButton {
    fn size(&self) -> Vector2 {self.size}
    fn get_pos(&self) -> Vector2 {
        self.shapes.items.get(0).map(|i|i.get_pos()).unwrap_or_default()
    }

    fn get_hover(&self) -> bool {self.hover}
    fn set_hover(&mut self, mut hover:bool) {
        if !self.visible {hover = false}
        self.hover = hover;

        let size = if hover {
            1.0
        } else {
            0.0
        };

        let transform = Transformation::new(
            0.0, 
            1.0,
            TransformType::BorderSize {start: size, end: size},
            TransformEasing::Linear,
            self.time()
        );

        self.shapes.transforms.push(transform);
    }
    fn get_selected(&self) -> bool {self.selected}
    fn set_selected(&mut self, mut selected:bool) {
        if !self.visible {selected = false}
        self.selected = selected;
        trace!("setting selected: {}", selected);

        if selected {
            let transform2 = Transformation::new(
                0.0, 
                1.0,
                TransformType::BorderSize {start: 1.0, end: 1.0},
                TransformEasing::Linear,
                self.time()
            );
            let transform = Transformation::new(
                1.0, 
                1.0,
                TransformType::BorderColor {start: Color::BLUE, end: Color::BLUE},
                TransformEasing::Linear,
                self.time()
            );

            self.shapes.transforms.push(transform);
            self.shapes.transforms.push(transform2);
        } else {
            self.set_hover(self.hover)
        }
    }
    fn get_selectable(&self) -> bool {false}
}
impl ScrollableItem for MainMenuButton {
    fn update(&mut self) {
        if self.shapes.items.len() == 0 {
            let font_size: u32 = 15;

            // draw box
            let r = Rectangle::new(
                [0.2, 0.2, 0.2, 1.0].into(),
                10.0,
                self.pos,
                self.size,
                Some(Border::new(Color::RED, 0.0))
            ).shape(Shape::Round(5.0, 10));
            
            // draw text
            let mut txt = Text::new(
                Color::WHITE,
                9.0,
                Vector2::zero(),
                font_size,
                self.text.to_owned(),
                get_font()
            );
            txt.center_text(r);

            
            self.shapes.items.push(DrawItem::Rectangle(r));
            self.shapes.items.push(DrawItem::Text(txt));
        }


        let time = self.timer.elapsed().as_secs_f64() * 1000.0;
        self.shapes.update(time);

        self.disposable_shapes.retain_mut(|i|{
            i.update(time);
            i.items.iter().find(|s|s.visible()).is_some()
        });


        if self.hide_time > 0.0 {
            if time - self.hide_time as f64 > 500.0 {
                self.visible = false;
            }
        }


    }

    fn draw(&mut self, _args:piston::RenderArgs, _pos_offset:Vector2, _parent_depth:f64, list:&mut Vec<Box<dyn Renderable>>) {
        if !self.visible { return }
        self.shapes.draw(list);

        for i in self.disposable_shapes.iter_mut() {
            i.draw(list);
        }
    }
}


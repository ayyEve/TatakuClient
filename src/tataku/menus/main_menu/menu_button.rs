use crate::prelude::*;

const X_OFFSET:f32 = 10.0;
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

    image: Option<Image>,
    hover_image: Option<Image>,
}
impl MainMenuButton {
    pub async fn new(_pos: Vector2, mut size: Vector2, text:&str, tex_name: &str) -> MainMenuButton {
        let pos = Vector2::ZERO;
        let shapes = TransformGroup::new(pos).alpha(1.0).border_alpha(0.0);
        let window_size = WindowSize::get().0;

        let mut image = SkinManager::get_texture(tex_name, true).await;
        let mut hover_image = SkinManager::get_texture(format!("{tex_name}-over"), true).await;

        // limit by height, not width
        if let Some(image) = &mut image {
            image.scale = Vector2::ONE * (size.y / image.tex_size().y);
            image.origin.y = 0.0;
            image.origin.x -= image.tex_size().x / 3.0;
            size = image.size();
            if let Some(hover) = &mut hover_image {
                hover.origin.y = 0.0;
                hover.origin.x -= hover.tex_size().x / 3.0;
                hover.scale = image.scale;
            }
        }

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

            hide_time: -1.0,
            image,
            hover_image,
        }
    }

    /// num: this button number, count: number of buttons
    pub fn show(&mut self, num: usize, count: usize, do_transform: bool) {
        if self.visible { return }
        self.shapes.items.clear();

        let time = self.time();
        self.visible = true;
        self.last_num = num;
        self.last_count = count;
        self.hide_time = -1.0;


        let radius = (self.window_size.y / 6.0) + X_OFFSET;
        let center = self.window_size / 2.0;


        let height = self.size.y;
        let angle = (PI / (count + 2 * ITEM_PADDING - 1) as f32) * (num + ITEM_PADDING) as f32 - PI / 2.0;
        let end = center + Vector2::new(
            angle.cos() * radius,
            angle.sin() * radius,
        ) - Vector2::new(0.0, height / 2.0);

        let duration = if do_transform { 500.0 } else { 1.0 };
        let start = Vector2::new(center.x, end.y);

        let t1 = Transformation::new(
            0.0,
            duration,
            TransformType::Position {start, end},
            Easing::Linear,
            time
        );
        let t2 = Transformation::new(
            0.0,
            duration,
            TransformType::Transparency { start: 0.0, end: 1.0 },
            Easing::Linear,
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
        self.visible = false;
        self.last_num = num;
        self.last_count = count;
        self.hide_time = time;


        let radius = (self.window_size.y / 6.0) * VISUALIZATION_SIZE_FACTOR + X_OFFSET;
        let center = self.window_size / 2.0;


        let height = self.size.y;
        let angle = (PI / (count + 2 * ITEM_PADDING - 1) as f32) * (num + ITEM_PADDING) as f32 - PI / 2.0;
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
            Easing::Linear,
            time
        );
        

        let t2 = Transformation::new(
            0.0,
            duration,
            TransformType::Transparency { start: 1.0, end: 0.0 },
            Easing::Linear,
            time
        );

        self.shapes.transforms.push(t1);
        self.shapes.transforms.push(t2);

        for i in self.disposable_shapes.iter_mut() {
            i.transforms.push(t1);
            i.transforms.push(t2);
        }

    }
    

    pub fn time(&self) -> f32 {
        self.timer.as_millis()
    }

    pub fn window_size_changed(&mut self, window_size: &Arc<WindowSize>) {
        self.window_size = window_size.0;
        let scale = self.window_size.y / 1080.0;

        if let Some(i) = &mut self.image {
            i.scale = Vector2::ONE * scale;
            self.size = i.size();
        }
        if let Some(i) = &mut self.hover_image {
            i.scale = Vector2::ONE * scale;
            self.size = i.size();
        }

        if self.visible {
            self.visible = false;
            self.show(self.last_num, self.last_count, false);
        }
    }
}
impl ScrollableItemGettersSetters for MainMenuButton {
    fn size(&self) -> Vector2 { self.size * self.window_size.y / 1080.0 }
    fn get_pos(&self) -> Vector2 {
        *self.shapes.pos
    }

    fn get_hover(&self) -> bool {self.hover}
    fn set_hover(&mut self, mut hover:bool) {
        if !self.visible { hover = false }
        if self.hover == hover { return }
        self.hover = hover;

        if hover {
            if let Some(hover) = self.hover_image.clone() {
                self.shapes.items.clear();
                self.shapes.push(hover);
            } else {
                // self.shapes.push(Rectangle::new(
                //     Color::TRANSPARENT_WHITE,
                //     10.0,
                //     self.pos,
                //     self.size,
                //     Some(Border::new(Color::RED, 0.0))
                // ).shape(Shape::Round(5.0, 10)))
                self.shapes.transforms.push(Transformation::new(
                    0.0,
                    1.0, 
                    TransformType::BorderTransparency { start: 0.0, end: 1.0 },
                    Easing::Linear,
                    0.0,
                ))
            }

        } else {
            if let Some(image) = self.image.clone() {
                self.shapes.items.clear();
                self.shapes.push(image);
            } else {
                self.shapes.transforms.push(Transformation::new(
                    0.0,
                    1.0, 
                    TransformType::BorderTransparency { start: 1.0, end: 0.0 },
                    Easing::Linear,
                    0.0,
                ))
            }

            // self.shapes.items.remove(self.shapes.items.len() - 1);
        }

        // let transform = Transformation::new(
        //     0.0, 
        //     1.0,
        //     TransformType::BorderSize {start: size, end: size},
        //     TransformEasing::Linear,
        //     self.time()
        // );

        // self.shapes.transforms.push(transform);
    }
    fn get_selected(&self) -> bool {self.selected}
    fn set_selected(&mut self, mut selected:bool) {
        if !self.visible {selected = false}
        if self.selected == selected { return }
        self.selected = selected;
        trace!("setting selected: {}", selected);

        self.set_hover(self.selected);
    }
    fn get_selectable(&self) -> bool {false}
}
impl ScrollableItem for MainMenuButton {
    fn update(&mut self) {
        if self.shapes.items.len() == 0 {
            let scale = self.window_size.y / 1080.0;

            if let Some(image) = self.image.clone() {
                if self.visible {
                    self.shapes.push(image);
                }
            } else {
                // draw box
                let r = Rectangle::new(
                    self.pos,
                    self.size * scale,
                    Color::new(0.2, 0.2, 0.2, 1.0),
                    Some(Border::new(Color::RED.alpha(0.0), 1.0))
                ).shape(Shape::Round(5.0, 10));
                
                // draw text
                let mut txt = Text::new(
                    Vector2::ZERO,
                    15.0 * scale,
                    self.text.to_owned(),
                    Color::WHITE,
                    get_font()
                );
                txt.center_text(&r);

                self.shapes.push(r);
                self.shapes.push(txt);
            }
        }

        let time = self.timer.as_millis();
        self.shapes.update(time);

        self.disposable_shapes.retain_mut(|i| {
            i.update(time);
            i.visible()
        });


        if self.hide_time > 0.0 {
            if time - self.hide_time > 500.0 {
                self.visible = false;
            }
        }

    }

    fn draw(&mut self, _pos_offset:Vector2, list: &mut RenderableCollection) {
        if !self.visible { return }
        // self.shapes.draw(list);
        list.push(self.shapes.clone());

        for i in self.disposable_shapes.iter() {
            // i.draw(list);
            list.push(i.clone())
        }
    }
}


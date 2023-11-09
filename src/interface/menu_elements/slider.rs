use crate::prelude::*;

// i think this is going to require me to implement mouse up events. rip
const SNAP_LENIENCY:f64 = 0.5;
const TRACKBAR_WIDTH:f32 = 1.0;

#[derive(Clone, ScrollableGettersSetters)]
pub struct Slider {
    pos: Vector2,
    size: Vector2,
    style: Style,
    node: Node,

    hover: bool,
    selected: bool,
    tag: String,

    mouse_down: bool, // store the mouse state
    text: String,
    keywords: Vec<String>,

    pub value: f64,
    pub range: Range<f64>,
    pub snapping: Option<f64>, // snap every multiple of this

    pub font: Font,
    pub font_size: f32,
    
    pub on_change: Arc<dyn Fn(&mut Self, f64) + Send + Sync>,
}
impl Slider {
    pub fn new(style: Style, text:&str, value:f64, range:Option<Range<f64>>, snapping:Option<f64>, layout_manager: &LayoutManager, font:Font) -> Self {
        let range = if let Some(r) = range{r} else {0.0..100.0};

        let (pos, size) = LayoutManager::get_pos_size(&style);
        let node = layout_manager.create_node(&style);

        Self {
            pos, 
            size, 
            style, 
            node,

            hover: false,
            selected: false,
            tag: String::new(),
            keywords: text.split(" ").map(|a|a.to_lowercase().to_owned()).collect(),

            text: text.to_owned(),
            value,
            range,
            snapping,

            mouse_down:false,
            font,

            font_size: 12.0,
            
            on_change: Arc::new(|_,_|{}),
        }
    }

    fn set_value_by_mouse(&mut self, mouse_pos: Vector2) {
        // extrapolate value
        let bounds = self.get_slider_bounds();
        let rel_x = mouse_pos.x - (self.pos.x + bounds.pos.x); // mouse pos - (self offset + text pos offset) (ie, mouse- slider bar start)
        let mut val = self.range.start + (rel_x / bounds.size.x) as f64 * (self.range.end - self.range.start);

        // solve for rel_x
        // val = min + (rel_x / bounds.x) * (max - min)
        // (val - min) = (rel_x / bounds.x) * (max - min)
        // (val - min) / (max - min) = rel_x / bounds.x
        // (val - min) / (max - min) * bounds.x = rel_x


        // check snapping (probably needs work lol)
        if let Some(snap) = self.snapping {
            if (val % snap).abs() < SNAP_LENIENCY {
                //TODO: find out if the snap is "up" or "down"
                println!("snapping");
                val -= val % snap;
            }
        }

        // make sure its within bounds
        val = val.clamp(self.range.start, self.range.end);
        // println!("val:{}, min:{}, max:{}", val, self.range.start, self.range.end);

        self.value = val;
        (self.on_change.clone())(self, val);
    }

    fn text_val(&self) -> String {format!("{}: {:.2}", self.text, self.value)}
    pub fn get_slider_bounds(&self) -> Bounds {
        // draw text
        let txt = Text::new(
            Vector2::ZERO,
            self.font_size.clone(),
            format!("{}: {:.2}", self.text, self.range.end),
            Color::WHITE,
            self.font.clone()
        );
        let text_size = txt.measure_text() + Vector2::new(10.0, 0.0);

        Bounds::new(
            Vector2::new(text_size.x, 0.0),
            Vector2::new(self.size.x - text_size.x, self.size.y)
        )
    }
}

impl ScrollableItem for Slider {
    fn get_style(&self) -> Style { self.style.clone() }
    fn apply_layout(&mut self, layout: &LayoutManager, parent_pos: Vector2) {
        let layout = layout.get_layout(self.node);
        self.pos = layout.location.into();
        self.pos += parent_pos;
        self.size = layout.size.into();
    }

    fn get_value(&self) -> Box<dyn std::any::Any> {Box::new(self.value)}
    fn get_keywords(&self) -> Vec<String> { self.keywords.clone() }


    fn draw(&mut self, pos_offset:Vector2, list:&mut RenderableCollection) {
        
        // draw bounding box
        list.push(Rectangle::new(
            self.pos+pos_offset,
            self.size,
            Color::new(0.0, 0.0, 0.0, 0.2),
            if self.hover {Some(Border::new(Color::RED, 1.0))} else if self.selected {Some(Border::new(Color::BLUE, 1.0))} else {None}
        ));

        // draw text
        let slider_bounds = self.get_slider_bounds();

        let mut txt = Text::new(
            Vector2::new(0.0, 20.0),
            self.font_size,
            self.text_val(),
            Color::WHITE,
            self.font.clone()
        );
        txt.center_text(&Bounds::new(self.pos+pos_offset, Vector2::new(self.size.x - slider_bounds.size.x, self.size.y)));
        // TODO: center text to area that slider_bounds isnt ([text][slider_bounds])
        // let text_size = txt.measure_text();
        list.push(txt);
        
        // draw track
        list.push(Rectangle::new(
            self.pos + pos_offset + slider_bounds.pos + Vector2::with_y(slider_bounds.size.y / 3.0),
            Vector2::new(slider_bounds.size.x, slider_bounds.size.y / 3.0),
            Color::BLACK,
            None
        ));

        // draw snap lines (definitely doesnt work yet)
        if let Some(snap) = self.snapping {
            for i in 0..slider_bounds.size.x.floor() as i32 {

                list.push(Rectangle::new(
                    (self.pos + pos_offset + Vector2::new(slider_bounds.pos.x - TRACKBAR_WIDTH/2.0, 0.0))
                        // actual value offset
                        + Vector2::new(i as f32 * snap as f32, 0.0),
                    Vector2::new(TRACKBAR_WIDTH, self.size.y / 1.3),
                    Color::RED,
                    None
                ));
            }
        }

        // draw value circle
        list.push(Circle::new(
            // bounds offset
        (self.pos + pos_offset + Vector2::new(slider_bounds.pos.x - TRACKBAR_WIDTH/2.0, 0.0))
            // actual value offset
            + Vector2::new(((self.value - self.range.start) / (self.range.end - self.range.start)) as f32 * slider_bounds.size.x, slider_bounds.size.y / 2.0),
        self.size.y / 3.0,
            Color::BLUE,
            None
        ));
    }


    fn on_mouse_move(&mut self, p:Vector2) {
        self.check_hover(p);

        if self.mouse_down {
            self.set_value_by_mouse(p)
        }
    }

    fn on_click_tagged(&mut self, pos:Vector2, _button:MouseButton, _mods:KeyModifiers) -> Option<String> {
        self.check_hover(pos);
        
        if self.hover {
            self.set_value_by_mouse(pos);
            self.mouse_down = true;

            Some(self.tag.clone())
        } else {
            None
        }
    }

    //TODO: on key press (left/right)
    fn on_click_release(&mut self, _pos:Vector2, _button:MouseButton) {
        self.mouse_down = false;
    }
}
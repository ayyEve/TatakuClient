use crate::prelude::*;

const TRAIL_CREATE_TIMER:f64 = 10.0;
const TRAIL_FADEOUT_TIMER_START:f64 = 20.0;
const TRAIL_FADEOUT_TIMER_DURATION:f64 = 100.0;

const TRAIL_CREATE_TIMER_IF_MIDDLE:f64 = 0.1;
const TRAIL_FADEOUT_TIMER_START_IF_MIDDLE:f64 = 20.0;
const TRAIL_FADEOUT_TIMER_DURATION_IF_MIDDLE:f64 = 500.0;

pub struct CursorManager {
    /// position of the visible cursor
    pub pos: Vector2,

    /// should the cursor be visible?
    pub visible: bool,

    pub color: Color,
    pub border_color: Color,

    /// should the mouse not follow the user's cursor.
    /// actually used inside game, not here
    pub replay_mode: bool,

    /// did the replay mode value change?
    /// needed so we know whether to show/hide the window cursor
    pub replay_mode_changed: bool,

    pub left_pressed: bool,
    pub right_pressed: bool,

    pub cursor_image: Option<Image>,
    pub cursor_trail_image: Option<Image>,
    pub trail_images: Vec<TransformGroup>,
    last_trail_time: f64,


    trail_create_timer: f64,
    trail_fadeout_timer_start: f64,
    trail_fadeout_timer_duration: f64,
}

impl CursorManager {
    pub fn new() -> Self {
        let mut cursor_image = SKIN_MANAGER.write().get_texture("cursor", true);
        if let Some(cursor) = &mut cursor_image {
            cursor.depth = -f64::MAX;
        }

        let mut cursor_trail_image = SKIN_MANAGER.write().get_texture("cursortrail", true);
        if let Some(cursor) = &mut cursor_trail_image {
            cursor.depth = (-f64::MAX) + 0.01;
        }

        let settings = get_settings!();


        let has_middle = SKIN_MANAGER.write().get_texture("cursormiddle", false).is_some();
        let (trail_create_timer, trail_fadeout_timer_start, trail_fadeout_timer_duration) = if has_middle {
            (TRAIL_CREATE_TIMER_IF_MIDDLE, TRAIL_FADEOUT_TIMER_START_IF_MIDDLE, TRAIL_FADEOUT_TIMER_DURATION_IF_MIDDLE)
        } else {
            (TRAIL_CREATE_TIMER, TRAIL_FADEOUT_TIMER_START, TRAIL_FADEOUT_TIMER_DURATION)
        };

        Self {
            pos: Vector2::zero(),
            visible: true,
            replay_mode: false,
            replay_mode_changed: false,
            color: Color::from_hex(&settings.cursor_color),
            border_color: Color::from_hex(&settings.cursor_border_color),

            left_pressed: false,
            right_pressed: false,

            trail_images: Vec::new(),
            cursor_image,
            cursor_trail_image,
            last_trail_time: 0.0,

            trail_create_timer, 
            trail_fadeout_timer_start,
            trail_fadeout_timer_duration,
        }
    }


    pub fn reload_skin(&mut self) {
        // TODO: this
        self.cursor_image = SKIN_MANAGER.write().get_texture("cursor", true);
        self.cursor_trail_image = SKIN_MANAGER.write().get_texture("cursortrail", true);
    }

    /// set replay mode.
    /// really just a helper
    #[allow(unused)]
    pub fn set_replay_mode(&mut self, val:bool) {
        if val != self.replay_mode {
            self.replay_mode = val;
            self.replay_mode_changed = true;
        }
    }

    pub fn set_cursor_pos(&mut self, pos:Vector2) {
        self.pos = pos;
    }

    pub fn update(&mut self, game_time: f64) {
        // trail stuff

        // check if we should add a new trail
        if self.cursor_trail_image.is_some() && game_time - self.last_trail_time >= self.trail_create_timer {
            if let Some(mut trail) = self.cursor_trail_image.clone() {
                let mut g = TransformGroup::new();
                g.transforms.push(Transformation::new(
                    self.trail_fadeout_timer_start, 
                    self.trail_fadeout_timer_duration, 
                    TransformType::Transparency {start: 1.0, end: 0.0}, 
                    TransformEasing::EaseOutSine, 
                    game_time
                ));
                trail.current_pos = self.pos;
                trail.depth = (-f64::MAX) + 0.01;
                g.items.push(DrawItem::Image(trail));

                self.trail_images.push(g);
                self.last_trail_time = game_time;
            }

        }
    
        // update the transforms, removing any that are not visible
        self.trail_images.retain_mut(|i| {
            i.update(game_time);
            i.items[0].visible()
        });
    }

    pub fn draw(&mut self, list:&mut Vec<Box<dyn Renderable>>) {
        if !self.visible {return}

        let mut radius = 5.0;
        if self.left_pressed || self.right_pressed {
            radius *= 2.0;
        }

        let settings = get_settings!();

        if self.cursor_trail_image.is_some() {
            // draw the transforms
            for i in self.trail_images.iter_mut() {
                i.draw(list);
            }
        }
        
        if let Some(cursor) = &mut self.cursor_image {
            cursor.current_pos = self.pos;
            cursor.current_color = self.color;
            list.push(Box::new(cursor.clone()));
        } else {
            list.push(Box::new(Circle::new(
                self.color,
                -f64::MAX,
                self.pos,
                radius * settings.cursor_scale,
                if settings.cursor_border > 0.0 {
                    Some(Border::new(
                        self.border_color,
                        settings.cursor_border as f64
                    ))
                } else {None}
            )));
        }

    }
}
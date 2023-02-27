#![allow(unused)]
use crate::prelude::*;
const DEBUG: bool = false;

#[derive(Clone, Debug)]
pub struct StoryboardSpriteDef {
    /// the layer the object appears on
    pub layer: Layer,

    /**
     * Where on the image should osu! consider that image's origin (coordinate) to be. 
     * This affects the (x) and (y) values, as well as several other command-specific behaviors. 
     * For example, choosing (origin) = TopLeft will let the (x),(y) values determine, where the top left corner of the image itself should be on the screen.
     */
    pub origin: Origin,

    
    /// https://osu.ppy.sh/wiki/en/Storyboard/Scripting/Objects
    pub filepath: String,

    /**
     * (x) and (y) are the x-/y-coordinates of where the object should be, by default respectively. The interpretation of this depends on the value of (origin); 
     * for instance, to place a 640x480 image as your background, the values could be: 
     * origin = TopLeft, x = 0, y = 0 
     * origin = Centre, x = 320, y = 240 
     * origin = BottomRight, x = 640, y = 480 
     */
    pub pos: Vector2
}

#[derive(Clone, Debug)]
pub struct StoryboardAnimationDef {
    /// the layer the object appears on
    pub layer: Layer,

    /**
     * Where on the image should osu! consider that image's origin (coordinate) to be. 
     * This affects the (x) and (y) values, as well as several other command-specific behaviors. 
     * For example, choosing (origin) = TopLeft will let the (x),(y) values determine, where the top left corner of the image itself should be on the screen.
     */
    pub origin: Origin,

    /// https://osu.ppy.sh/wiki/en/Storyboard/Scripting/Objects
    pub filepath: String,

    /**
     * (x) and (y) are the x-/y-coordinates of where the object should be, by default respectively. The interpretation of this depends on the value of (origin); 
     * for instance, to place a 640x480 image as your background, the values could be: 
     * origin = TopLeft, x = 0, y = 0 
     * origin = Centre, x = 320, y = 240 
     * origin = BottomRight, x = 640, y = 480 
     */
    pub pos: Vector2,

    /// indicates how many frames the animation has. If we have "sample0.png" and "sample1.png"
    pub frame_count: u16,

    /// indicates how many milliseconds should be in between each frame. For instance, if we wanted our animation to advance at 2 frames per second, frameDelay = 500.
    pub frame_delay: f32,

    /// indicates if the animation should loop or not.
    pub loop_type: LoopType,
}

#[derive(Clone, Debug)]
pub enum StoryboardElementDef {
    Sprite(StoryboardSpriteDef),
    Animation(StoryboardAnimationDef)
}
impl StoryboardElementDef {
    pub fn read(line: &String) -> Option<Self> {
        if line.starts_with('_') || line.starts_with(' ') { return None; }

        let mut split = line.split(",");
        let Some(ele) = split.next() else { return None };
        let Some(layer) = split.next().and_then(|i|Layer::from_str(i)) else { return None };
        let Some(origin) = split.next().and_then(|i|Origin::from_str(i)) else { return None };
        let Some(filepath) = split.next() else { return None };
        let Some(x) = split.next().and_then(|s|s.parse::<i32>().ok()) else { return None };
        let Some(y) = split.next().and_then(|s|s.parse::<i32>().ok()) else { return None };
        let pos = Vector2::new(x as f64, y as f64);
        let filepath = filepath.trim_matches('"').to_owned();

        match ele {
            "Sprite" => Some(StoryboardElementDef::Sprite(StoryboardSpriteDef { layer, origin, filepath, pos })),
            "Animation" => {
                let Some(frame_count) = split.next().and_then(|s|s.parse::<u16>().ok()) else { return None };
                let Some(frame_delay) = split.next().and_then(|s|s.parse::<f32>().ok()) else { return None };
                let loop_type = split.next().and_then(|i|LoopType::from_str(i)).unwrap_or(LoopType::LoopForever);

                Some(StoryboardElementDef::Animation(StoryboardAnimationDef { layer, origin, filepath, pos, frame_count, frame_delay, loop_type }))
            }

            _=> None
        }
    }
}


#[derive(Copy, Clone, Debug)]
pub enum Origin {
    TopLeft = 0,
    Centre = 1,
    CentreLeft = 2,
    TopRight = 3,
    BottomCentre = 4,
    TopCentre = 5,
    Custom = 6, // (same effect as TopLeft, but should not be used)
    CentreRight = 7,
    BottomLeft = 8,
    BottomRight = 9
}
impl Origin {
    pub fn from_str(str: &str) -> Option<Self> {
        match str {
            "0" | "TopLeft" => Some(Self::TopLeft),
            "1" | "Centre" => Some(Self::Centre),
            "2" | "CentreLeft" => Some(Self::CentreLeft),
            "3" | "TopRight" => Some(Self::TopRight),
            "4" | "BottomCentre" => Some(Self::BottomCentre),
            "5" | "TopCentre" => Some(Self::TopCentre),
            "6" | "Custom" => Some(Self::Custom),
            "7" | "CentreRight" => Some(Self::CentreRight),
            "8" | "BottomLeft" => Some(Self::BottomLeft),
            "9" | "BottomRight" => Some(Self::BottomRight),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum LoopType {
    LoopForever = 0,
    LoopOnce = 1
}
impl LoopType {
    pub fn from_str(str: &str) -> Option<Self> {
        match str {
            "0" | "LoopForever" => Some(Self::LoopForever),
            "1" | "LoopOnce" => Some(Self::LoopOnce),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum Layer {
    Background = 0,
    Fail = 1,
    Pass = 2,
    Foreground = 3
}
impl Layer {
    pub fn from_str(str: &str) -> Option<Self> {
        match str {
            "0" | "Background" => Some(Self::Background),
            "1" | "Fail" => Some(Self::Fail),
            "2" | "Pass" => Some(Self::Pass),
            "3" | "Foreground" => Some(Self::Foreground),
            _ => None,
        }
    }
}



#[derive(Clone, Debug)]
pub struct StoryboardEntryDef {
    pub element: StoryboardElementDef,
    pub commands: Vec<StoryboardCommand>
}

#[derive(Clone, Debug)]
pub struct StoryboardDef {
    pub entries: Vec<StoryboardEntryDef>
}
impl StoryboardDef {
    pub fn read(lines: Vec<String>) -> TatakuResult<Self> {
        let mut entries = Vec::new();
        let mut current_entry = None;

        struct TempLoopDef {
            // used to determine when we've left the loop event list
            depth_index: usize,
            start_time: f32,
            loops: u32,
            events: Vec<StoryboardCommand>
        }
        impl TempLoopDef {
            fn apply(&self, entry: &mut StoryboardEntryDef) {
                if DEBUG { println!("applying loop {}", self.loops); }

                // loop events 
                let mut time = self.start_time;
                let mut end_time = self.events.iter().fold(0f32, |v, cmd|v.max(cmd.end_time));
                for i in 0..self.loops {
                    // next iteration
                    time += end_time;

                    for mut event in self.events.clone() {
                        if DEBUG { println!("[loop] adding event {event:?}"); }
                        event.start_time += time;
                        event.end_time += time;
                        entry.commands.push(event)
                    }
                }
            }
        }


        // are we reading a loop event list? if so, what are the params
        let mut loop_def:Option<TempLoopDef> = None;

        let mut trigger_def: Option<usize> = None;

        for line in lines {
            if line.len() == 0 || line.starts_with("//") { continue }

            // check if there's a new element
            if let Some(new_ele) = StoryboardElementDef::read(&line) {
                
                // deal with old ele if exists
                if let Some(mut old_ele) = std::mem::take(&mut current_entry) {
                    if let Some(loop_def) = std::mem::take(&mut loop_def) {
                        loop_def.apply(&mut old_ele)
                    }

                    entries.push(old_ele);
                }
                
                current_entry = Some(StoryboardEntryDef {
                    element: new_ele,
                    commands: Vec::new()
                });

                if DEBUG { println!("\nnew entry: {current_entry:?}"); }

                continue;
            }


            let old_len = line.len();
            let line = line.trim_start_matches([' ', '_']);
            let current_depth = old_len - line.len();

            if let Some(trigger_depth) = trigger_def {
                if trigger_depth != current_depth {
                    trigger_def = None;
                } else {
                    // skip triggers for now
                    continue;
                }
            }


            // if the loop has finished
            if loop_def.as_ref().filter(|l|l.depth_index != current_depth).is_some() {
                let loop_def = std::mem::take(&mut loop_def).unwrap();
                if let Some(entry) = &mut current_entry {
                    loop_def.apply(entry);
                }
            }

            let Some(current_entry) = &mut current_entry else { continue };

            let mut split = line.split(",");

            // helper because this code was already ugly
            macro_rules! parse_or_continue {
                ($name: ident, $T:ty) => {
                    let Some($name) = split.next().and_then(|s|s.parse::<$T>().ok()) else { println!("error reading {}", stringify!($name)); continue };
                };
                ($name: ident, $T:ty, $default: ident) => {
                    let $name = split.next().and_then(|s|s.parse::<$T>().ok()).unwrap_or($default);
                };
                ($name: ident, $T:ty, _) => {
                    let Some($name) = split.next().and_then(|s|<$T>::from_str(s)) else { println!("error reading {}", stringify!($name)); continue };
                };
            }

            let Some(event) = split.next() else { continue };

            // check for loops because they're a special case
            if event == "L" {
                parse_or_continue!(start_time, f32); // let Some(start_time) = split.next().and_then(|s|s.parse::<f32>().ok()) else { continue };
                parse_or_continue!(loops, u32); // let Some(loops) = split.next().and_then(|s|s.parse::<u32>().ok()) else { continue };
                if DEBUG { println!("starting loop_def at depth {}", current_depth + 1); }

                loop_def = Some(TempLoopDef {
                    depth_index: current_depth + 1,
                    start_time,
                    loops,
                    events: Vec::new(),
                });

                continue;
            }
            
            if event == "T" {
                trigger_def = Some(current_depth + 1);
                continue
            }

            // check for other events
            parse_or_continue!(easing, StoryboardEasing, _); //split.next().and_then(|s|s.parse::<i32>().ok()) else { continue };
            parse_or_continue!(start_time, f32); // let Some(start_time) = split.next().and_then(|s|s.parse::<f32>().ok()) else { continue };
            parse_or_continue!(end_time, f32, start_time); // let Some(end_time) = split.next().and_then(|s|s.parse::<f32>().ok()) else { continue };
            let event = match event {
                "F" => {
                    parse_or_continue!(start, f32);
                    parse_or_continue!(end, f32, start);
                    // if start == end {
                    //     end += 1.0;
                    // }
                    StoryboardEvent::Fade { start, end }
                }
                "M" => {
                    parse_or_continue!(start_x, f64);
                    parse_or_continue!(start_y, f64);
                    parse_or_continue!(end_x, f64, start_x);
                    parse_or_continue!(end_y, f64, start_y);
                    let start = Vector2::new(start_x, start_y);
                    let end = Vector2::new(end_x, end_y);
                    StoryboardEvent::Move { start, end }
                }
                "MX" => {
                    parse_or_continue!(start_x, f32);
                    parse_or_continue!(end_x, f32, start_x);
                    StoryboardEvent::MoveX { start_x, end_x }
                }
                "MY" => {
                    parse_or_continue!(start_y, f32);
                    parse_or_continue!(end_y, f32, start_y);
                    StoryboardEvent::MoveY { start_y, end_y }
                }

                "S" => {
                    parse_or_continue!(start_scale, f32);
                    parse_or_continue!(end_scale, f32, start_scale);
                    StoryboardEvent::Scale { start_scale, end_scale }
                }

                "V" => {
                    parse_or_continue!(start_scale_x, f64);
                    parse_or_continue!(start_scale_y, f64);
                    parse_or_continue!(end_scale_x, f64, start_scale_x);
                    parse_or_continue!(end_scale_y, f64, start_scale_y);
                    let start_scale = Vector2::new(start_scale_x, start_scale_y);
                    let end_scale = Vector2::new(end_scale_x, end_scale_y);
                    StoryboardEvent::VectorScale { start_scale, end_scale}
                }
                
                "R" => {
                    parse_or_continue!(start_rotation, f32);
                    parse_or_continue!(end_rotation, f32, start_rotation);
                    StoryboardEvent::Rotate { start_rotation, end_rotation }
                }

                "C" => {
                    continue;
                    // parse_or_continue!(start_r, u8);
                    // parse_or_continue!(start_g, u8);
                    // parse_or_continue!(start_b, u8);
                    // parse_or_continue!(end_r, u8, start_r);
                    // parse_or_continue!(end_g, u8, start_g);
                    // parse_or_continue!(end_b, u8, start_b);
                    // let start_color = color_from_byte(start_r, start_g, start_b);
                    // let end_color = color_from_byte(end_r, end_g, end_b);
                    // StoryboardEvent::Color { start_color, end_color }
                }

                "P" => {
                    parse_or_continue!(param, Param, _);
                    StoryboardEvent::Parameter { param }
                }

                // loops are checked earlier

                // triggers will be checked earlier once i have the willpower to add them

                other => { println!("unknown storyboard event {other}"); continue},
            };

            let cmd = StoryboardCommand {
                event,
                easing,
                start_time,
                end_time,
            };

            if let Some(loop_def) = &mut loop_def {
                loop_def.events.push(cmd);
            } else {
                if DEBUG { println!("adding event {cmd:?}"); }
                current_entry.commands.push(cmd);
            }
        }

        Ok(Self { entries })
    }
}

#[test]
fn test() {
    // let path = "E:/Program Files/osu!/Songs/209841 Otokaze - KaeriMichi/Otokaze - KaeriMichi (Narcissu).osb";
    // let path = "E:/Program Files/osu!/Songs/883505 nanobii - HYPERDRIVE/nanobii - HYPERDRIVE (hypercyte).osb";
    let path = "E:/Program Files/osu!/Songs/151720 ginkiha - EOS/ginkiha - EOS (alacat).osb";

    let lines = Io::read_lines_resolved(path).unwrap().collect::<Vec<String>>();

    let storyboard = StoryboardDef::read(lines).unwrap();

    // println!("{storyboard:#?}");
}
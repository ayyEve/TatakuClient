use crate::prelude::*;


fn get_note_type(c: char) -> NoteType {
    todo!();
    // match c {

    // }
}

#[derive(Default)]
pub struct TJABeatmap {
    title: String,
    /// generally artist
    subtitle: String,
    bpm: f32,
    wave: String,
    offset: f32,
    demostart: f32,

    branches: Vec<BranchObject>,
    measures: Vec<Measure>,
    circles: Vec<Circle>
}
impl TJABeatmap {
    pub fn parse(path: String) -> TatakuResult<Self> {
        if let Ok(file) = String::from_utf8(std::fs::read(path)?) {
            let lines = file.split("\n").map(|l|l.trim());

            let mut s = Self::default();

            let mut in_song = false;
            let mut courses:HashMap<String, Course> = HashMap::new();

            let mut current_course_name = "oni".to_owned();

            // so many vars
            let mut bpm = 120.0;
            let mut gogo = false; // kiai
            let mut scroll = 1.0; // scroll speed
            let mut measure = 4.0;
            let mut ms = 0f32; // current time in the current map
            let mut bar_lines = true;
            let mut section_begin = true;

            let mut last_bpm = bpm;
            let mut last_gogo = gogo;

            let mut branch = false;
            let mut current_branch = None;
            let mut branch_first_measure = false;
            let mut branch_settings = BranchSettings::default();
            let mut branch_obj = BranchObject::default();

            let mut lyrics_line = String::new();

            let mut last_drumroll:Option<EndDrumroll> = None;
            let mut balloon_id = 0;

            let mut current_measure:Vec<Measure> = Vec::new();

            macro_rules! insert_empty_note {
                () => {

                    todo!()
                }
            }

            macro_rules! insert_note {
                ($c: ident) => {

                    todo!()
                }
            }

            macro_rules! push_measure {
                () => {

                    todo!()
                }
            }



            for (_i, line) in lines.enumerate() {
                // trim out comments
                let line = line.split("//").next().unwrap_or("").trim();
                if line.is_empty() {continue}

                if line.starts_with("#") { // meta
                    let mut split = line.split(" ");
                    let property = split.next().unwrap();
                    let value = split.next().unwrap_or("");

                    match &*property {
                        "#start" if !in_song => {
                            in_song = true;
                        }
                        "#end" if in_song => {
                            in_song = false;
                        }
                        "#lyric" => {
                            if let Some(course) = courses.get_mut(&current_course_name) {
                                course.branch = true;
                            }
                            lyrics_line = value.replace("\\n", "\n").to_owned();
                        }

                        // note things
                        "#gogostart" => gogo = true,
                        "#gogoend" => gogo = false,
                        "#bpmchange" => bpm = value.parse().unwrap_or(bpm),
                        "#scroll" => scroll = value.parse().unwrap_or(scroll),
                        "#measure" => {
                            let mut split2 = value.split("/");
                            let (numerator, denominator):(i32,i32) = match split2.next().zip(split2.next()).and_then(|f|f.0.parse().ok().zip(f.1.parse().ok())) {
                                Some(s) => s,
                                None => continue
                            };
                            measure = (numerator as f32 / denominator as f32) * 4.0;
                        }

                        "#delay" => ms += value.parse::<f32>().unwrap_or_default() * 1000.0,
                        "#barlineon" => bar_lines = true,
                        "#barlineoff" => bar_lines = false,

                        "#branchstart" => {
                            if let Some(course) = courses.get_mut(&current_course_name) {
                                course.branch = true;
                            }

                            branch = true;
                            current_branch = None;
                            branch_first_measure = true;

                            branch_settings = BranchSettings {
                                ms,
                                gogo,
                                bpm,
                                scroll,
                                section_begin
                            };

                            let mut val_split = value.split(",");

                            let requirement = BranchRequirement {
                                advanced: val_split.next().unwrap_or("0").parse().unwrap_or_default(),
                                master: val_split.next().unwrap_or("0").parse().unwrap_or_default(),
                            };

                            let requirement_active = match (requirement.advanced > 0.0, requirement.master > 0.0) {
                                (true, true) => RequirementActive::Normal,
                                (false, true) => RequirementActive::Advanced,
                                _ => RequirementActive::Master,
                            };

                            branch_obj = BranchObject {
                                ms,
                                original_ms: ms,
                                requirement_active,
                                requirement_type: if value.chars().next().unwrap_or('a') == 'r' {RequirementType::Drumroll} else {RequirementType::Accuracy},
                                requirement,
                                branches: HashMap::new()
                            };
                            s.branches.push(branch_obj.clone());

                            if s.measures.len() == 1 && branch_obj.requirement_type == RequirementType::Drumroll {
                                for circle in s.circles.iter() {
                                    if let (Some(end_time), NoteType::Balloon|NoteType::Drumroll|NoteType::DaiDrumroll) = (circle.end_time, circle.note_type) {
                                        s.measures.push(Measure { 
                                            ms: end_time, 
                                            original_ms: end_time, 
                                            speed: circle.bpm * circle.scroll / 60.0, 
                                            visible: false, 
                                            branch: circle.branch.clone(),
                                            next_branch: None
                                        });
                                        break
                                    }
                                }
                            }
                            if s.measures.len() > 0 {
                                s.measures.iter_mut().last().unwrap().next_branch = Some(branch_obj.clone());
                            }
                        }

                        "#branchend" => {
                            branch = false;
                            current_branch = None;
                        }

                        "#section" => {
                            section_begin = true;
                            if branch && current_branch.is_none() {
                                branch_settings.section_begin = true;
                            }
                        }

                        "#n" | "#e" | "#m" if branch => {
                            ms = branch_settings.ms;
                            gogo = branch_settings.gogo;
                            bpm = branch_settings.bpm;
                            scroll = branch_settings.scroll;
                            section_begin = branch_settings.section_begin;
                            branch_first_measure = true;

                            let branch_name = match &*property {
                                "#n" => "normal",
                                "#e" => "advanced",
                                "#m" => "master",
                                _ => ""
                            };

                            current_branch = Some(Branch {
                                name: branch_name.to_owned(),
                                active: branch_name == branch_obj.requirement_active.as_str()
                            });
                            branch_obj.branches.insert(branch_name.to_owned(), current_branch.clone().unwrap());
                        }

                        _ => {}
                    }
                } else if line.contains(":") { // properties
                    let mut split = line.split(":");
                    let property = split.next().unwrap();
                    let value = split.next().unwrap_or("");
                    if value.is_empty() {continue};

                    match &*property.to_lowercase() {
                        "title" => s.title = value.to_owned(),
                        "subtitle" => s.subtitle = value.to_owned(),
                        "bpm" => {
                            s.bpm = value.parse().unwrap_or_default();
                            bpm = s.bpm;
                        },
                        "wave" => s.wave = value.to_owned(),
                        "offset" => s.offset = value.parse().unwrap_or_default(),
                        "demostart" => s.demostart = value.parse().unwrap_or_default(),

                        // current course properties
                        "course" => {
                            current_course_name = value.to_owned();
                            // new course, insert it
                            courses.insert(current_course_name.clone(), Course::default());
                        },
                        "level" => {
                            if let Some(course) = courses.get_mut(&current_course_name) {
                                course.level = value.parse().unwrap_or_default();
                            }
                        }
                        "balloon" => {
                            if let Some(course) = courses.get_mut(&current_course_name) {
                                course.balloon = value.split(",").map(|f|f.parse().unwrap_or(0)).collect();
                            }
                        }
                        "scoreinit" => {
                            if let Some(course) = courses.get_mut(&current_course_name) {
                                course.score_init = value.parse().unwrap_or_default();
                            }
                        }
                        "scorediff" => {
                            if let Some(course) = courses.get_mut(&current_course_name) {
                                course.score_diff = value.parse().unwrap_or_default();
                            }
                        }

                        _ => {}
                    }
                } else { // notes
                    
                    for symbol in line.to_uppercase().chars() {
                        let mut error = false;

                        match symbol {
                            '0' => insert_empty_note!(),
                            
                            '1'|'2'|'3'|'4'|'A'|'B' => {
                                let t = get_note_type(symbol);
                                let mut circle_obj = Circle {
                                    note_type: t,
                                    gogo, 
                                    bpm,
                                    scroll,
                                    section: section_begin,
                                    ..Circle::default()
                                };
                                section_begin = false;

                                if last_drumroll.is_some() {
                                    circle_obj.end_drumroll = std::mem::take(&mut last_drumroll)
                                }
                                insert_note!(circle_obj);
                            }

                            '5'|'6'|'7'|'9' => {
                                let t = get_note_type(symbol);
                                
                                let mut circle_obj = Circle {
                                    note_type: t,
                                    gogo, 
                                    bpm,
                                    scroll,
                                    section: section_begin,
                                    ..Circle::default()
                                };
                                section_begin = false;

                                if last_drumroll.is_some() {
                                    if symbol == '9' {
                                        let c = Circle {
                                            end_drumroll: std::mem::take(&mut last_drumroll),
                                            gogo, 
                                            bpm,
                                            scroll,
                                            section: section_begin,

                                            ..Circle::default()
                                        };
                                        insert_note!(c);
                                    } else {
                                        insert_empty_note!()
                                    }
                                }

                                if symbol == '7' || symbol == '9' {
                                    let hits = *courses[&current_course_name].balloon.get(balloon_id).unwrap_or(&1);
                                    circle_obj.required_hits = Some(hits);
                                    balloon_id += 1;
                                }
                                last_drumroll = Some(circle_to_end_drumroll(&circle_obj));
                                insert_note!(circle_obj);
                            }
                            '8' => {
                                if last_drumroll.is_some() {
                                    let c = Circle {
                                        end_drumroll: std::mem::take(&mut last_drumroll),
                                        gogo, 
                                        bpm,
                                        scroll,
                                        section: section_begin,

                                        ..Circle::default()
                                    };
                                    insert_note!(c);
                                    section_begin = false;
                                    insert_empty_note!();
                                }
                            }

                            ',' => {
                                if current_measure.len() == 0 && (bpm != last_bpm || gogo != last_gogo || !lyrics_line.is_empty()) {
                                    insert_empty_note!();
                                }
                                push_measure!();
                                current_measure.clear();
                            }

                            'A'..='Z' => {
                                insert_empty_note!()
                            }
                            _ => {
                                // cry
                            }
                        }
                    }


                }
            }

            push_measure!();
            if let Some(ldr) = &mut last_drumroll {
                ldr.end_time = ms;
                ldr.original_end_time = ms;
            }

            Ok(s)
        } else {
            Err(TatakuError::Beatmap(BeatmapError::InvalidFile))
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Course {
    course: String,
    level: u8,

    balloon: Vec<usize>,
    score_init: usize,
    score_diff: usize,

    branch: bool,
    lyric: bool
}

#[derive(Clone, Debug, Default)]
pub struct Branch {
    name: String,
    active: bool,
}

#[derive(Copy, Clone, Default)]
struct BranchSettings {
    ms: f32,
    gogo: bool,
    bpm: f32,
    scroll: f32,
    section_begin: bool
}

#[derive(Clone, Debug, Default)]
pub struct BranchObject {
    ms: f32,
    original_ms: f32,
    // diff name of this branch (?)
    requirement_active: RequirementActive,
    // idk lol
    requirement_type: RequirementType,
    requirement: BranchRequirement,

    branches: HashMap<String, Branch>
}
#[derive(Clone, Debug, Default)]
pub struct BranchRequirement {
    advanced: f32,
    master: f32,
}


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RequirementActive {
    Normal,
    Advanced,
    Master,
}
impl RequirementActive {
    fn as_str(&self) -> &'static str {
        match self {
            RequirementActive::Normal => "normal",
            RequirementActive::Advanced => "advanced",
            RequirementActive::Master => "master",
        }
    }
}
impl Default for RequirementActive {
    fn default() -> Self {Self::Normal}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RequirementType {
    Drumroll,
    Accuracy,
}
impl Default for RequirementType {
    fn default() -> Self {Self::Drumroll}
}

#[derive(Clone, Debug, Default)]
pub struct Measure {
    ms: f32,
    original_ms: f32,
    speed: f32,
    visible: bool,
    branch: Option<BranchObject>,
    next_branch: Option<BranchObject>,
}

#[derive(Clone, Debug, Default)]
pub struct Circle {
    note_type: NoteType,
    // txt: String,
    gogo: bool,
    bpm: f32,
    scroll: f32,
    section: bool,
    branch: Option<BranchObject>,

    end_time: Option<f32>,

    end_drumroll: Option<EndDrumroll>,
    required_hits: Option<usize>
}

#[derive(Clone, Debug)]
pub struct EndDrumroll {
    note_type: NoteType,
    // txt: String,
    gogo: bool,
    bpm: f32,
    scroll: f32,
    section: bool,
}

fn circle_to_end_drumroll(circle: &Circle) -> EndDrumroll {
    EndDrumroll {
        note_type: circle.note_type,
        // txt: String,
        gogo: circle.gogo,
        bpm: circle.bpm,
        scroll: circle.scroll,
        section: circle.section,
    }
}


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NoteType {
    None , //= "0",
    Don , //= "1",
    Ka , //= "2",
    DaiDon , //= "3",
    DaiKa , //= "4",
    Drumroll , //= "5",
    DaiDrumroll , //= "6",
    Balloon , //= "7",
}
impl Default for NoteType {
    fn default() -> Self {Self::None}
}
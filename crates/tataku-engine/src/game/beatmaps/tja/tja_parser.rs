use crate::prelude::*;
use super::tja_beatmap::*;

/// helper for parsing .tja files
#[derive(Default)]
pub struct TjaParser {
    title: String,
    title_en: String,

    subtitle: String,
    subtitle_en: String,

    creator: String,

    bpm: f32,
    offset: f32,
    audio_preview: f32,

    audio_filename: String,
    image_filename: String,

    current_course: ParseCourse,
    courses: Vec<ParseCourse>,

    course_lines: Vec<String>
}

impl TjaParser {
    pub fn parse<'a>(mut self, lines: impl Iterator<Item=&'a str>) -> TatakuResult<Vec<TjaBeatmap>> {
        self.bpm = 120.0;
        self.offset = 0.0;

        for line in lines {
            let Some(line) = line.split("//").next() else {continue };

            if line.is_empty() { continue }
            // i am aware of the potential issue this causes, but it should be fine
            self.course_lines.push(line.to_owned());

            if line.starts_with("#") { 
                if !self.current_course.is_valid() {
                    self.current_course = ParseCourse::new(&self);
                }

                self.current_course.parse_course_line(line);

                if self.current_course.complete {
                    self.complete_course();
                }
            } else if line.contains(":") {
                self.parse_metadata(line);
            } else {
                self.current_course.parse_notes_line(line)?;
            }
        }

        if self.current_course.complete {
            self.complete_course();
        }

        Ok(self.courses.into_iter().map(|c|c.course).collect())
    }

    pub fn parse_metadata(&mut self, line: &str) {
        let mut split = line.split(":");
        let Some(property) = split.next() else { return };
        let property = property.to_lowercase();
        let Some(value) = split.next() else { return };
        if value.is_empty() { return };

        match &*property {
            "title" => self.title = value.to_owned(),
            "subtitle" => self.subtitle = value.to_owned(),
            "wave" => self.audio_filename = value.to_owned(),
            "notedesigner" => self.creator = value.to_owned(),
            "maker" => self.creator = value.to_owned(),

            "bpm" => self.bpm = value.parse().unwrap_or_default(),
            "offset" => self.offset = value.parse().unwrap_or_default(),
            "demostart" => self.audio_preview = value.parse().unwrap_or_default(),

            // current course properties
            _=> self.current_course.add_metadata(&property, value),
        }
    }

    pub fn get_default_beatmap(&self) -> TjaBeatmap {
        TjaBeatmap {
            title: self.title_en.clone(),
            title_unicode: self.title.clone(),
            subtitle: self.subtitle_en.clone(),
            subtitle_unicode: self.subtitle.clone(),
            course_creator: self.creator.clone(),

            bpm: self.bpm,
            offset: -self.offset * 1000.0,
            audio_path: self.audio_filename.clone(),
            image_path: self.image_filename.clone(),
            preview_time: self.audio_preview * 1000.0,

            ..Default::default()
        }
    }

    pub fn complete_course(&mut self) {
        let mut course = ParseCourse::new(self);
        std::mem::swap(&mut self.current_course, &mut course);
        course.course.hash = md5(std::mem::take(&mut self.course_lines).join("\n"));

        self.courses.push(course);
    }
}


#[derive(Default)]
struct ParseCourse {
    course: TjaBeatmap,

    /// list of required hits for the balloons
    required_hits: Vec<usize>,
    used_required_hits: usize,

    last_long: LongType,

    current_bpm: f32,
    current_measure: f32,
    current_measure_int: u8,
    current_time: f32,

    current_branch: Option<TjaBranch>,
    current_branch_group: Option<TjaBranchGroup>,

    current_measure_events: Vec<MeasureEvent>,

    in_song: bool,
    complete: bool,
}
impl ParseCourse {
    pub fn new(parser: &TjaParser) -> Self { 
        Self {
            course: parser.get_default_beatmap(),
            current_time: -parser.offset * 1000.0,
            current_bpm: parser.bpm,
            current_measure: 1.0,
            current_measure_int: 4,
            
            ..Default::default()
        }
    }
    fn is_valid(&self) -> bool { self.current_bpm > 0.0 }

    fn add_metadata(&mut self, key: &str, val: &str) {
        match key {
            "course" => self.course.course_name = val.to_owned(),
            "level" => self.course.course_level = val.parse().unwrap_or_default(),
            "balloon" => self.required_hits = val.split(",").map(|f|f.parse().unwrap_or(0)).collect(),
            
            "scoreinit" => {}, //self.course.score_init = value.parse().unwrap_or_default(),
            "scorediff" => {}, //self.course.score_diff = value.parse().unwrap_or_default(),

            _ => warn!("unknown metadata property: {key} (value {val})"),
        }
    }

    fn parse_course_line(&mut self, line: &str) {
        let mut split = line.split(" ");
        let property = split.next().unwrap().to_lowercase();
        let value = split.next().unwrap_or("");

        match &*property {
            "#start" => self.in_song = true,
            // course parsing is complete, we're done
            "#end" => self.complete = true,
            
            "#lyric" => {
                // if let Some(course) = courses.get_mut(&current_course_name) {
                //     course.branch = true;
                // }
                // lyrics_line = value.replace("\\n", "\n").to_owned();
            }

            "#gogostart" => self.add_course_event(TjaCourseEventType::Kiai(true)),
            "#gogoend" => self.add_course_event(TjaCourseEventType::Kiai(false)),
            "#bpmchange" => {
                let Ok(bpm) = value.parse() else { return };
                self.current_bpm = bpm;
                self.add_course_event(TjaCourseEventType::Bpm(bpm));
            }

            "#scroll" => {
                let Ok(scroll) = value.parse() else { return };
                self.add_course_event(TjaCourseEventType::Scroll(scroll));
            }
            "#measure" => {
                let mut split2 = value.split("/");
                let (numerator, denominator):(i32,i32) = match split2.next().zip(split2.next()).and_then(|f|f.0.parse().ok().zip(f.1.parse().ok())) {
                    Some(s) => s,
                    None => return
                };
                self.current_measure_int = numerator as u8;
                self.current_measure = numerator as f32 / denominator as f32; // * 4.0;
                self.add_course_event(TjaCourseEventType::Measure(numerator as u8));
            }

            "#delay" => self.current_measure_events.push(MeasureEvent::Delay(value.parse::<f32>().unwrap_or_default() * 1000.0)), //self.current_time += ,
            "#barlineon" => self.add_course_event(TjaCourseEventType::BarLine(true)),
            "#barlineoff" => self.add_course_event(TjaCourseEventType::BarLine(false)),
            "#section" => self.add_course_event(TjaCourseEventType::Section),

            "#branchstart" => self.add_branch_group(value),
            "#branchend" => self.complete_branch(),

            "#n" | "#e" | "#m" => {
                let Some(branch_group) = &mut self.current_branch_group else { return };
            
                // add the current branch if exists
                if let Some(branch) = std::mem::take(&mut self.current_branch) {
                    branch_group.branches.insert(branch.diff, branch);
                }

                let diff = match &*property {
                    "#n" => BranchDifficulty::Normal,
                    "#e" => BranchDifficulty::Advanced,
                    "#m" => BranchDifficulty::Master,
                    _ => unreachable!()
                };

                // reset the current time to the branch group's start time so notes are lines up correctly
                self.current_time = branch_group.start_time;

                // set the current branch
                self.current_branch = Some(TjaBranch { diff, ..Default::default() });
            }

            _ => {}
        }
    }

    fn parse_notes_line(&mut self, line: &str) -> TatakuResult {
        if !self.in_song { return Ok(()) }

        let line = line.trim(); // remove any whitespace
        for symbol in line.to_uppercase().chars() {

            match symbol {
                ' ' => {}
                // empty note
                '0' => self.current_measure_events.push(MeasureEvent::EmptyNote),
                
                '1'|'2'|'3'|'4'|'A'|'B' => self.current_measure_events.push(MeasureEvent::Circle(TjaCircle {
                    time: 0.0,
                    is_don: ['1','3','A'].contains(&symbol),
                    is_big: ['3','4','A','B'].contains(&symbol),
                })),
                

                // drumroll
                '5'|'6' => self.add_drumroll(symbol == '6'),
                
                // balloon
                '7'|'9' => self.add_balloon(),
                
                // drumroll and balloon end 
                '8' => self.finish_long(),

                // // any other letter add as empty note
                // 'C'..='Z' => self.current_measure_events.push(MeasureEvent::EmptyNote),
                
                // end of measure
                ',' => self.complete_measure(),

                // cry
                _ => {
                    warn!("unknown tja note char '{symbol}'");
                    return Err(BeatmapError::InvalidFile.into())
                }

            }
        }

        Ok(())
    }

    fn add_drumroll(&mut self, is_big: bool) {
        self.last_long = LongType::Drumroll;
        self.current_measure_events.push(MeasureEvent::Drumroll(TjaDrumroll { 
            time: 0.0, 
            end_time: f32::NAN, 
            is_big
        }));
    }

    fn add_balloon(&mut self) {
        // if we already have a balloon, this is the end (specifically for the 9 type but idc)
        if let LongType::Balloon = self.last_long {
            self.finish_long();
            return;
        }
        self.last_long = LongType::Balloon;
        let hits_required = self.next_required_hits();
        self.current_measure_events.push(MeasureEvent::Balloon(TjaBalloon { 
            time: 0.0, 
            end_time: f32::NAN, 
            hits_required,
        }));
    }
    fn finish_long(&mut self) {
        self.current_measure_events.push(MeasureEvent::LongEnd(self.last_long));
        self.last_long = LongType::None;
    }


    // get the length of the current measure in ms
    fn measure_length(&self) -> f32 {
        // 60_000 * MEASURE * 4 / BPM
        60_000.0 / self.current_bpm * self.current_measure * 4.0
    }
    fn next_required_hits(&mut self) -> usize {
        let required_hits = self.required_hits.get(self.used_required_hits).cloned().unwrap_or_default();
        self.used_required_hits += 1;
        required_hits
    }

    fn complete_measure(&mut self) {
        // process the current measure events
        let time_step = self.events_step_length();
        if self.current_measure_events.is_empty() {
            self.current_time += time_step;
        }
        std::mem::take(&mut self.current_measure_events).into_iter().for_each(|n|self.add_note(n, time_step));
    }

    /// get the step length for the current measure events
    fn events_step_length(&self) -> f32 {
        let measure_length = self.measure_length();

        let mut events_count = 0;
        for i in self.current_measure_events.iter() {
            match i {
                MeasureEvent::Delay(_) => {}
                _ => events_count += 1,
            }
        }
        if events_count == 0 {
            return measure_length;
        }

        measure_length / events_count as f32
    }


    fn add_note(&mut self, mut note: MeasureEvent, mut time_step: f32) {
        let time = self.current_time;
        note.set_time(time);

        if let Some(branch) = &mut self.current_branch {
            match note {
                MeasureEvent::EmptyNote => {}
                MeasureEvent::Delay(delay) => time_step += delay,
                MeasureEvent::Circle(c) => branch.circles.push(c),
                MeasureEvent::Balloon(b) => branch.balloons.push(b),
                MeasureEvent::Drumroll(d) => branch.drumrolls.push(d),
                MeasureEvent::LongEnd(last_long) => self.get_last_long(last_long).as_mut().map(|l|l.set_end_time(time)).nope(),
            }
        } else {
            match note {
                MeasureEvent::EmptyNote => {}
                MeasureEvent::Delay(delay) => time_step += delay,
                MeasureEvent::Circle(c) => self.course.circles.push(c),
                MeasureEvent::Balloon(b) => self.course.balloons.push(b),
                MeasureEvent::Drumroll(d) => self.course.drumrolls.push(d),
                MeasureEvent::LongEnd(last_long) => self.get_last_long(last_long).as_mut().map(|l| l.set_end_time(time)).nope(),
            }
        }

        self.current_time += time_step;
    }

    /// get the next "long" note that hasnt had its end time set
    fn get_last_long(&mut self, long_type: LongType) -> Option<ExistingLongType> {
        match long_type {
            LongType::Balloon => {
                let list = if let Some(b) = &mut self.current_branch { &mut b.balloons } else { &mut self.course.balloons };
                list.iter_mut().find(|b| b.end_time.is_nan()).map(ExistingLongType::Balloon)
            }
            LongType::Drumroll => {
                let list = if let Some(b) = &mut self.current_branch { &mut b.drumrolls } else { &mut self.course.drumrolls };
                list.iter_mut().find(|d| d.end_time.is_nan()).map(ExistingLongType::Drumroll)
            }
            LongType::None => None
        }
        // match (long_type, &mut self.current_branch) {
        //     (LongType::None, _) => None,
        //     (LongType::Balloon, Some(b)) => b.balloons.iter_mut(),
        //     (LongType::Balloon, None) => self.course.balloons.iter_mut().find(|b|b.end_time.is_nan()).map(|b|ExistingLongType::Balloon(b)),
        //     (LongType::Drumroll, Some(b)) => b.drumrolls.iter_mut().find(|b|b.end_time.is_nan()).map(|d|ExistingLongType::Drumroll(d)),
        //     (LongType::Drumroll, None) => self.course.drumrolls.iter_mut().find(|b|b.end_time.is_nan()).map(|d|ExistingLongType::Drumroll(d)),
        // }
    }


    fn add_branch_group(&mut self, value: &str) {
        let mut val_split = value.split(",");
        let t = val_split.next().unwrap_or("p").trim(); // type
        let a = val_split.next().unwrap_or("0").trim(); // advaced requirement
        let m = val_split.next().unwrap_or("0").trim(); // master requirement

        let requirement = BranchRequirement {
            requirement_type: match t {
                "r" => BranchRequirementType::Drumroll,
                _ => BranchRequirementType::Accuracy,
            },
            advanced: a.parse().unwrap_or_default(),
            master: m.parse().unwrap_or_default(),
        };

        let branch_group = TjaBranchGroup {
            start_time: self.current_time,
            requirement,
            branches: HashMap::new(),
        };

        self.current_branch_group = Some(branch_group);
        self.add_course_event(TjaCourseEventType::Branch);
    }
    fn complete_branch(&mut self) {
        let Some(mut branch_group) = std::mem::take(&mut self.current_branch_group) else { return };
    
        // add the current branch if exists
        if let Some(branch) = std::mem::take(&mut self.current_branch) {
            branch_group.branches.insert(branch.diff, branch);
        }

        // add branch group
        self.course.branches.push(branch_group);
    }

    fn add_course_event(&mut self, event: TjaCourseEventType) {
        self.course.course_events.push(TjaCourseEvent { 
            time: self.current_time, 
            event,
        });
    }
}




#[derive(Copy, Clone, Debug, Default)]
enum LongType {
    #[default]
    None,
    Drumroll,
    Balloon
}

enum ExistingLongType<'a> {
    Drumroll(&'a mut TjaDrumroll),
    Balloon(&'a mut TjaBalloon),
}
impl<'a> ExistingLongType<'a> {
    fn set_end_time(&mut self, time: f32) {
        match self {
            Self::Balloon(b) => b.end_time = time,
            Self::Drumroll(d) => d.end_time = time,
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum MeasureEvent {
    EmptyNote,
    Delay(f32),
    Circle(TjaCircle),
    Drumroll(TjaDrumroll),
    Balloon(TjaBalloon),
    LongEnd(LongType),
}
impl MeasureEvent {
    fn set_time(&mut self, time: f32) {
        match self {
            Self::Balloon(b) => b.time = time,
            Self::Circle(c) => c.time = time,
            Self::Drumroll(d) => d.time = time,
            _ => {}
        }
    }
}



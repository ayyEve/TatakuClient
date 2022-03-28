use crate::prelude::*;

#[allow(unused, dead_code)]
#[derive(Clone, Debug)]
pub struct SkinSettings {
    // general
    pub name: String,
    pub author: String,
    pub version: f32,

    pub cursor_rotate: bool,
    pub cursor_expand: bool,
    pub cursor_center: bool,
    pub cursor_trail_rotate: bool,

    pub sliderball_frames: u8,
    pub hit_circle_overlay_above_number: bool,
    /// ??
    pub slider_style: u8, 
    pub sliderball_flip: bool,
    pub allow_sliderball_tint: bool,
    pub spinner_fade_playfield: bool,

    // colors
    pub combo_colors: Vec<Color>,
    pub slider_border: Color,
    pub slider_track_override: Color,
    /// ???
    pub spinner_approach_circle: Color,
    pub song_select_active_text: Color,
    pub song_select_inactive_text: Color,
    pub input_overlay_text: Color,

    // fonts
    pub hitcircle_prefix: String,
    pub hitcircle_overlap: u8,

    pub score_prefix: String,
    pub score_overlap: u8,

    pub combo_prefix: String,
    pub combo_overlap: u8,
    
    // mania
    pub mania_settings: Vec<ManiaSkinSettings>
}
#[allow(unused, dead_code)]
impl SkinSettings {
    pub fn from_file(path:String) -> TatakuResult<Self> {
        enum SkinSection {
            General,
            Colors, // colours
            Fonts,
            Mania,
        }

        let mut s = Self::default();

        // return defaults if skin does not exist
        if !exists(&path) {
            trace!("[Skin] skin.ini missing, using defaults");
            return Ok(s)
        }



        // read lines
        let mut current_area = SkinSection::General;
        let mut lines = read_lines(&path)?;

        while let Some(Ok(line)) = lines.next() {
            // split out comments, and trim wacky chars
            let line = line.split("//").next().unwrap().trim();
            // ignore empty lines (and comment-only lines)
            if line.len() == 0 {continue}
            
            // check for section change
            if line.starts_with("[") {
                match line {
                    "[General]" => current_area = SkinSection::General,
                    "[Colours]" => current_area = SkinSection::Colors,
                    "[Fonts]" => current_area = SkinSection::Fonts,
                    "[Mania]" => {
                        current_area = SkinSection::Mania;
                        let ms = ManiaSkinSettings::default();
                        s.mania_settings.push(ms);
                    },

                    _ => warn!("unknown skin section '{}'", line)
                }
                continue;
            }

            match current_area {
                SkinSection::General => {
                    let mut split = line.split(":");
                    let key = split.next().unwrap().trim();
                    let val = split.next().unwrap_or_default().trim();

                    let vbool = || {val == "1"};
                    
                    match &*key.to_lowercase() {
                        "name" => s.name = val.to_owned(),
                        "author" => s.author = val.to_owned(),
                        "version" => s.version = val.parse().unwrap_or_default(),
                        "sliderballflip" => s.sliderball_flip = vbool(),
                        "cursorrotate" => s.cursor_rotate = vbool(),
                        "cursortrailrotate" => s.cursor_trail_rotate = vbool(),
                        "cursorrxpand" => s.cursor_expand = vbool(),
                        "cursorcentre" => s.cursor_center = vbool(),
                        "sliderballframes" => s.sliderball_frames = val.parse().unwrap_or(60),
                        "hitcircleoverlayabovenumer" | "hitcircleoverlayabovenumber" => s.hit_circle_overlay_above_number = vbool(),
                        "sliderstyle" => s.slider_style = val.parse().unwrap_or_default(),
                        "allowsliderballTtint" => s.allow_sliderball_tint = vbool(),
                        "spinnerfadeplayfield" => s.spinner_fade_playfield = vbool(),
                        _ => {}
                    }
                    
                },
                SkinSection::Colors => {
                    let mut split = line.split(":");
                    let key = split.next().unwrap().trim();
                    let val = split.next().unwrap_or_default().trim();

                    let val2 = col(val.split(",").map(|s|s.parse::<u8>().unwrap_or_default()).collect::<Vec<u8>>().as_slice());
                    
                    match &*key.to_lowercase() {
                        "songselectactivetext" => s.song_select_active_text = val2,
                        "songselectinactivetext" => s.song_select_inactive_text = val2,
                        "sliderborder" => s.slider_border = val2,
                        "slidertrackoverride" => s.slider_track_override = val2,
                        "inputoverlaytext" => s.input_overlay_text = val2,
                        
                        _ => {}
                    }
                },
                SkinSection::Fonts => {
                    let mut split = line.split(":");
                    let key = split.next().unwrap().trim();
                    let val = split.next().unwrap_or_default().trim();
                    
                    match &*key.to_lowercase() {
                        "hitcircleprefix" => s.hitcircle_prefix = val.to_owned(),
                        "hitcircleoverlap" => s.hitcircle_overlap = val.parse().unwrap_or_default(),
                        "scoreprefix" =>  s.score_prefix = val.to_owned(),
                        "scoreoverlap" => s.score_overlap = val.parse().unwrap_or_default(),
                        "comboprefix" =>  s.combo_prefix = val.to_owned(),
                        "combooverlap" => s.combo_overlap = val.parse().unwrap_or_default(),
                        
                        _ => {}
                    }
                },
                SkinSection::Mania => {
                    let mut split = line.split(":");
                    let key = split.next().unwrap().trim();
                    let val = split.next().unwrap_or_default().trim().to_owned();

                    let len = s.mania_settings.len();
                    let s = &mut s.mania_settings[len - 1];

                    if key.starts_with("KeyImage") {
                        let num:u8 = key.trim_start_matches("KeyImage").trim_end_matches("D").parse().unwrap_or(10);
                        if num > 9 {continue}

                        if key.ends_with("D") {
                            s.key_image_d.insert(num, val);
                        } else {
                            s.key_image.insert(num, val);
                        }
                    } else if key.starts_with("NoteImage") {
                        let num:u8 = key
                            .trim_start_matches("NoteImage")
                            .trim_end_matches("H")
                            .trim_end_matches("L")
                            .trim_end_matches("T")
                            .parse()
                            .unwrap_or(10);
                        if num > 9 {continue}

                        if key.ends_with("H") {
                            s.note_image_h.insert(num, val);
                        } else if key.ends_with("L") {
                            s.note_image_l.insert(num, val);
                        } else if key.ends_with("T") {
                            s.note_image_t.insert(num, val);
                        } else {
                            s.note_image.insert(num, val);
                        }
                    } else {
                        match &*key {
                            "Keys" => s.keys = val.parse().unwrap_or_default(),
                            

                            _ => {}
                        }
                    }

                },
            }
        }
        
        Ok(s)
    }
}
impl Default for SkinSettings {
    fn default() -> Self {
        Self {
            // general
            name: "Default".to_owned(),
            author: "Tataku".to_owned(),
            version: 1.0,

            cursor_rotate: true,
            cursor_expand: true,
            cursor_center: true,
            cursor_trail_rotate: false,
            
            sliderball_frames: 1,
            hit_circle_overlay_above_number: false,
            /// ??
            slider_style: 2, 
            sliderball_flip: false,
            allow_sliderball_tint: false,
            spinner_fade_playfield: false,

            // colors
            combo_colors: vec![
                col(&[0,255,0]),
                col(&[0,255,255]),
                col(&[255,128,255]),
                col(&[255,255,0]),
            ],
            slider_border: col(&[250,250,250]),
            slider_track_override: col(&[0,0,0]),
            /// ???
            spinner_approach_circle: col(&[77,139,217]),
            song_select_active_text: col(&[255,255,255]),
            song_select_inactive_text: col(&[255,255,255]),
            input_overlay_text: col(&[255,255,255]),

            // fonts
            hitcircle_prefix: "default".to_owned(),
            hitcircle_overlap: 0,
            score_prefix: "score".to_owned(),
            score_overlap: 0,
            combo_prefix: "combo".to_owned(),
            combo_overlap: 0,

            // mania
            mania_settings: Vec::new(),
        }
    }
}

fn col(b:&[u8]) -> Color {
    Color::new(
        b[0] as f32 / 255.0, 
        b[1] as f32 / 255.0, 
        b[2] as f32 / 255.0, 
        1.0
    )
}

#[derive(Clone, Default, Debug)]
pub struct ManiaSkinSettings {
    pub keys: u8,

    pub colors:      HashMap<u8, Color>,
    pub key_image:   HashMap<u8, String>,
    pub key_image_d: HashMap<u8, String>,

    pub note_image:   HashMap<u8, String>,
    pub note_image_h: HashMap<u8, String>,
    pub note_image_l: HashMap<u8, String>,
    pub note_image_t: HashMap<u8, String>,
}
use crate::prelude::*;

#[allow(unused, dead_code)]
pub struct SkinSettings {
    // general
    name: String,
    author: String,

    cursor_rotate: bool,
    cursor_expand: bool,
    cursor_center: bool,

    sliderball_frames: u8,
    hit_circle_overlay_above_number: bool,
    /// ??
    slider_style: u8, 


    // colors
    combo_colors: Vec<Color>,
    slider_border: Color,
    slider_track_override: Color,
    /// ???
    spinner_approach_circle: Color,
}
#[allow(unused, dead_code)]
impl SkinSettings {
    fn from_file(path:String) -> TatakuResult<Self> {
        enum SkinSection {
            General,
            Colors, // colours
            Fonts,
            Mania {keys:u8},
        }

        let mut s = Self::default();


        // read lines
        let mut current_area = SkinSection::General;
        let mut lines = read_lines(&path)?;
        for line in lines {

        }

        
        Ok(s)
    }
}
impl Default for SkinSettings {
    fn default() -> Self {
        Self {
            // general
            name: "Default".to_owned(),
            author: "ayyEve".to_owned(),

            cursor_rotate: true,
            cursor_expand: true,
            cursor_center: true,

            sliderball_frames: 1,
            hit_circle_overlay_above_number: false,
            /// ??
            slider_style: 2, 

            // colors
            combo_colors: vec![
                col([0,255,0]),
                col([0,255,255]),
                col([255,128,255]),
                col([255,255,0]),
            ],
            slider_border: col([250,250,250]),
            slider_track_override: col([0,0,0]),
            /// ???
            spinner_approach_circle: col([77,139,217]),
        }
    }
}

fn col(b:[u8;3]) -> Color {
    Color::new(
        b[0] as f32 / 255.0, 
        b[1] as f32 / 255.0, 
        b[2] as f32 / 255.0, 
        1.0
    )
}

// impl Into<Color> for [u8;3] {
//     fn into(self) -> Color {
//         
//     }
// }

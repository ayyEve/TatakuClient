use crate::prelude::*;

#[derive(Clone, Debug)]
pub struct NoteDef {
    /// Position in osu! pixels of the object.
    pub pos: Vector2,

    /// Time when the object is to be hit, in milliseconds from the beginning of the beatmap's audio.
    pub time: f32,
    /// Bit flags indicating the hitsound applied to the object
    pub hitsound: u8,
    pub hitsamples: HitSamples,
    pub new_combo: bool,
    pub color_skip: u8,
}


#[derive(Clone, Debug)]
pub struct SliderDef {
    // x,y,time,type,hitSound,curveType|curvePoints,slides,length,edgeSounds,edgeSets,hitSample
    pub raw: String,

    /// Position in osu! pixels of the object.
    pub pos: Vector2,
    /// Time when the object is to be hit, in milliseconds from the beginning of the beatmap's audio.
    pub time: f32,
    /// Bit flags indicating the hitsound applied to the object
    pub hitsound: u8,
    pub curve_type: CurveType,
    pub curve_points: Vec<Vector2>,
    pub slides: u64,
    pub length: f32,
    pub edge_sounds: Vec<u8>,
    pub edge_sets: Vec<[u8;2]>,
    pub new_combo: bool,
    pub color_skip: u8,
    
    /// Information about which samples are played when the object is hit. It is closely related to hitSound;
    pub hitsamples: HitSamples,
}


#[derive(Clone, Debug)]
pub struct SpinnerDef {
    pub pos: Vector2,
    pub time: f32,
    pub hitsound: u8,
    pub end_time: f32,
    pub new_combo: bool,
    pub color_skip: u8,
    
    pub hitsamples: HitSamples
}


#[derive(Clone, Debug)]
pub struct HoldDef {
    pub pos: Vector2,
    pub time: f32,
    pub hitsound: u8,
    pub end_time: f32,
    
    pub hitsamples: HitSamples
}



#[derive(Clone, Copy, Debug)]
pub enum CurveType {
    Bézier,
    Catmull,
    Linear,
    Perfect
}


#[derive(Clone, Debug, Default)]
pub struct HitSamples {
    // Hit sample syntax: normalSet:additionSet:index:volume:filename

    /// Sample set of the normal sound.
    pub normal_set: u8,
    /// Sample set of the whistle, finish, and clap sounds.
    pub addition_set: u8,
    /// Index of the sample. If this is 0, the timing point's sample index will be used instead.
    pub index: u8,
    /// Volume of the sample from 1 to 100. If this is 0, the timing point's volume will be used instead.
    pub volume: u8,
    /// Custom filename of the addition sound.
    pub filename: Option<String>
}
impl HitSamples {
    pub fn from_str(str:Option<&str>) -> Self {

        macro_rules! read_val {
            ($split:expr) => {
                $split.next().unwrap_or("0").parse().unwrap_or(0)
            };
        }

        match str {
            None => Self::default(),
            Some(str) => {
                let mut split = str.split(':');

                let normal_set = read_val!(split); //split.next().unwrap_or("0").parse().unwrap_or(0);
                let addition_set = split.next().unwrap_or("0").parse().unwrap_or(0);
                let index = split.next().unwrap_or("0").parse().unwrap_or(0);
                let volume = split.next().unwrap_or("0").parse().unwrap_or(0);
                // i wonder if this can be simplified
                let filename = split.next().map(|s| s.to_owned());
                Self {
                    normal_set,
                    addition_set,
                    index,
                    volume,
                    filename,
                }
            }
        }

    }
}

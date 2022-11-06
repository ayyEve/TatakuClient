#![allow(unused)]
use crate::prelude::*;
use std::convert::TryInto;

pub fn convert_osu_replay(filepath: impl AsRef<Path>) -> TatakuResult<Replay> {
    let osu_replay = read_osu_replay(filepath)?;
    Ok(osu_replay.get_replay())
}

fn read_osu_replay(file: impl AsRef<Path>) -> TatakuResult<OsuReplay> {
    let file = std::fs::read(file)?;
    let file = file.as_slice();
    let mut offset = 0;

    let game_mode = read_byte(file, &mut offset)?;
    let game_version = read_int(file, &mut offset)?;
    let map_hash = read_string(file, &mut offset)?;
    let username = read_string(file, &mut offset)?;
    let replay_hash = read_string(file, &mut offset)?;

    let x300 = read_short(file, &mut offset)?;
    let x100 = read_short(file, &mut offset)?;
    let x50 = read_short(file, &mut offset)?;
    let geki = read_short(file, &mut offset)?;
    let katu = read_short(file, &mut offset)?;
    let miss = read_short(file, &mut offset)?;

    let score = read_int(file, &mut offset)?;
    let max_combo = read_short(file, &mut offset)?;
    
    let perfect = read_byte(file, &mut offset)? == 1;
    let mods_num = read_int(file, &mut offset)?;
    let health_str = read_string(file, &mut offset)?;
    let timestamp = read_long(file, &mut offset)?;

    let data_len = read_int(file, &mut offset)? as usize;
    
    if offset + (data_len - 1) >= file.len() { return Err(TatakuError::String(format!("buffer overflow"))) }
    let mut replay_data = &file[offset..(offset + data_len)]; offset += data_len;

    let score_id = read_long(file, &mut offset)?;

    // convert game mode
    let game_mode = match game_mode {
        0 => "osu",
        1 => "taiko",
        2 => "catch",
        3 => "mania",
        _ => ""
    }.to_owned();

    // parse health
    // Life bar graph: comma separated pairs u/v, where u is the time in milliseconds into the song and v is a floating point value from 0 - 1 that represents the amount of life you have at the given time (0 = life bar is empty, 1= life bar is full)
    let mut health = Vec::new();
    for i in health_str.split(",") {
        if i.len() == 0 { continue }
        let mut split2 = i.split("|");

        macro_rules! parse {
            ($seg:expr) => {{
                let str = split2.next().ok_or(TatakuError::String(format!("missing {} segment in health string", $seg)))?;
                str.parse().map_err(|e| TatakuError::String(format!("parse err: {e}")))?
            }};
        }

        let time:u32  = parse!("time");
        let value:f32 = parse!("health");
        health.push(OsuHealth { time, value });
    }

    // parse mods
    let mut mods = Vec::new();
    for i in 0..32 {
        let i = 2u32.pow(i);
        if (mods_num & i) > 0 {
            mods.push(OsuMods::from_num(i));
        }
    }


    // parse replay data
    let mut replay_frames = parse_lzma_stream(&mut replay_data)?;

    if game_version >= 20130319 {
        // last one is rng data
        replay_frames.pop();
    }


    Ok(OsuReplay {
        game_mode,
        game_version,
        map_hash,
        username,
        replay_hash,
        x300,
        x100,
        x50,
        geki,
        katu,
        miss,
        score,
        max_combo,
        perfect,
        mods,
        health,
        timestamp,
        score_id,
        replay_frames,
    })
}


fn parse_lzma_stream(lzma: &mut impl std::io::BufRead) -> TatakuResult<Vec<OsuReplayFrame>>{
    let mut replay_data_decompressed = Vec::new();
    if let Err(e) = lzma_rs::lzma_decompress(lzma, &mut replay_data_decompressed) {
        return Err(TatakuError::String(format!("Error decompressing replay data")))
    }
    let replay_str = String::from_utf8_lossy(&replay_data_decompressed);

    let mut replay_frames = Vec::new();
    let mut accumulated_time = 0;
    for i in replay_str.split(",") {
        if i.len() == 0 { continue }
        let mut split2 = i.split("|");

        macro_rules! parse {
            ($seg:expr) => {{
                let str = split2.next().ok_or(TatakuError::String(format!("missing {} segment in replay string", $seg)))?;
                str.parse().map_err(|e| TatakuError::String(format!("{e}")))?
            }};
        }
        
        let time:i64 = parse!("time");
        let x:f32    = parse!("x");
        let y:f32    = parse!("y");
        let keys:u32 = parse!("keys");
        accumulated_time += time;

        let mut key_presses = Vec::new();
        for i in 0..32 {
            let i = 2u32.pow(i);
            if (keys & i) > 0 {
                key_presses.push(OsuKeys::from_num(i));
            }
        }
        
        replay_frames.push(OsuReplayFrame {
            time: accumulated_time,
            x,
            y,
            keys: key_presses
        });
    }

    Ok(replay_frames)
}


macro_rules! read_num {
    ($bytes:expr, $offset: expr, $t:ident) => {{
        let len = (<$t>::BITS / 8) as usize;

        if *$offset + (len - 1) >= $bytes.len() { return Err(TatakuError::String(format!("buffer overflow"))); }

        let val = <$t>::from_le_bytes($bytes[*$offset..(*$offset + len)].try_into().unwrap());
        *$offset += len;
        Ok(val)
    }}
}

fn read_byte(bytes: &[u8], offset:&mut usize) -> TatakuResult<u8> {
    if *offset >= bytes.len() { return Err(TatakuError::String(format!("buffer overflow"))); }

    let b = bytes[*offset];
    *offset += 1;
    Ok(b)
}

fn read_short(bytes: &[u8], offset:&mut usize) -> TatakuResult<u16> {
    read_num!(bytes, offset, u16)
}
fn read_int(bytes: &[u8], offset:&mut usize) -> TatakuResult<u32> {
    read_num!(bytes, offset, u32)
}
fn read_long(bytes: &[u8], offset:&mut usize) -> TatakuResult<u64> {
    read_num!(bytes, offset, u64)
}

fn read_string(bytes: &[u8], offset:&mut usize) -> TatakuResult<String> {
    let b = bytes[*offset];
    *offset += 1;

    if b == 0x00 {
        Ok(String::new())
    } else if b == 0x0b {
        let len = read_uleb128(bytes, offset) as usize;
        // println!("got string len {len}");

        let string = String::from_utf8(bytes[*offset..(*offset+len)].to_vec()).map_err(|e|format!("error parsing string: {e}"))?;
        *offset += len;
        Ok(string)
    } else {
        Err(TatakuError::String(format!("wrong first byte for uleb: {:X}", b)))
    }
}

fn read_uleb128(bytes: &[u8], offset:&mut usize) -> u128 {
    let mut result:u128 = 0;
    let mut shift = 0;
    loop {
        let byte = bytes[*offset];
        *offset += 1;

        result |= ((byte & 0x7f) as u128) << shift;
        if byte & 0x80 == 0 {
            return result;
        }

        shift += 7;
    }
}



#[derive(Clone, Debug)]
pub struct OsuReplay {
    game_mode: String,
    game_version: u32,
    map_hash: String,
    username: String,
    replay_hash: String,

    x300: u16,
    x100: u16,
    x50: u16,
    geki: u16,
    katu: u16,
    miss: u16,
    score: u32,
    max_combo: u16,

    perfect: bool,
    mods: Vec<OsuMods>,
    health: Vec<OsuHealth>,
    timestamp: u64,
    score_id: u64,

    replay_frames: Vec<OsuReplayFrame>
}
impl OsuReplay {
    pub fn get_score(&self) -> Score {
        let mut score = Score::new(self.map_hash.clone(), self.username.clone(), self.game_mode.clone());
        score.score = self.score as u64;
        score.max_combo = self.max_combo;
        
        for (key, count) in [
            ("x300", self.x300),
            ("x100", self.x100),
            ("x50", self.x50),
            ("xgeki", self.geki),
            ("xkatu", self.katu),
            ("xmiss", self.miss),
        ] {
            score.judgments.insert(key.to_owned(), count);
        }

        if self.mods.contains(&OsuMods::DoubleTime) {
            score.speed = 1.33;
        } else if self.mods.contains(&OsuMods::HalfTime) {
            score.speed = 0.75;
        }
        
        // mods
        {
            let mods = score.mods_mut();
            if self.mods.contains(&OsuMods::Easy) { mods.insert("easy".to_owned()); }
            if self.mods.contains(&OsuMods::HardRock) { mods.insert("hard_rock".to_owned()); }
            if self.mods.contains(&OsuMods::Autoplay) { mods.insert("autoplay".to_owned()); }
            if self.mods.contains(&OsuMods::NoFail) { mods.insert("no_fail".to_owned()); }
        }

        score
    }


    pub fn get_replay(&self) -> Replay {
        let mut replay = Self::parse_frames(&self.game_mode, &self.replay_frames);
        replay.score_data = Some(self.get_score());
        replay
    }

    

    pub fn replay_from_score_and_lzma(score: &Score, lzma: &mut impl std::io::BufRead) -> TatakuResult<Replay> {
        let frames = parse_lzma_stream(lzma)?;
        let mut replay = Self::parse_frames(&score.playmode, &frames);
        replay.score_data = Some(score.clone());

        Ok(replay)
    }


    fn parse_frames(game_mode: &String, replay_frames: &Vec<OsuReplayFrame>) -> Replay {
        let mut replay = Replay::new();

        // mania keys are stored in the x pos as bitflags
        if game_mode == "mania" {
            let mut pressed_keys = HashSet::new();

            for f in replay_frames.iter() {
                let pressed = f.x as u32;

                // i dont know what these are, theres one at -1 and 0 :/
                if f.y as i32 == -500 { continue }

                // check all keys (0 is mania1 here)
                for i in 0..=9 {
                    let i2 = 2u32.pow(i);

                    // check press
                    if (pressed & i2) > 0 {
                        if !pressed_keys.contains(&i) {
                            pressed_keys.insert(i);
                            replay.frames.push((f.time as f32, ReplayFrame::Press(get_mania_key(i))));
                        }
                    } else {
                        // check release
                        if pressed_keys.contains(&i) {
                            pressed_keys.remove(&i);
                            replay.frames.push((f.time as f32, ReplayFrame::Release(get_mania_key(i))));
                        }
                    }
                }

            }

        } else {
            let mut last_mouse_pos = Vector2::zero();
            let mut last_keys = Vec::new();

            for f in replay_frames.iter() {
                // check mouse pos
                let mouse_pos = Vector2::new(f.x as f64, f.y as f64);
                if last_mouse_pos != mouse_pos {
                    last_mouse_pos = mouse_pos;
                    replay.frames.push((f.time as f32, ReplayFrame::MousePos(f.x, f.y)));
                }

                // check press and release
                for k in KEYS.iter() {

                    // press 
                    if !last_keys.contains(k) && f.keys.contains(k) {
                        let key = k.to_keypress(&game_mode);

                        replay.frames.push((f.time as f32, ReplayFrame::Press(key)));
                    }
                    
                    // release 
                    if last_keys.contains(k) && !f.keys.contains(k) && game_mode != "taiko" {
                        let key = k.to_keypress(&game_mode);

                        replay.frames.push((f.time as f32, ReplayFrame::Release(key)));
                    }
                }

                last_keys = f.keys.clone();
            }
        }

        replay
    }



} 


#[derive(Clone, Debug)]
pub struct OsuReplayFrame {
    time: i64,
    x: f32,
    y: f32,
    keys: Vec<OsuKeys>
}

#[derive(Copy, Clone, Debug)]
pub struct OsuHealth {
    time: u32,
    value: f32
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum OsuMods {
    None, //	0	
    NoFail, //	1 (0)	
    Easy, //	2 (1)	
    TouchDevice, //	4 (2)	Replaces unused NoVideo mod
    Hidden, //	8 (3)	
    HardRock, //	16 (4)	
    SuddenDeath, //	32 (5)	
    DoubleTime, //	64 (6)	
    Relax, //	128 (7)	
    HalfTime, //	256 (8)	
    Nightcore, //	512 (9)	always used with DT : 512 + 64 = 576
    Flashlight, //	1024 (10)	
    Autoplay, //	2048 (11)	
    SpunOut, //	4096 (12)	
    Relax2, //	8192 (13)	Autopilot
    Perfect, //	16384 (14)	
    Key4, //	32768 (15)	
    Key5, //	65536 (16)	
    Key6, //	131072 (17)	
    Key7, //	262144 (18)	
    Key8, //	524288 (19)	
    KeyMod, //	1015808	k4+k5+k6+k7+k8
    FadeIn, //	1048576 (20)	
    Random, //	2097152 (21)	
    LastMod, //	4194304 (22)	Cinema
    TargetPractice, //	8388608 (23)	osu!cuttingedge only
    Key9, //	16777216 (24)	
    Coop, //	33554432 (25)	
    Key1, //	67108864 (26)	
    Key3, //	134217728 (27)	
    Key2, //	268435456 (28)	
    ScoreV2, //	536870912 (29)	
    Mirror, //	1073741824 (30)	
}
impl OsuMods {
    fn from_num(num: u32) -> Self {
        match num {
            0 => Self::None,
            1 => Self::NoFail,
            2 => Self::Easy,
            4 => Self::TouchDevice,
            8 => Self::Hidden,
            16 => Self::HardRock,
            32 => Self::SuddenDeath,
            64 => Self::DoubleTime,
            128 => Self::Relax,
            256 => Self::HalfTime,
            512 => Self::Nightcore,
            1024 => Self::Flashlight,
            2048 => Self::Autoplay,
            4096 => Self::SpunOut,
            8192 => Self::Relax2,
            16384 => Self::Perfect,
            32768 => Self::Key4,
            65536 => Self::Key5,
            131072 => Self::Key6,
            262144 => Self::Key7,
            524288 => Self::Key8,
            1015808 => Self::KeyMod,
            1048576 => Self::FadeIn,
            2097152 => Self::Random,
            4194304 => Self::LastMod,
            8388608 => Self::TargetPractice,
            16777216 => Self::Key9,
            33554432 => Self::Coop,
            67108864 => Self::Key1,
            134217728 => Self::Key3,
            268435456 => Self::Key2,
            536870912 => Self::ScoreV2,
            1073741824 => Self::Mirror,
            _ => Self::None,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum OsuKeys {
    None, // 0
    M1, // 1
    M2, // 2
    K1, // 4
    K2, // 8
    Smoke, // 16
    Other (u32)
}
impl OsuKeys {
    fn from_num(num: u32) -> Self {
        match num {
            0 => Self::None,
            1 => Self::M1,
            2 => Self::M2,
            4 => Self::K1,
            8 => Self::K2,
            16 => Self::Smoke,
            n => Self::Other(n),
        }
    }

    fn to_keypress(&self, playmode:&PlayMode) -> KeyPress {
        match (&**playmode, self) {
            // osu
            ("osu", Self::M1) => KeyPress::LeftMouse,
            ("osu", Self::M2) => KeyPress::RightMouse,
            ("osu", Self::K1) => KeyPress::Left,
            ("osu", Self::K2) => KeyPress::Right,
            ("osu", Self::Smoke) => KeyPress::Dash,

            // taiko
            ("taiko", Self::M1) => KeyPress::LeftDon,
            ("taiko", Self::M2) => KeyPress::LeftKat,
            ("taiko", Self::K1) => KeyPress::RightDon,
            ("taiko", Self::K2) => KeyPress::RightKat,

            _ => KeyPress::Unknown
        }
    }


}

const KEYS:&[OsuKeys] = &[
    OsuKeys::M1,
    OsuKeys::M2,
    OsuKeys::K1,
    OsuKeys::K2,
    OsuKeys::Smoke,
];


fn get_mania_key(i: u32) -> KeyPress {
    match i {
        0 => KeyPress::Mania1,
        1 => KeyPress::Mania2,
        2 => KeyPress::Mania3,
        3 => KeyPress::Mania4,
        4 => KeyPress::Mania5,
        5 => KeyPress::Mania6,
        6 => KeyPress::Mania7,
        7 => KeyPress::Mania8,
        8 => KeyPress::Mania9,
        _ => KeyPress::Unknown,
    }
}

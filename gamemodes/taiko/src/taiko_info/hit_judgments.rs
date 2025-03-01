use crate::prelude::*;


const COMBO_MULTIPLIER: ComboMultiplier = ComboMultiplier::Linear { 
    combo: 10, 
    multiplier: 1.1, 
    combo_cap: Some(80)
};

pub struct TaikoHitJudgments;
#[allow(non_upper_case_globals)]
impl TaikoHitJudgments {
    pub const Geki: HitJudgment = HitJudgment::new(
        "xgeki",
        "Geki",
        0.0,
        AffectsCombo::Ignore,
        300,
        COMBO_MULTIPLIER,
        Color::new(0.0, 0.7647, 1.0, 1.0),
        "taiko-hit300g",
        false,
        false,
    );

    pub const X300: HitJudgment = HitJudgment::new(
        "x300",
        "x300",
        3.0,
        AffectsCombo::Increment,
        300,
        COMBO_MULTIPLIER,
        Color::new(0.0, 0.7647, 1.0, 1.0),
        "taiko-hit300",
        false,
        false,
    );
    
    pub const Katu: HitJudgment = HitJudgment::new(
        "xkatu",
        "Katu",
        0.0,
        AffectsCombo::Ignore,
        100,
        COMBO_MULTIPLIER,
        Color::new(0.3411, 0.8901, 0.0745, 1.0),
        "taiko-hit100k",
        true,
        false,
    );

    pub const X100: HitJudgment = HitJudgment::new(
        "x100",
        "x100",
        1.0,
        AffectsCombo::Increment,
        100,
        COMBO_MULTIPLIER,
        Color::new(0.3411, 0.8901, 0.0745, 1.0),
        "taiko-hit100",
        true,
        false,
    );

    pub const Miss: HitJudgment = HitJudgment::new(
        "xmiss",
        "Miss",
        -10.0,
        AffectsCombo::Reset,
        0,
        ComboMultiplier::None,
        Color::new(0.9, 0.05, 0.05, 1.0),
        "taiko-hit0",
        true,
        true,
    );

    pub const SliderPoint: HitJudgment = HitJudgment::new(
        "slider_point",
        "",
        0.0,
        AffectsCombo::Ignore,
        10,
        COMBO_MULTIPLIER,
        Color::TRANSPARENT_WHITE,
        "",
        false,
        false,
    );

    pub const SpinnerPoint: HitJudgment = HitJudgment::new(
        "spinner_point",
        "",
        0.0,
        AffectsCombo::Ignore,
        100,
        COMBO_MULTIPLIER,
        Color::TRANSPARENT_WHITE,
        "",
        false,
        false,
    );

    pub const fn variants() -> &'static [HitJudgment] {
        &[
            Self::Geki,
            Self::X300,
            Self::Katu,
            Self::X100,
            Self::Miss,
            Self::SliderPoint,
            Self::SpinnerPoint,
        ]
    }
}





// #[derive(Copy, Clone, Debug, Eq, PartialEq)]
// pub enum TaikoHitJudgments {
//     Geki,
//     X300,
//     Katu,
//     X100,
//     Miss,

//     SliderPoint,
//     SpinnerPoint,
// }

// use TaikoHitJudgments::*;
// impl HitJudgments for TaikoHitJudgments {
//     fn variants(&self) -> Vec<Box<dyn HitJudgments>> {
//         vec![
//             Box::new(Geki),
//             Box::new(X300),
//             Box::new(Katu),
//             Box::new(X100),
//             Box::new(Miss),
//         ]
//     }

//     fn as_str_internal(&self) -> &'static str {
//         match self {
//             Geki => "xgeki",
//             X300 => "x300",
//             Katu => "xkatu",
//             X100 => "x100",
//             Miss => "xmiss",
//             SliderPoint => "slider_point",
//             SpinnerPoint => "spinner_point",
//         }
//     }

//     fn as_str_display(&self) -> &'static str {
//         match self {
//             Geki => "Geki",
//             X300 => "x300",
//             Katu => "Katu",
//             X100 => "x100",
//             Miss => "Miss",
//             _ => "you shouldnt see this"
//         }
//     }

//     fn get_health(&self) -> f32 {
//         match self {
//             X300 => 3.0,
//             X100 => 1.0,
//             Miss => -10.0,

//             // gekis and katus are just addons to existing judgments, they should not be given extra health
//             _ => 0.0
//         }
//     }

//     fn affects_combo(&self) -> AffectsCombo {
//         match self {
//             Miss => AffectsCombo::Reset,
            
//             // gekis and katus are just addons to existing judgments, they should count for combo
//             SliderPoint | SpinnerPoint | Geki | Katu  => AffectsCombo::Ignore,
//             _ => AffectsCombo::Increment,
//         }
//     }

//     fn get_score(&self, combo: u16) -> i32 {
//         let combo = (combo.clamp(0, 80) / 10).max(1) as i32;

//         combo * match self {
//             X300 | Geki => 300,
//             X100 | Katu => 100,

//             SliderPoint => 10,
//             SpinnerPoint => 100,

//             _ => 0,
//         }
//     }

//     // stolen from quaver
//     fn color(&self) -> Color {
//         match self {
//             X300 | Geki => Color::new(0.0, 0.7647, 1.0, 1.0),
//             X100 | Katu => Color::new(0.3411, 0.8901, 0.0745, 1.0),
//             Miss => Color::new(0.9, 0.05, 0.05, 1.0),
//             _ => Color::default()
//         }
//     }

//     fn tex_name(&self) -> &'static str {
//         match self {
//             Geki => "taiko-hit300g",
//             X300 => "taiko-hit300",
//             Katu => "taiko-hit100k",
//             X100 => "taiko-hit100",
//             Miss => "taiko-hit0",
//             _ => ""
//         }
//     }

//     /// does this judgment fail a perfect score?
//     fn fails_perfect(&self) -> bool { 
//         match self {
//             Geki | X300 | SliderPoint | SpinnerPoint => false,
//             _ => true
//         }
//     }

//     /// does this judgment fail a sudden death score?
//     fn fails_sudden_death(&self) -> bool {
//         match self {
//             Miss => true,
//             _ => false
//         }
//      }

// }

use crate::prelude::*;

const COMBO_MULTIPLIER: ComboMultiplier = ComboMultiplier::Linear { 
    combo: 10, 
    multiplier: 1.1, 
    combo_cap: Some(80)
};

pub struct OsuHitJudgments;
#[allow(non_upper_case_globals)]
impl OsuHitJudgments {
    pub const X300: HitJudgment = HitJudgment::new(
        "x300",
        "x300",
        3.0,
        AffectsCombo::Increment,
        300,
        COMBO_MULTIPLIER,
        Color::new(0.0, 0.7647, 1.0, 1.0),
        "hit300",
        false, 
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
        "hit100",
        true, 
        false,
    );
    pub const X50: HitJudgment = HitJudgment::new(
        "x50",
        "x50",
        -2.0,
        AffectsCombo::Increment,
        50,
        COMBO_MULTIPLIER,
        Color::new(0.8549, 0.6823, 0.2745, 1.0),
        "hit50",
        true, 
        true,
    );
    pub const Miss: HitJudgment = HitJudgment::new(
        "xmiss",
        "Miss",
        -10.0,
        AffectsCombo::Reset,
        0,
        ComboMultiplier::None,
        Color::new(0.9, 0.05, 0.05, 1.0),
        "hit0",
        true, 
        true,
    );

    pub const SliderDot: HitJudgment = HitJudgment::new(
        "slider_dot",
        "",
        1.0,
        AffectsCombo::Increment,
        100,
        ComboMultiplier::None,
        Color::new(0.0, 0.0, 0.0, 0.0),
        "",
        false, 
        false,
    );
    pub const SliderDotMiss: HitJudgment = HitJudgment::new(
        "slider_dot_miss",
        "",
        -2.0,
        AffectsCombo::Reset,
        0,
        ComboMultiplier::None,
        Color::new(0.9, 0.05, 0.05, 1.0),
        "",
        true, 
        true,
    );

    pub const SliderEnd: HitJudgment = HitJudgment::new(
        "slider_end",
        "",
        1.0,
        AffectsCombo::Increment,
        0, // TODO: is this correct?
        COMBO_MULTIPLIER,
        Color::new(0.0, 0.0, 0.0, 0.0),
        "",
        false, 
        false,
    );
    pub const SliderEndMiss: HitJudgment = HitJudgment::new(
        "xmiss", // alias to miss, so it counts as misses when added
        "",
        -5.0,
        AffectsCombo::Reset,
        0,
        ComboMultiplier::None,
        Color::new(0.9, 0.05, 0.05, 1.0),
        "hit0",
        true, 
        true,
    );

    pub const SpinnerMiss: HitJudgment = HitJudgment::new(
        "xmiss",
        "",
        -5.0,
        AffectsCombo::Reset,
        0,
        ComboMultiplier::None,
        Color::new(0.0, 0.0, 0.0, 0.0),
        "hit0",
        true, 
        true,
    );
    pub const SpinnerPoint: HitJudgment = HitJudgment::new(
        "spinner_point",
        "",
        1.0,
        AffectsCombo::Ignore,
        1000,
        ComboMultiplier::None,
        Color::new(0.0, 0.0, 0.0, 0.0),
        "",
        false, 
        false,
    );

    pub const fn variants() -> &'static [HitJudgment] {
        &[
            Self::X300,
            Self::X100,
            Self::X50,
            Self::Miss,

            Self::SliderDot,
            Self::SliderDotMiss,

            Self::SliderEnd,
            Self::SliderEndMiss,

            Self::SpinnerPoint, 
            Self::SpinnerMiss,
        ]
    }

}


// #[derive(Copy, Clone, Debug, Eq, PartialEq)]
// pub enum OsuHitJudgments {
//     X300,
//     X100,
//     X50,
//     Miss,

//     SliderDot,
//     SliderDotMiss,

//     SliderEnd,
//     SliderEndMiss,

//     SpinnerPoint,
//     SpinnerMiss,
// }

// use OsuHitJudgments::*;
// impl OsuHitJudgments {
//     pub fn should_draw(&self) -> bool {
//         match self {
//             X300
//             | X100
//             | X50
//             | Miss
//             | SliderEndMiss
//             | SliderDotMiss
//             | SpinnerMiss
//             => true,

//             _ => false
//         }
//     }
// }

// impl HitJudgments for OsuHitJudgments {
//     fn variants(&self) -> Vec<Box<dyn HitJudgments>> {
//         vec![
//             Box::new(X300),
//             Box::new(X100),
//             Box::new(X50),
//             Box::new(Miss),
//         ]
//     }


//     fn as_str_internal(&self) -> &'static str {
//         match self {
//             X300 => "x300",
//             X100 => "x100",
//             X50  => "x50",
//             Miss => "xmiss",
//             SliderDot => "slider_dot",
//             SliderDotMiss => "slider_dot_miss",
            

//             SliderEnd => "slider_end",
//             SliderEndMiss => "xmiss", // alias to miss, so it counts as misses when added

//             SpinnerMiss => "xmiss",
//             SpinnerPoint => "spinner_point"
//         }
//     }

//     fn as_str_display(&self) -> &'static str {
//         match self {
//             X300 => "x300",
//             X100 => "x100",
//             X50  => "x50",
//             Miss => "Miss",
//             _ => "",
//         }
//     }


//     fn get_health(&self) -> f32 {
//         match self {
//             X300 => 3.0,
//             X100 => 1.0,
//             X50  => -2.0,
//             Miss => -10.0,

//             SliderDot => 1.0,
//             SliderDotMiss => -2.0,

//             SliderEnd => 1.0,
//             SliderEndMiss => -5.0,

//             SpinnerMiss => -5.0,
//             SpinnerPoint => 1.0,
//         }
//     }

//     fn affects_combo(&self) -> AffectsCombo {
//         match self {
//             Miss | SliderDotMiss | SliderEndMiss | SpinnerMiss => AffectsCombo::Reset,
//             SpinnerPoint => AffectsCombo::Ignore,
            
//             _ => AffectsCombo::Increment,
//         }
//     }

//     fn get_score(&self, combo: u16) -> i32 {
//         // slider dot and spinner point not affected by combo
//         if let SliderDot = self { return 100; }
//         if let SpinnerPoint = self { return 1000; }

//         let combo = (combo.clamp(0, 80) / 10).max(1) as i32;
//         combo * match self {
//             X300 => 300,
//             X100 => 100,
//             X50  => 50,
//             _ => 0
//         }
//     }

//     fn color(&self) -> Color {
//         match self {
//             X300 => Color::new(0.0, 0.7647, 1.0, 1.0),
//             X100 => Color::new(0.3411, 0.8901, 0.0745, 1.0),
//             X50  => Color::new(0.8549, 0.6823, 0.2745, 1.0),
//             Miss | SliderEndMiss | SliderDotMiss => Color::new(0.9, 0.05, 0.05, 1.0),
//             _ => Color::default(),
//         }
//     }

//     fn fails_perfect(&self) -> bool {
//         match self {
//             X300 | SliderDot | SliderEnd => false,
//             _ => true
//         }
//     }

//     fn fails_sudden_death(&self) -> bool {
//         match self {
//             X50 | Miss | SliderEndMiss | SliderDotMiss | SpinnerMiss => true,
//             _ => false
//         }
//     }

    // fn tex_name(&self) -> &'static str {
    //     match self {
    //         X300 => "hit300",
    //         X100 => "hit100",
    //         X50 => "hit50",
    //         Miss => "hit0",
            
    //         SliderDotMiss 
    //         | SpinnerMiss
    //         | SliderEndMiss => "hit0",

    //         SliderDot 
    //         |  SliderEnd => "",
            
    //         _ => ""
    //     }
    // }

// }

//         match self {
//             X300
//             | X100
//             | X50
//             | Miss
//             | SliderEndMiss
//             | SliderDotMiss
//             | SpinnerMiss
//             => true,

//             _ => false
//         }
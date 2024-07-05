use crate::prelude::*;


const COMBO_MULTIPLIER: ComboMultiplier = ComboMultiplier::Linear { 
    combo: 10, 
    multiplier: 1.1, 
    combo_cap: Some(80)
};

pub struct ManiaHitJudgments;
#[allow(non_upper_case_globals)]
impl ManiaHitJudgments {
    pub const Marvelous: HitJudgment = HitJudgment::new(
        "xgeki",
        "Marvelous",
        3.0,
        AffectsCombo::Increment,
        330,
        COMBO_MULTIPLIER,
        Color::new(0.9800, 0.9960, 0.7098, 1.0),
        "",
        false,
        false,
    );

    pub const Perfect: HitJudgment = HitJudgment::new(
        "x300",
        "Perfect",
        2.0,
        AffectsCombo::Increment,
        300,
        COMBO_MULTIPLIER,
        Color::new(0.9608, 0.8706, 0.4087, 1.0),
        "",
        false,
        false,
    );

    pub const Great: HitJudgment = HitJudgment::new(
        "xkatu",
        "Great",
        1.0,
        AffectsCombo::Increment,
        200,
        COMBO_MULTIPLIER,
        Color::new(0.3372, 0.9922, 0.4314, 1.0),
        "",
        true,
        false,
    );

    pub const Good: HitJudgment = HitJudgment::new(
        "x100",
        "Good",
        -2.0,
        AffectsCombo::Increment,
        100,
        COMBO_MULTIPLIER,
        Color::new(0.0, 0.8157, 0.9961, 1.0),
        "",
        true,
        false,
    );  

    pub const Okay: HitJudgment = HitJudgment::new(
        "x50",
        "Okay",
        -5.0,
        AffectsCombo::Increment,
        50,
        COMBO_MULTIPLIER,
        Color::new(0.7451, 0.3725, 0.7098, 1.0),
        "",
        true, 
        true
    );

    pub const Miss: HitJudgment = HitJudgment::new(
        "xmiss",
        "Miss",
        -10.0,
        AffectsCombo::Reset,
        0,
        ComboMultiplier::None,
        Color::new(0.9725, 0.3921, 0.3647, 1.0),
        "",
        true,
        true,
    );

    pub const fn variants() -> &'static [HitJudgment] {
        &[
            Self::Marvelous,
            Self::Perfect,
            Self::Great,
            Self::Good,
            Self::Okay,
            Self::Miss,
        ]
    }
}



// #[derive(Copy, Clone, Debug)]
// pub enum ManiaHitJudgments {
//     Marvelous,
//     Perfect,
//     Great,
//     Good,
//     Okay,
//     Miss,
// }

// impl HitJudgments for ManiaHitJudgments {
//     fn variants(&self) -> Vec<Box<dyn HitJudgments>> {
//         vec![
//             Box::new(Self::Marvelous),
//             Box::new(Self::Perfect),
//             Box::new(Self::Great),
//             Box::new(Self::Good),
//             Box::new(Self::Okay),
//             Box::new(Self::Miss),
//         ]
//     }

//     fn as_str_internal(&self) -> &'static str {
//         match self {
//            Self::Marvelous => "xgeki",
//            Self::Perfect   => "x300",
//            Self::Great     => "xkatu",
//            Self::Good      => "x100",
//            Self::Okay      => "x50",
//            Self::Miss      => "xmiss",
//         }
//     }

//     fn as_str_display(&self) -> &'static str {
//         match self {
//             Self::Marvelous => "Marvelous",
//             Self::Perfect   => "Perfect",
//             Self::Great     => "Great",
//             Self::Good      => "Good",
//             Self::Okay      => "Okay",
//             Self::Miss      => "Miss",
//         }
//     }

//     fn get_health(&self) -> f32 {
//         match self {
//             Self::Marvelous => 3.0,
//             Self::Perfect   => 2.0,
//             Self::Great     => 1.0,
//             Self::Good      => -2.0,
//             Self::Okay      => -5.0,
//             Self::Miss      => -10.0,
//         }
//     }

//     fn affects_combo(&self) -> AffectsCombo {
//         match self {
//             Self::Miss => AffectsCombo::Reset,
//             _ => AffectsCombo::Increment,
//         }
//     }

//     fn get_score(&self, combo: u16) -> i32 {
//         let combo = (combo.clamp(0, 80) / 10).max(1) as i32;

//         combo * match self {
//             Self::Marvelous => 330,
//             Self::Perfect   => 300,
//             Self::Great     => 200,
//             Self::Good      => 100,
//             Self::Okay      => 50,
//             Self::Miss      => 0,
//         }
//     }


//     // stolen from quaver
//     fn color(&self) -> Color {
//         match self {
//             Self::Marvelous => Color::from_rgb_bytes(250, 254, 181),
//             Self::Perfect   => Color::from_rgb_bytes(245, 222, 104),
//             Self::Great     => Color::from_rgb_bytes(86, 253, 110),
//             Self::Good      => Color::from_rgb_bytes(0, 208, 254),
//             Self::Okay      => Color::from_rgb_bytes(190, 95, 181),
//             Self::Miss      => Color::from_rgb_bytes(248, 100, 93),
//         }
//     }

    
//     fn fails_perfect(&self) -> bool {
//         match self {
//             Self::Marvelous | Self::Perfect => false,
//             _ => true
//         }
//     }

//     fn fails_sudden_death(&self) -> bool {
//         match self {
//             Self::Okay | Self::Miss => true,
//             _ => false
//         }
//     }
// }
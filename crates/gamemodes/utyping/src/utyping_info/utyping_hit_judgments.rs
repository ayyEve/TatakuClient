use crate::prelude::*;

const COMBO_MULTIPLIER: ComboMultiplier = ComboMultiplier::Linear { 
    combo: 10, 
    multiplier: 1.1, 
    combo_cap: Some(80)
};

pub struct UTypingHitJudgment;
#[allow(non_upper_case_globals)]
impl UTypingHitJudgment {
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
    pub const X100: HitJudgment = HitJudgment::new(
        "x100,",
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

    pub const fn variants() -> &'static [HitJudgment] {
        &[
            Self::X300,
            Self::X100,
            Self::Miss,
        ]
    }
}



// #[derive(Copy, Clone, Debug, Eq, PartialEq)]
// pub enum UTypingHitJudgment {
//     X300,
//     X100,
//     Miss,
// }

// use UTypingHitJudgment::*;

// impl HitJudgments for UTypingHitJudgment {
//     fn variants(&self) -> Vec<Box<dyn HitJudgments>> {
//         vec![
//             Box::new(X300),
//             Box::new(X100),
//             Box::new(Miss),
//         ]
//     }

//     fn as_str_internal(&self) -> &'static str {
//         match self {
//             X300 => "x300",
//             X100 => "x100",
//             Miss => "xmiss",
//         }
//     }

//     fn as_str_display(&self) -> &'static str {
//         match self {
//             X300 => "x300",
//             X100 => "x100",
//             Miss => "Miss",
//         }
//     }

//     fn get_health(&self) -> f32 {
//         match self {
//             X300 => 3.0,
//             X100 => 1.0,
//             Miss => -10.0,
//         }
//     }

//     fn affects_combo(&self) -> AffectsCombo {
//         match self {
//             Miss => AffectsCombo::Reset,
//             _ => AffectsCombo::Increment,
//         }
//     }

//     fn get_score(&self, combo: u16) -> i32 {
//         let combo = (combo.clamp(0, 80) / 10).max(1) as i32;

//         combo * match self {
//             X300 => 300,
//             X100 => 100,
//             _ => 0,
//         }
//     }


//     // stolen from quaver
//     fn color(&self) -> Color {
//         match self {
//             X300 => Color::new(0.0, 0.7647, 1.0, 1.0),
//             X100 => Color::new(0.3411, 0.8901, 0.0745, 1.0),
//             Miss => Color::new(0.9, 0.05, 0.05, 1.0),
//         }
//     }

//     fn tex_name(&self) -> &'static str {
//         match self {
//             X300 => "taiko-hit300",
//             X100 => "taiko-hit100",
//             Miss => "taiko-hit0",
//         }
//     }

//     /// does this judgment fail a perfect score?
//     fn fails_perfect(&self) -> bool { 
//         match self {
//             X300 => false,
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

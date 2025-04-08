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

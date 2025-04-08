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
        Color::new(0.9, 0.05, 0.05, 1.0),
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

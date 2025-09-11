use crate::prelude::*;
use tataku::Color;
use engine::gameplay::judgments::*;

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
        Color::TRANSPARENT,
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
        Color::TRANSPARENT,
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

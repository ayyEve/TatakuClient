use crate::prelude::*;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TaikoHitJudgments {
    Geki,
    X300,
    Katu,
    X100,
    Miss,

    SliderPoint,
    SpinnerPoint,
}

use TaikoHitJudgments::*;

impl HitJudgments for TaikoHitJudgments {
    fn variants(&self) -> Vec<Box<dyn HitJudgments>> {
        vec![
            Box::new(Geki),
            Box::new(X300),
            Box::new(Katu),
            Box::new(X100),
            Box::new(Miss),
        ]
    }

    fn get_health(&self) -> f32 {
        match self {
            X300 => 3.0,
            X100 => 1.0,
            Miss => -10.0,

            // gekis and katus are just addons to existing judgments, they should not be given extra health
            _ => 0.0
        }
    }

    fn affects_combo(&self) -> AffectsCombo {
        match self {
            Miss => AffectsCombo::Reset,
            
            // gekis and katus are just addons to existing judgments, they should count for combo
            SliderPoint | SpinnerPoint | Geki | Katu  => AffectsCombo::Ignore,
            _ => AffectsCombo::Increment,
        }
    }

    fn get_score(&self, combo: u16) -> i32 {
        let combo = (combo.clamp(0, 80) / 10).max(1) as i32;

        combo * match self {
            X300 | Geki => 300,
            X100 | Katu => 100,

            SliderPoint => 10,
            SpinnerPoint => 100,

            _ => 0,
        }
    }

    fn as_str_internal(&self) -> &'static str {
        match self {
            Geki => "xgeki",
            X300 => "x300",
            Katu => "xkatu",
            X100 => "x100",
            Miss => "xmiss",
            SliderPoint => "slider_point",
            SpinnerPoint => "spinner_point",
        }
    }

    fn as_str_display(&self) -> &'static str {
        match self {
            Geki => "Geki",
            X300 => "x300",
            Katu => "Katu",
            X100 => "x100",
            Miss => "Miss",
            _ => "you shouldnt see this"
        }
    }

    // stolen from quaver
    fn color(&self) -> Color {
        match self {
            X300 | Geki => Color::new(0.0, 0.7647, 1.0, 1.0),
            X100 | Katu => Color::new(0.3411, 0.8901, 0.0745, 1.0),
            Miss => Color::new(0.9, 0.05, 0.05, 1.0),
            _ => Color::default()
        }
    }

    fn tex_name(&self) -> &'static str {
        match self {
            Geki => "taiko-hit300g",
            X300 => "taiko-hit300",
            Katu => "taiko-hit100k",
            X100 => "taiko-hit100",
            Miss => "taiko-hit0",
            _ => ""
        }
    }

}
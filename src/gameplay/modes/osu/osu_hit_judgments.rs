use crate::prelude::*;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum OsuHitJudgments {
    X300,
    X100,
    X50,
    Miss,

    SliderDot,
    SliderDotMiss,

    SliderEnd,
    SliderEndMiss,
}

use OsuHitJudgments::*;
impl OsuHitJudgments {
    pub fn should_draw(&self) -> bool {
        match self {
            X300
            | X100
            | X50
            | Miss
            | SliderEndMiss
            | SliderDotMiss
            => true,

            _ => false
        }
    }
}


impl HitJudgments for OsuHitJudgments {
    fn variants(&self) -> Vec<Box<dyn HitJudgments>> {
        vec![
            Box::new(X300),
            Box::new(X100),
            Box::new(X50),
            Box::new(Miss),
        ]
    }

    fn get_health(&self) -> f32 {
        match self {
            X300 => 3.0,
            X100 => 1.0,
            X50  => -2.0,
            Miss => -10.0,

            SliderDot => 1.0,
            SliderDotMiss => -2.0,

            SliderEnd => 1.0,
            SliderEndMiss => -5.0
        }
    }

    fn affects_combo(&self) -> AffectsCombo {
        match self {
            Miss | SliderDotMiss | SliderEndMiss => AffectsCombo::Reset,
            SliderDot => AffectsCombo::Ignore,
            
            _ => AffectsCombo::Increment,
        }
    }

    fn get_score(&self, combo: u16) -> i32 {
        // slider dot not affected by combo
        if let SliderDot = self { return 100; }

        let combo = (combo.clamp(0, 80) / 10).max(1) as i32;
        combo * match self {
            X300 => 300,
            X100 => 100,
            X50  => 50,
            _ => 0
        }
    }

    fn as_str_internal(&self) -> &'static str {
        match self {
            X300 => "x300",
            X100 => "x100",
            X50  => "x50",
            Miss => "xmiss",
            SliderDot => "slider_dot",
            SliderDotMiss => "slider_dot_miss",

            SliderEnd => "slider_end",
            SliderEndMiss => "xmiss", // alias to miss, so it counts as misses when added
        }
    }

    fn as_str_display(&self) -> &'static str {
        match self {
            X300 => "x300",
            X100 => "x100",
            X50  => "x50",
            Miss => "Miss",
            _ => "",
        }
    }

    fn color(&self) -> Color {
        match self {
            X300 => Color::new(0.0, 0.7647, 1.0, 1.0),
            X100 => Color::new(0.3411, 0.8901, 0.0745, 1.0),
            X50  => Color::new(0.8549, 0.6823, 0.2745, 1.0),
            Miss | SliderEndMiss | SliderDotMiss => Color::new(0.9, 0.05, 0.05, 1.0),
            _ => Color::default(),
        }
    }

    fn fails_perfect(&self) -> bool {
        match self {
            X300 | SliderDot | SliderEnd => false,
            _ => true
        }
    }

    fn fails_sudden_death(&self) -> bool {
        match self {
            X50 | Miss | SliderEndMiss | SliderDotMiss => true,
            _ => false
        }
    }

    fn tex_name(&self) -> &'static str {
        match self {
            X300 => "hit300",
            X100 => "hit100",
            X50 => "hit50",
            Miss => "hit0",
            
            SliderDotMiss 
            | SliderEndMiss => "hit0",

            SliderDot 
            |  SliderEnd => "",
        }
    }

}
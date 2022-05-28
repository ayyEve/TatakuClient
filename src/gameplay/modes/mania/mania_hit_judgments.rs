use crate::prelude::*;

#[derive(Copy, Clone, Debug)]
pub enum ManiaHitJudgments {
    Marvelous,
    Perfect,
    Great,
    Good,
    Okay,
    Miss,
}

impl HitJudgments for ManiaHitJudgments {
    fn variants(&self) -> Vec<Box<dyn HitJudgments>> {
        vec![
            Box::new(Self::Marvelous),
            Box::new(Self::Perfect),
            Box::new(Self::Great),
            Box::new(Self::Good),
            Box::new(Self::Okay),
            Box::new(Self::Miss),
        ]
    }

    fn get_health(&self) -> f32 {
        match self {
            ManiaHitJudgments::Marvelous => 3.0,
            ManiaHitJudgments::Perfect   => 2.0,
            ManiaHitJudgments::Great     => 1.0,
            ManiaHitJudgments::Good      => -2.0,
            ManiaHitJudgments::Okay      => -5.0,
            ManiaHitJudgments::Miss      => -10.0,
        }
    }

    fn affects_combo(&self) -> AffectsCombo {
        match self {
            Self::Miss => AffectsCombo::Reset,
            _ => AffectsCombo::Increment,
        }
    }

    fn get_score(&self, combo: u16) -> i32 {
        let combo = (combo.clamp(0, 80) / 10).max(1) as i32;

        combo * match self {
            ManiaHitJudgments::Marvelous => 330,
            ManiaHitJudgments::Perfect   => 300,
            ManiaHitJudgments::Great     => 200,
            ManiaHitJudgments::Good      => 100,
            ManiaHitJudgments::Okay      => 50,
            ManiaHitJudgments::Miss      => 0,
        }
    }

    fn as_str_internal(&self) -> &'static str {
        match self {
            ManiaHitJudgments::Marvelous => "xgeki",
            ManiaHitJudgments::Perfect   => "x300",
            ManiaHitJudgments::Great     => "xkatu",
            ManiaHitJudgments::Good      => "x100",
            ManiaHitJudgments::Okay      => "x50",
            ManiaHitJudgments::Miss      => "xmiss",
        }
    }

    fn as_str_display(&self) -> &'static str {
        match self {
            ManiaHitJudgments::Marvelous => "Marvelous",
            ManiaHitJudgments::Perfect   => "Perfect",
            ManiaHitJudgments::Great     => "Great",
            ManiaHitJudgments::Good      => "Good",
            ManiaHitJudgments::Okay      => "Okay",
            ManiaHitJudgments::Miss      => "Miss",
        }
    }

    // stolen from quaver
    fn color(&self) -> Color {
        match self {
            ManiaHitJudgments::Marvelous => color_from_byte(250, 254, 181),
            ManiaHitJudgments::Perfect   => color_from_byte(245, 222, 104),
            ManiaHitJudgments::Great     => color_from_byte(86, 253, 110),
            ManiaHitJudgments::Good      => color_from_byte(0, 208, 254),
            ManiaHitJudgments::Okay      => color_from_byte(190, 95, 181),
            ManiaHitJudgments::Miss      => color_from_byte(248, 100, 93),
        }
    }
}
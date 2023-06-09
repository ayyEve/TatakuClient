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
            Self::Marvelous => 3.0,
            Self::Perfect   => 2.0,
            Self::Great     => 1.0,
            Self::Good      => -2.0,
            Self::Okay      => -5.0,
            Self::Miss      => -10.0,
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
            Self::Marvelous => 330,
            Self::Perfect   => 300,
            Self::Great     => 200,
            Self::Good      => 100,
            Self::Okay      => 50,
            Self::Miss      => 0,
        }
    }

    fn as_str_internal(&self) -> &'static str {
        match self {
           Self::Marvelous => "xgeki",
           Self::Perfect   => "x300",
           Self::Great     => "xkatu",
           Self::Good      => "x100",
           Self::Okay      => "x50",
           Self::Miss      => "xmiss",
        }
    }

    fn as_str_display(&self) -> &'static str {
        match self {
            Self::Marvelous => "Marvelous",
            Self::Perfect   => "Perfect",
            Self::Great     => "Great",
            Self::Good      => "Good",
            Self::Okay      => "Okay",
            Self::Miss      => "Miss",
        }
    }

    // stolen from quaver
    fn color(&self) -> Color {
        match self {
            Self::Marvelous => Color::from_rgb_bytes(250, 254, 181),
            Self::Perfect   => Color::from_rgb_bytes(245, 222, 104),
            Self::Great     => Color::from_rgb_bytes(86, 253, 110),
            Self::Good      => Color::from_rgb_bytes(0, 208, 254),
            Self::Okay      => Color::from_rgb_bytes(190, 95, 181),
            Self::Miss      => Color::from_rgb_bytes(248, 100, 93),
        }
    }

    
    fn fails_perfect(&self) -> bool {
        match self {
            Self::Marvelous | Self::Perfect => false,
            _ => true
        }
    }

    fn fails_sudden_death(&self) -> bool {
        match self {
            Self::Okay | Self::Miss => true,
            _ => false
        }
    }
}
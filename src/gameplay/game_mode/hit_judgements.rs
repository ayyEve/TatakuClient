
use crate::prelude::*;

pub trait HitJudgments: Send + Sync {
    /// list of all judgments (in display order)
    fn variants(&self) -> Vec<Box<dyn HitJudgments>>;

    /// how much health should be gained/lost for this judgment
    fn get_health(&self) -> f32;

    /// how does this judgment affect the combo
    fn affects_combo(&self) -> AffectsCombo;

    /// how much score is this judgment worth (at the combo provided)
    fn get_score(&self, combo: u16) -> i32;

    /// internal str for this judgment
    fn as_str_internal(&self) -> &'static str;

    /// what does this judgment look like when displayed?
    fn as_str_display(&self) -> &'static str;

    /// what color is this judgment?
    fn color(&self) -> Color;

    /// what is the texture name for this judgment?
    fn tex_name(&self) -> &'static str { "" }
}


pub enum AffectsCombo {
    /// add one to the combo
    Increment,
    /// do nothing to the combo
    Ignore,
    /// reset the combo
    Reset
}


pub enum DefaultHitJudgments {
    None
}
impl HitJudgments for DefaultHitJudgments {
    fn variants(&self) -> Vec<Box<dyn HitJudgments>> { vec![] }
    fn get_health(&self) -> f32 { 0.0 }
    fn affects_combo(&self) -> AffectsCombo { AffectsCombo::Ignore }
    fn get_score(&self, _combo: u16) -> i32 { 0 }
    fn as_str_internal(&self) -> &'static str { "" }
    fn as_str_display(&self) -> &'static str { "" }

    fn color(&self) -> Color { Color::new(0.0, 0.0, 0.0, 0.0) }
}

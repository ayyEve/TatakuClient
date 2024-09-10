use crate::prelude::*;

// pub trait HitJudgments: Send + Sync {
//     /// list of all judgments (in display order)
//     fn variants(&self) -> Vec<Box<dyn HitJudgments>>;

//     /// how much health should be gained/lost for this judgment
//     fn get_health(&self) -> f32;

//     /// how does this judgment affect the combo
//     fn affects_combo(&self) -> AffectsCombo;

//     /// how much score is this judgment worth (at the combo provided)
//     fn get_score(&self, combo: u16) -> i32;

//     /// internal str for this judgment
//     fn as_str_internal(&self) -> &'static str;

//     /// what does this judgment look like when displayed?
//     fn as_str_display(&self) -> &'static str;

//     /// what color is this judgment?
//     fn color(&self) -> Color;

//     /// what is the texture name for this judgment?
//     fn tex_name(&self) -> &'static str { "" }

//     /// does this judgment fail a perfect score?
//     fn fails_perfect(&self) -> bool { false }

//     /// does this judgment fail a sudden death score?
//     fn fails_sudden_death(&self) -> bool { false }
// }


// pub enum DefaultHitJudgments {
//     None
// }
// impl HitJudgments for DefaultHitJudgments {
//     fn variants(&self) -> Vec<Box<dyn HitJudgments>> { vec![] }
//     fn get_health(&self) -> f32 { 0.0 }
//     fn affects_combo(&self) -> AffectsCombo { AffectsCombo::Ignore }
//     fn get_score(&self, _combo: u16) -> i32 { 0 }
//     fn as_str_internal(&self) -> &'static str { "" }
//     fn as_str_display(&self) -> &'static str { "" }

//     fn color(&self) -> Color { Color::new(0.0, 0.0, 0.0, 0.0) }
// }





#[derive(Copy, Clone, Debug, Default)]
pub struct HitJudgment {
    /// internal str for this judgment
    pub id: &'static str,

    // does this alias as another id?
    pub alias_id: Option<&'static str>,

    /// what does this judgment look like when displayed?
    pub display_name: &'static str,

    /// how much health should be gained/lost for this judgment
    pub health: f32,

    /// how does this judgment affect the combo
    pub affects_combo: AffectsCombo,

    // /// how much score is this judgment worth (at the combo provided)
    // get_score(&self, combo: u16) -> i32;

    /// how much is this worth at a base value
    pub base_score_value: i32,

    /// what are the combo steps for this judgment
    pub combo_multiplier: ComboMultiplier,

    /// what color is this judgment?
    pub color: Color,

    /// what is the texture name for this judgment?
    pub tex_name: &'static str,

    /// does this judgment fail a perfect score?
    pub fails_perfect: bool,

    /// does this judgment fail a sudden death score?
    pub fails_sudden_death: bool,
}
impl HitJudgment {
    pub const fn new(
        internal_id: &'static str,
        display_name: &'static str,
        health: f32,
        affects_combo: AffectsCombo,
        base_score_value: i32,
        combo_multipliers: ComboMultiplier,
        color: Color,
        tex_name: &'static str,
        fails_perfect: bool,
        fails_sudden_death: bool,
    ) -> Self {
        Self {
            id: internal_id,
            alias_id: None,
            display_name,
            health,
            affects_combo,
            base_score_value,
            combo_multiplier: combo_multipliers,
            color,
            tex_name,
            fails_perfect,
            fails_sudden_death,
        }
    }
}

impl PartialEq for HitJudgment {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for HitJudgment {}
impl std::hash::Hash for HitJudgment {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl AsRef<str> for HitJudgment {
    fn as_ref(&self) -> &str { self.id }
}


#[derive(Copy, Clone, Debug, Default)]
pub enum AffectsCombo {
    /// add one to the combo
    Increment,

    /// do nothing to the combo
    #[default]
    Ignore,

    /// reset the combo
    Reset
}

#[derive(Copy, Clone, Debug, Default)]
pub enum ComboMultiplier {
    /// There is no extra combo modifier
    #[default]
    None,

    /// Always multiply the score by a custom value
    Custom(f32),

    /// Every X, increase the score multiplier by Y
    /// Think of this as a graph with x being the combo count and y being the multiplier
    Linear {
        combo: u16,
        multiplier: f32,
        /// after this combo, stop increasing the multiplier
        combo_cap: Option<u16>,
    }
}


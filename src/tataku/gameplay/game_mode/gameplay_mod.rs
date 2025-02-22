
pub trait GameplayMod: Send + Sync {
    fn name(&self) -> &'static str;
    fn short_name(&self) -> &'static str;
    fn display_name(&self) -> &'static str;

    fn description(&self) -> &'static str { "No description provided :c" }
    fn texture_name(&self) -> &'static str { self.name() }
    
    fn score_multiplier(&self) -> f32 { 1.0 }
    fn removes(&self) -> &'static [&'static str] { &[] }
}

pub struct GameplayModGroup {
    pub name: String,
    pub mods: Vec<Box<dyn GameplayMod>>
}
impl GameplayModGroup {
    pub fn new(name: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            mods: Vec::new()
        }
    }
    
    pub fn with_mod<G: GameplayMod + 'static>(mut self, m: G) -> Self {
        self.mods.push(Box::new(m));
        self
    }
}



// default mods

pub struct Autoplay;
impl GameplayMod for Autoplay {
    fn name(&self) -> &'static str { "autoplay" }
    fn short_name(&self) -> &'static str {"AT" }
    fn display_name(&self) -> &'static str { "Autoplay" }

    fn description(&self) -> &'static str { "Let the game play for you!" }
    fn score_multiplier(&self) -> f32 { 0.0 }
}

pub struct NoFail;
impl GameplayMod for NoFail {
    fn name(&self) -> &'static str { "no_fail" }
    fn short_name(&self) -> &'static str { "NF" }
    fn display_name(&self) -> &'static str { "No Fail" }
    fn description(&self) -> &'static str { "Even if you fail, you don't!" }

    fn removes(&self) -> &'static [&'static str] {
        &[
            "sudden_death",
            "perfect"
        ]
    }
}
pub struct SuddenDeath;
impl GameplayMod for SuddenDeath {
    fn name(&self) -> &'static str { "sudden_death" }
    fn short_name(&self) -> &'static str { "SD" }
    fn display_name(&self) -> &'static str { "Sudden Death" }
    fn description(&self) -> &'static str { "Insta-fail if you miss" }

    fn removes(&self) -> &'static [&'static str] {
        &[
            "no_fail",
            "perfect"
        ]
    }
}
pub struct Perfect;
impl GameplayMod for Perfect {
    fn name(&self) -> &'static str { "perfect" }
    fn short_name(&self) -> &'static str { "PF" }
    fn display_name(&self) -> &'static str { "Perfect" }
    fn description(&self) -> &'static str { "Insta-fail if you do any less than perfect" }

    fn removes(&self) -> &'static [&'static str] {
        &[
            "no_fail",
            "sudden_death"
        ]
    }
}


pub fn default_mod_groups() -> Vec<GameplayModGroup> {
    vec![
        GameplayModGroup::new("Difficulty")
            .with_mod(NoFail)
            .with_mod(SuddenDeath)
            .with_mod(Perfect)
        ,
        
        GameplayModGroup::new("Fun")
            .with_mod(Autoplay)
        ,
    ]
}
use crate::prelude::*;

/// A reflect-friendly version of GameplayModGroup
#[derive(Reflect)]
#[derive(Clone, Debug)]
#[reflect(display="display")]
pub struct ReflectModGroup {
    pub name: String,
    pub mods: Vec<ReflectMod>,
}
impl ReflectModGroup {
    pub fn from_group(
        group: &GameplayModGroup,
        mods: &ModManager,
    ) -> Self {
        Self {
            name: group.name.clone(),
            mods: group.mods.iter()
                .map(|inner| ReflectMod::from_mod(*inner, mods))
                .collect(),
        }
    }

    pub fn update(&mut self, mods: &ModManager) {
        for m in self.mods.iter_mut() {
            m.enabled = mods.has_mod(&m.id);
        }
    }
}
impl Display for ReflectModGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)
    }
}

/// A reflect-friendly version of GameplayMod
/// specifically with an enabled field
/// TODO: move back to &'static strs once https://gitlab.ayyeve.dev/tataku/tataku-common/-/issues/3 is fixed
#[derive(Reflect)]
#[derive(Clone, Debug)]
#[reflect(display="display")]
pub struct ReflectMod {
    pub enabled: bool,
    pub id: String,
    pub short_name: String,
    pub display_name: String,
    pub description: String,
    pub adjusts_difficulty: bool,
    pub score_multiplier: f32,
    pub removes: Vec<String>
}
impl ReflectMod {
    fn from_mod(
        inner: GameplayMod,
        mods: &ModManager
    ) -> Self {
        Self {
            id: inner.name.to_owned(),
            short_name: inner.short_name.to_owned(),
            display_name: inner.display_name.to_owned(),
            description: inner.description.to_owned(),
            adjusts_difficulty: inner.adjusts_difficulty,
            score_multiplier: inner.score_multiplier,
            removes: inner.removes.iter().map(|i| (*i).to_string()).collect(),

            enabled: mods.has_mod(inner),
        }
    }
}

impl Display for ReflectMod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.display_name.fmt(f)
    }
}

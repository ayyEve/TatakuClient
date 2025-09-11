use crate::*;
use common::reflect::*;
use gameplay::{
    info::GamemodeInfo,
    widgets::{
        GameplayWidget,
        GameplayWidgetLayout,
    },
};

#[derive(Clone)]
#[derive(Reflect, Debug2)]
pub struct GameplayWidgetBuilder {
    pub name: &'static str,
    
    #[reflect(skip)]
    pub default_layout: GameplayWidgetLayout,

    #[debug(skip)]
    #[reflect(skip)]
    pub build: fn(&GamemodeInfo, &Arc<settings::common_gameplay::CommonGameplaySettings>) -> Box<dyn GameplayWidget>,
}
impl GameplayWidgetBuilder {
    pub fn build(
        &self, 
        info: &GamemodeInfo, 
        settings: &Arc<settings::common_gameplay::CommonGameplaySettings>
    ) -> Box<dyn GameplayWidget> {
        (self.build)(info, settings)
    }
}

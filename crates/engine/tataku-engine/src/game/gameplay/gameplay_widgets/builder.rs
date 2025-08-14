use crate::prelude::*;

#[derive(Clone)]
#[derive(Reflect, Debug2)]
pub struct GameplayWidgetBuilder {
    pub name: &'static str,
    
    #[reflect(skip)]
    pub default_layout: GameplayWidgetLayout,

    #[debug(skip)]
    #[reflect(skip)]
    pub build: fn(&GamemodeInfo, &Arc<CommonGameplaySettings>) -> Box<dyn GameplayWidget>,
}
impl GameplayWidgetBuilder {
    pub fn build(
        &self, 
        info: &GamemodeInfo, 
        settings: &Arc<CommonGameplaySettings>
    ) -> Box<dyn GameplayWidget> {
        (self.build)(info, settings)
    }
}

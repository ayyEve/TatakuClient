use crate::*;
use tataku::Vector2;

pub trait GameplayWidget: Send + Sync {
    fn display_name(&self) -> &'static str;

    /// the max size of the element (before scaling)
    fn max_size(&self) -> Vector2;
    fn update(&mut self, shell: &mut GameplayWidgetUpdateShell);

    fn draw(
        &mut self, 
        shell: &mut GameplayWidgetDrawShell,
    );
    
    fn reset(&mut self) {}

    fn reload_skin(
        &mut self, 
        _shell: &mut GameplayWidgetReloadSkinShell,
    ) {}
}


pub struct GameplayWidgetUpdateShell<'a> {
    pub manager: &'a mut dyn gameplay::gameplay_manager::GameplayManagerTrait,
    pub font_context: &'a mut ui::widget::TextLayoutContexts,
    pub scale: Vector2,
}

pub struct GameplayWidgetDrawShell<'a> {
    pub transform: tataku::Matrix,
    pub list: &'a mut graphics::RenderableCollection
}

pub struct GameplayWidgetReloadSkinShell<'a> {
    pub source: &'a graphics::TextureSource, 
    pub skin_manager: &'a mut dyn graphics::SkinProvider,
}

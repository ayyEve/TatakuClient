use crate::prelude::*;
use tataku_graphics::prelude::*;
use tataku_ui::prelude::TextLayoutContexts;

pub trait GameplayWidget: Send + Sync {
    fn display_name(&self) -> &'static str;

    /// the max size of the element (before scaling)
    fn max_size(&self) -> Vector2;
    fn update(&mut self, shell: &mut GameplayWidgetUpdateShell);

    #[cfg(feature="graphics")]
    fn draw(
        &mut self, 
        shell: &mut GameplayWidgetDrawShell,
    );
    
    fn reset(&mut self) {}

    #[cfg(feature="graphics")]
    fn reload_skin(
        &mut self, 
        _shell: &mut GameplayWidgetReloadSkinShell,
    ) {}
}


pub struct GameplayWidgetUpdateShell<'a> {
    pub manager: &'a mut dyn GameplayManagerTrait,
    pub font_context: &'a mut TextLayoutContexts,
    pub scale: Vector2,
}

pub struct GameplayWidgetDrawShell<'a> {
    pub pos_offset: Vector2, 
    pub scale: Vector2, 
    pub align: Alignment,
    pub list: &'a mut RenderableCollection
}

pub struct GameplayWidgetReloadSkinShell<'a> {
    pub source: &'a TextureSource, 
    pub skin_manager: &'a mut dyn SkinProvider,
}


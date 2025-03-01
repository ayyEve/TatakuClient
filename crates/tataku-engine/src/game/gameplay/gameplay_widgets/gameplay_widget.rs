use crate::prelude::*;

#[async_trait]
pub trait GameplayWidget: Send + Sync {
    fn display_name(&self) -> &'static str;

    /// the max size of the element (before scaling)
    fn max_size(&self) -> Vector2;
    fn update(&mut self, manager: &mut dyn GameplayManagerTrait);

    #[cfg(feature="graphics")]
    fn draw(
        &mut self, 
        pos_offset: Vector2, 
        scale: Vector2, 
        align: Alignment,
        list: &mut RenderableCollection
    );
    
    fn reset(&mut self) {}

    #[cfg(feature="graphics")]
    async fn reload_skin(
        &mut self, 
        _source: &TextureSource, 
        _skin_manager: &mut dyn SkinProvider
    ) {}
}

use crate::prelude::*;

use engine::graphics::{
    Image,
    SkinUsage,
    SkinProvider,
    TextureSource,
    RenderableCollection,
};

#[derive(Clone)]
pub struct ImageCircle {
    pub circle: Image,
    pub overlay: Image,
}
impl ImageCircle {
    pub fn load_from_skin(
        finisher: bool, 
        source: &TextureSource,
        skin_manager: &mut dyn SkinProvider
    ) -> Option<Self> {
        let hitcircle = if finisher {
            "taikobigcircle"
        } else {
            "taikohitcircle"
        };

        let overlay_name = format!("{hitcircle}overlay");
        let overlay = skin_manager.get_texture(
            Path::new(&overlay_name), 
            source, 
            SkinUsage::Gamemode, 
            false,
        )?;

        let circle = skin_manager.get_texture(
            Path::new(hitcircle), 
            source, 
            SkinUsage::Gamemode, 
            false,
        )?;

        Some(Self {
            circle,
            overlay,
        })
    }

    pub fn draw(&self, transform: tataku::Matrix, list: &mut RenderableCollection) {
        list.push(self.circle.clone().with_transform(transform));
        list.push(self.overlay.clone().with_transform(transform));
    }
}

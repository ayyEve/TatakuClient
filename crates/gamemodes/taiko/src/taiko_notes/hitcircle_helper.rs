use crate::prelude::*;
#[derive(Clone)]
pub struct HitCircleImageHelper {
    color: Color,
    circle: Option<Image>,
    overlay: Image,
}
impl HitCircleImageHelper {
    pub fn new(
        settings: &Arc<TaikoSettings>, 
        hit_type: HitType, 
        finisher: bool, 
        source: &TextureSource,
        skin_manager: &mut dyn SkinProvider
    ) -> Option<Self> {
        let color = match hit_type {
            HitType::Don => settings.don_color.color,
            HitType::Kat => settings.kat_color.color,
        };

        let (radius, hitcircle) = if finisher {
            (settings.note_radius * settings.big_note_multiplier, "taikobigcircle")
        } else {
            (settings.note_radius, "taikohitcircle")
        };

        let scale = Vector2::ONE * (radius * 2.0) / TAIKO_NOTE_TEX_SIZE;
        let overlay = skin_manager.get_texture_then(
            &format!("{hitcircle}overlay"), 
            source, 
            SkinUsage::Gamemode, 
            false, 
            |i| {
                i.pos = Vector2::ZERO;
                i.scale = scale;
            }
        )?;

        let circle = skin_manager.get_texture_then(
            hitcircle, 
            source, 
            SkinUsage::Gamemode, 
            false,
            |i| {
                i.pos = Vector2::ZERO;
                i.scale = scale;
                i.color = color;
            }
        );


        Some(Self {
            color,
            circle,
            overlay,
        })
    }

    pub fn set_pos(&mut self, pos: Vector2) {
        if let Some(circle) = self.circle.as_mut() {
            circle.pos  = pos; 
        }
        self.overlay.pos = pos; 
    }

    pub fn draw(&self, list: &mut RenderableCollection) {
        if let Some(circle) = &self.circle {
            list.push(circle.clone());
        } else {
            list.push(Circle::new(
                self.overlay.pos,
                (self.overlay.size().x / 2.0) * 0.95,
                self.color
            ));
        }

        list.push(self.overlay.clone());
    }

    pub fn update_settings(
        &mut self, 
        settings: Arc<TaikoSettings>, 
        finisher: bool
    ) {
        let radius = if finisher {
            settings.note_radius * settings.big_note_multiplier
        } else {
            settings.note_radius
        };

        let scale = Vector2::ONE * (radius * 2.0) / TAIKO_NOTE_TEX_SIZE;
        if let Some(circle) = self.circle.as_mut() {
            circle.scale = scale;
        }
        self.overlay.scale = scale;
    }
}

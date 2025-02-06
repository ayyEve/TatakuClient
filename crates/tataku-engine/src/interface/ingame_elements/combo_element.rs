// use crate::prelude::*;

// const NUMBER: u32 = 5_000;


// pub struct ComboElement {
//     // combo_size: Vector2,
//     combo_image: Option<SkinnedNumber>,
//     combo_text: Text,

//     combo: u16,
// }
// impl ComboElement {
//     pub fn new() -> Self {
//         Self {
//             combo_image: None,
//             combo_text: Text::new(Vector2::ZERO, 30.0, format!("{NUMBER}x"), Color::WHITE, Font::Main),
//             // combo_size: Text::measure_text_raw(&[Font::Main], 30.0, &format!("{NUMBER}x"), Vector2::ONE, 0.0),
//             combo: 0,
//         }
//     }
// }
// #[async_trait]
// impl InnerUIElement for ComboElement {
//     fn display_name(&self) -> &'static str { "Combo" }

//     fn size(&self) -> Vector2 { 
//         self.combo_image.as_ref()
//             .map(|i| i.measure_text())
//             .unwrap_or_else(|| self.combo_text.measure_text())
//     }

//     fn update(&mut self, manager: &mut GameplayManager) {
//         self.combo = manager.score.score.combo;
//     }

//     #[cfg(feature="graphics")]
//     fn draw(
//         &mut self, 
//         pos_offset: Vector2, 
//         scale: Vector2, 
//         list: &mut RenderableCollection
//     ) {
//         // let mut combo_bounds = Bounds::new(
//         //     pos_offset,
//         //     self.combo_size * scale
//         // );
        
//         if let Some(combo) = &mut self.combo_image {
//             combo.number = self.combo as f64;

//             let mut combo = combo.clone();
//             combo.pos = pos_offset;
//             combo.scale = scale;
//             // combo.center_text(&combo_bounds);
//             list.push(combo);
//         } else {
//             let combo_text = Text::new(
//                 pos_offset,
//                 30.0 * scale.y,
//                 format_number(self.combo),
//                 Color::WHITE,
//                 Font::Main
//             );
//             // combo_text.center_text(&combo_bounds);
//             list.push(combo_text);
//         }
//     }

//     #[cfg(feature="graphics")]
//     async fn reload_skin(&mut self, source: &TextureSource, skin_manager: &mut dyn SkinProvider) {
//         self.combo_image = SkinnedNumber::new(Vector2::ZERO, 0.0, Color::WHITE, "combo", Some('x'), 0, skin_manager, source, SkinUsage::Gamemode).await.ok()
//     }
// }


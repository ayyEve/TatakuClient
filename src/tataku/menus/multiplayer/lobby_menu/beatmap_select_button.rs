use crate::prelude::*;

#[derive(ScrollableGettersSetters)]
pub struct BeatmapSelectButton {
    pos: Vector2,
    size: Vector2,
    hover: bool,
    tag: String,
    ui_scale: Vector2,
    base_size: Vector2,

    helper: CurrentLobbyDataHelper,
}
impl BeatmapSelectButton {
    pub fn new(size: Vector2) -> Self {
        Self {
            pos: Vector2::ZERO,
            size,
            base_size: size,
            hover: false,
            tag: "beatmap_select".to_owned(),
            ui_scale: Vector2::ONE,

            helper: CurrentLobbyDataHelper::new()
        }
    }
}

impl ScrollableItem for BeatmapSelectButton {
    fn ui_scale_changed(&mut self, scale: Vector2) {
        self.ui_scale = scale;
        self.size = self.base_size * scale;
    }
    fn update(&mut self) {
        self.helper.update();
    }

    fn draw(&mut self, pos_offset:Vector2, list: &mut RenderableCollection) {
        // background and border
        let bg = Rectangle::new(self.pos + pos_offset, self.size, Color::GRAY.alpha(0.8), Some(Border::new(if self.hover {Color::RED} else {Color::BLACK}, 2.0))).shape(Shape::Round(4.0));
        list.push(bg);


        let Some(data) = &**self.helper else { return };

        let text = if let Some(a) = &data.current_beatmap {
            a.title.clone()
        } else {
            "None".to_owned()
        };

        list.push(Text::new(self.pos + pos_offset + Vector2::ONE * 5.0, 32.0, text, Color::BLACK, Font::Main));
    }
}

use crate::prelude::*;
const LINE_HEIGHT: f32 = 50.0;
const FONT_SIZE: f32 = 32.0;

#[derive(ScrollableGettersSetters)]
pub struct BeatmapSelectButton {
    pos: Vector2,
    size: Vector2,
    style: Style,
    node: Node,

    hover: bool,
    tag: String,
    ui_scale: Vector2,
    base_size: Vector2,

    lobby: CurrentLobbyDataHelper,
    beatmap: CurrentBeatmapHelper,
    mode: CurrentPlaymodeHelper,
    mods: ModManagerHelper,

    lines: Vec<String>
}
impl BeatmapSelectButton {
    pub fn new(layout_manager: &LayoutManager) -> Self {
        let style = Style {
            size: Size {
                width: Dimension::Percent(0.5),
                height: Dimension::Percent(0.5),
            },
            ..Default::default()
        };


        let (pos, size) = LayoutManager::get_pos_size(&style);
        let node = layout_manager.create_node(&style);

        // let size = Vector2::new(width, LINE_HEIGHT * 4.0);
        Self {
            pos,
            size,
            style,
            node,

            base_size: size,
            hover: false,
            tag: "beatmap_select".to_owned(),
            ui_scale: Vector2::ONE,

            lobby: CurrentLobbyDataHelper::new(),
            beatmap: CurrentBeatmapHelper::new(),
            mode: CurrentPlaymodeHelper::new(),
            mods: ModManagerHelper::new(),
            lines: vec!["No Beatmap Selected".to_owned()]
        }
    }
}

impl ScrollableItem for BeatmapSelectButton {
    fn get_style(&self) -> Style { self.style.clone() }
    fn apply_layout(&mut self, layout: &LayoutManager, parent_pos: Vector2) {
        let layout = layout.get_layout(self.node);
        self.pos = layout.location.into();
        self.pos += parent_pos;
        self.size = layout.size.into();
    }

    fn ui_scale_changed(&mut self, scale: Vector2) {
        self.ui_scale = scale;
        self.size = self.base_size * scale;
    }
    fn update(&mut self) {
        let mut needs_update = false;
        needs_update |= self.lobby.update();
        needs_update |= self.beatmap.update();
        needs_update |= self.mods.update();
        needs_update |= self.mode.update();

        // if anything has changed, update the text
        if needs_update {
            self.lines.clear();
            
            let Some(data) = &**self.lobby else { return };
            let Some(lobby_map) = &data.current_beatmap else { self.lines.push("No Beatmap Selected".to_owned()); return };

            if let Some(beatmap) = self.beatmap.as_ref().as_ref().filter(|b|b.beatmap_hash == lobby_map.hash) {
                self.lines.push(format!("{} - {}", beatmap.artist, beatmap.title));
                self.lines.push(format!("{} by {}", beatmap.version, beatmap.creator));

                // difficulty info
                let mode = beatmap.check_mode_override(self.mode.0.clone());
                if let Some(info) = get_gamemode_info(&mode) { 
                    let diff = get_diff(&beatmap, &mode, &self.mods);
                    let diff_meta = BeatmapMetaWithDiff::new(beatmap.clone(), diff);

                    self.lines.push(info.get_diff_string(&diff_meta, &self.mods));
                }
            } else {
                // beatmap version string
                self.lines.push(lobby_map.title.clone());

                // download prompt
                self.lines.push("Click here to open beatmap download page".to_owned());
            }
        }
    }

    fn draw(&mut self, pos_offset:Vector2, list: &mut RenderableCollection) {
        // background and border
        let bg = Rectangle::new(self.pos + pos_offset, self.size, Color::GRAY.alpha(0.8), Some(Border::new(if self.hover {Color::RED} else {Color::BLACK}, 2.0))).shape(Shape::Round(4.0));
        list.push(bg);

        let mut pos = self.pos + pos_offset + Vector2::ONE * 5.0;
        let y_margin = LINE_HEIGHT;

        for line in self.lines.iter() {
            list.push(Text::new(pos, FONT_SIZE, line, Color::BLACK, Font::Main));
            pos.y += y_margin;
        }
    }
}

use crate::prelude::*;

/// how big a direct download item is
pub const DIRECT_ITEM_SIZE:Vector2 = Vector2::new(500.0, 80.0);

/// how big the item is when in a dialog
const DIALOG_ITEM_SIZE:Vector2 = Vector2::new(300.0, 40.0);


#[derive(Clone, ScrollableGettersSetters)]
pub struct DirectItem {
    pos: Vector2,
    size: Vector2,
    hover: bool,
    selected: bool,
    tag: String,

    pub item: Arc<dyn DirectDownloadable>,
    pub downloading: bool,
    last_progress: f32,
}
impl DirectItem {
    pub fn new(item: Arc<dyn DirectDownloadable>, in_dialog: bool) -> DirectItem {
        let size = if in_dialog {DIALOG_ITEM_SIZE} else {DIRECT_ITEM_SIZE};
        let tag = format!("{}|{}|false", item.filename(), item.audio_preview().unwrap_or_default());

        DirectItem {
            pos: Vector2::ZERO,
            size,
            item,
            tag,

            hover: false,
            selected: false,
            downloading: false,
            last_progress: 0.0
        }
    }
}

impl ScrollableItem for DirectItem {
    fn get_value(&self) -> Box<dyn std::any::Any> { Box::new(self.item.clone()) }
    fn update(&mut self) {
        self.downloading = self.item.is_downloading();

        if self.downloading {
            let progress = self.item.get_download_progress();
            let progress = progress.read();
            self.last_progress = progress.progress();

            if progress.complete() {
                self.tag = format!("{}|{}|true", self.item.filename(), self.item.audio_preview().unwrap_or_default());
            }
        }
        
    }

    fn on_click(&mut self, _pos:Vector2, _button:MouseButton, _mods:KeyModifiers) -> bool {
        if !self.hover { return false }

        // if self.selected && self.hover {self.item.download()}
        if self.selected && !self.downloading {
            // add our item to the direct download queue
            GlobalValueManager::get_mut::<DirectDownloadQueue>().unwrap().push(self.item.clone());
        }

        self.selected = self.hover;
        self.hover
    }

    fn draw(&mut self, pos_offset:Vector2, list: &mut RenderableCollection) {
        if self.downloading {
            // progress fill
            let progress_width = self.size.x * self.last_progress;
            list.push(Rectangle::new(
                self.pos + pos_offset,
                Vector2::new(progress_width, self.size.y),
                Color::CYAN,
                None
            ));

            // remaining fill
            list.push(Rectangle::new(
                self.pos + pos_offset + Vector2::with_x(progress_width),
                Vector2::new(self.size.x - progress_width, self.size.y),
                Color::WHITE,
                None
            ));
            
            // border
            list.push(Rectangle::new(
                self.pos + pos_offset,
                self.size,
                Color::TRANSPARENT_WHITE,
                Some(Border::new(if self.hover {Color::BLUE} else if self.selected {Color::RED} else {Color::BLACK}, 1.5))
            ));
        } else {
            list.push(Rectangle::new(
                self.pos + pos_offset,
                self.size(),
                Color::WHITE,
                Some(Border::new(if self.hover {Color::BLUE} else if self.selected {Color::RED} else {Color::BLACK}, 1.5))
            ));
        }


        list.push(Text::new(
            self.pos + pos_offset + Vector2::new(5.0, 25.0),
            20.0,
            format!("{} - {}", self.item.artist(), self.item.title()),
            Color::BLACK,
            Font::Main
        ));

        list.push(Text::new(
            self.pos + pos_offset + Vector2::new(5.0, 50.0),
            20.0,
            format!("Mapped by {}", self.item.creator()),
            Color::BLACK,
            Font::Main
        ));
    }

}

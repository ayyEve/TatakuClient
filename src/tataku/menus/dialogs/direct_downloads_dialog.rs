use crate::prelude::*;

pub struct DirectDownloadDialog {
    list: ScrollableArea,
    layout_manager: LayoutManager,

    queue_helper: GlobalValue<DirectDownloadQueue>,
    
    bounds: Rectangle,
    pub should_close: bool,
}
impl DirectDownloadDialog {
    pub fn new() -> Self {
        let layout_manager = LayoutManager::new();

        let window_size = WindowSize::get();
        let bounds = Self::get_bounds(window_size.0);
        let mut list = ScrollableArea::new(
            Style {
                size: Size {
                    width: Dimension::Percent(0.2),
                    height: Dimension::Percent(0.33),
                },
                ..Default::default()
            }, 
            ListMode::VerticalList,
            &layout_manager
        );

        let queue = GlobalValueManager::get::<DirectDownloadQueue>().unwrap();
        let style = Style {
            size: LayoutManager::full_width(),
            ..Default::default()
        };
        for i in queue.iter() {
            list.add_item(Box::new(DirectItem::new(style.clone(), &list.layout_manager, i.clone(), true)));
        }

        Self {
            list,
            layout_manager,
            
            bounds,
            queue_helper: GlobalValue::new(),
            should_close: false,
        }
    }

    fn get_bounds(size: Vector2) -> Rectangle {
        let width = size.x / 5.0;
        let height = size.y / 3.0;
        Rectangle::new(
            Vector2::new(size.x - width, height / 2.0),
            Vector2::new(width, height),
            Color::BLACK.alpha(0.7),
            Some(Border::new(
                Color::BLACK, 
                1.5
            ))
        )
    }

}

#[async_trait]
impl Dialog<Game> for DirectDownloadDialog {
    fn should_close(&self) -> bool { self.should_close }
    fn get_bounds(&self) -> Bounds { *self.bounds }
    async fn force_close(&mut self) { self.should_close = true; }

    fn container_size_changed(&mut self, size: Vector2) {
        self.bounds = Self::get_bounds(size);
        self.layout_manager.apply_layout(size);
        self.list.apply_layout(&self.layout_manager, Vector2::ZERO);

        // self.list.set_pos(self.bounds.pos);
        // self.list.set_size(self.bounds.size);
    }

    async fn on_mouse_move(&mut self, pos:Vector2, _g:&mut Game) {
        self.list.on_mouse_move(pos)
    }

    async fn on_mouse_down(&mut self, pos:Vector2, button:MouseButton, mods:&KeyModifiers, _game:&mut Game) -> bool {
        let hovered = self.list.get_hover();
        let Some(tag) = self.list.on_click_tagged(pos, button, *mods) else { return hovered };
        let items = self.list.get_tagged(tag);
        let Some(item) = items.first() else { return hovered };
        let value = item.get_value();

        let Some(downloadable) = value.downcast_ref::<Arc<dyn DirectDownloadable>>() else { 
            warn!("failed direct downloadable downcast");
            return hovered 
        };

        info!("clicked: {}", downloadable.filename());

        true
    }

    async fn update(&mut self, _game:&mut Game) {
        if self.queue_helper.update() {
            self.list.clear();
            let style = Style {
                size: LayoutManager::full_width(),
                ..Default::default()
            };

            for i in self.queue_helper.iter() {
                self.list.add_item(Box::new(DirectItem::new(style.clone(), &self.list.layout_manager, i.clone(), true)));
            }
        }
    }

    async fn draw(&mut self, offset: Vector2, list: &mut RenderableCollection) {
        // background and border
        let mut bounds = self.bounds;
        bounds.pos += offset;
        list.push(bounds);

        // draw items in the list
        self.list.draw(offset, list);
    }
}

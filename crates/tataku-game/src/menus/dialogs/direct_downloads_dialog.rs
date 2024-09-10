use crate::prelude::*;

pub struct DirectDownloadDialog {
    num: usize, 

    list: ScrollableArea,

    queue_helper: GlobalValue<DirectDownloadQueue>,
    
    bounds: Rectangle,
    pub should_close: bool,
}
impl DirectDownloadDialog {
    pub fn new() -> Self {
        let window_size = WindowSize::get();
        let bounds = Self::get_bounds(&window_size);
        let mut list = ScrollableArea::new(bounds.pos, bounds.size, ListMode::VerticalList);

        let queue = GlobalValueManager::get::<DirectDownloadQueue>().unwrap();
        for i in queue.iter() {
            list.add_item(Box::new(DirectItem::new(i.clone(), true)));
        }

        Self {
            num: 0,
            list,
            
            bounds,
            queue_helper: GlobalValue::new(),
            should_close: false,
        }
    }

    fn get_bounds(window_size: &Arc<WindowSize>) -> Rectangle {
        let width = window_size.x / 5.0;
        let height = window_size.y / 3.0;
        Rectangle::new(
            Vector2::new(window_size.x - width, height / 2.0),
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
impl Dialog for DirectDownloadDialog {
    fn name(&self) -> &'static str { "direct_download" }
    fn get_num(&self) -> usize { self.num }
    fn set_num(&mut self, num: usize) { self.num = num }
    fn should_close(&self) -> bool { self.should_close }
    // fn get_bounds(&self) -> Bounds { *self.bounds }
    async fn force_close(&mut self) { self.should_close = true; }

    



    // async fn update(&mut self, _values: &mut ShuntingYardValues) -> Vec<TatakuAction> { self.actions.take() }
    
    async fn handle_message(&mut self, _message: Message, _values: &mut dyn Reflect) {
        // let Some(tag) = message.tag.as_string() else { return }; 
    }
    
    fn view(&self, _values: &mut dyn Reflect) -> IcedElement {
        use iced_elements::*;
        EmptyElement.into_element()
    }

    // async fn window_size_changed(&mut self, window_size: Arc<WindowSize>) {
    //     self.bounds = Self::get_bounds(&window_size);

    //     self.list.set_pos(self.bounds.pos);
    //     self.list.set_size(self.bounds.size);
    // }

    // async fn on_mouse_move(&mut self, pos:Vector2, _g:&mut Game) {
    //     self.list.on_mouse_move(pos)
    // }

    // async fn on_mouse_down(&mut self, pos:Vector2, button:MouseButton, mods:&KeyModifiers, _game:&mut Game) -> bool {
    //     let hovered = self.list.get_hover();
    //     let Some(tag) = self.list.on_click_tagged(pos, button, *mods) else { return hovered };
    //     let items = self.list.get_tagged(tag);
    //     let Some(item) = items.first() else { return hovered };
    //     let value = item.get_value();

    //     let Some(downloadable) = value.downcast_ref::<Arc<dyn DirectDownloadable>>() else { 
    //         warn!("failed direct downloadable downcast");
    //         return hovered 
    //     };

    //     info!("clicked: {}", downloadable.filename());

    //     true
    // }

    // async fn update(&mut self, _game:&mut Game) {
    //     if self.queue_helper.update() {
    //         self.list.clear();
    //         for i in self.queue_helper.iter() {
    //             self.list.add_item(Box::new(DirectItem::new(i.clone(), true)));
    //         }
    //     }
    // }

    // async fn draw(&mut self, offset: Vector2, list: &mut RenderableCollection) {
    //     // background and border
    //     let mut bounds = self.bounds;
    //     bounds.pos += offset;
    //     list.push(bounds);

    //     // draw items in the list
    //     self.list.draw(offset, list);
    // }
}

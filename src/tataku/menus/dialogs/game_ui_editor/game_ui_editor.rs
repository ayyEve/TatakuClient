use crate::prelude::*;
const PAIN:bool = true;

pub struct GameUIEditorDialog {
    pub should_close: bool,
    pub elements: Vec<UIElement>,

    mouse_pos: Vector2,

    /// selected_item_index, original_pos, mouse_start
    mouse_down: Option<(usize, Vector2, Vector2)>,

    window_size: Arc<WindowSize>,

    #[allow(unused)]
    event_sender: Arc<parking_lot::Mutex<MultiFuze<UIElementEvent>>>,
    event_receiver: MultiBomb<UIElementEvent>,

    sidebar: ScrollableArea,

    highlight_name: Option<String>
}
impl GameUIEditorDialog {
    pub fn new(elements: Vec<UIElement>) -> Self {
        let (event_sender, event_receiver) = MultiBomb::new();
        let event_sender = Arc::new(parking_lot::Mutex::new(event_sender));

        let window_size = WindowSize::get();

        let mut sidebar = ScrollableArea::new(Vector2::zero(), Vector2::new(window_size.x/3.0, window_size.y * (2.0/3.0)), true);

        for i in elements.iter() {
            sidebar.add_item(Box::new(UISideBarElement::new(i.element_name.clone(), i.inner.display_name(), event_sender.clone())));
        }

        sidebar.set_pos(Vector2::new(0.0, (window_size.y - sidebar.size().y) / 3.0));
        sidebar.refresh_layout();

        Self {
            should_close: false,
            elements,
            mouse_pos: Vector2::zero(),
            mouse_down: None,

            window_size: WindowSize::get(),

            event_sender,
            event_receiver,
            sidebar,
            highlight_name: None
        }
    }

    pub fn update_elements(&mut self, manager: &mut IngameManager) {
        for i in self.elements.iter_mut() {
            i.update(manager)
        }
    }

    fn find_ele_under_mouse(&mut self) -> Option<(usize, &mut UIElement)> {
        for (i, ele) in self.elements.iter_mut().enumerate() {
            let bounds = ele.get_bounds();

            if bounds.contains(self.mouse_pos) {
                return Some((i, ele))
            }
        }
        None
    }
}

#[async_trait]
impl Dialog<()> for GameUIEditorDialog {
    async fn window_size_changed(&mut self, window_size: Arc<WindowSize>) {
        self.window_size = window_size;
    }

    fn get_bounds(&self) -> Rectangle {
        Rectangle::bounds_only(
            Vector2::zero(), 
            self.window_size.0
        )
    }

    fn should_close(&self) -> bool {
        self.should_close
    }

    async fn on_mouse_move(&mut self, pos:&Vector2, _g:&mut ()) {
        self.mouse_pos = *pos;
        self.sidebar.on_mouse_move(self.mouse_pos);

        if let Some((index, old_pos, mouse_start)) = self.mouse_down {
            let ele = &mut self.elements[index];
            
            let change = old_pos + (*pos - mouse_start);

            ele.pos_offset = change;
        }
    }

    async fn on_mouse_down(&mut self, _pos:&Vector2, button:&MouseButton, mods:&KeyModifiers, _g:&mut ()) -> bool {
        let pos = self.mouse_pos;

        if !self.sidebar.on_click(pos, *button, *mods) {
            if button == &MouseButton::Left {
                if let Some((i, ele)) = self.find_ele_under_mouse() {
                    self.mouse_down = Some((i, ele.pos_offset, pos));
                }
            }
        }


        true
    }

    async fn on_mouse_up(&mut self, _pos:&Vector2, _button:&MouseButton, _mods:&KeyModifiers, _g:&mut ()) -> bool {
        // let pos = self.mouse_pos;

        if let Some((i, _, _)) = self.mouse_down {
            // save pos and scale
            self.elements[i].save().await;
        }

        self.mouse_down = None;

        true
    }

    async fn on_mouse_scroll(&mut self, delta:&f64, _g:&mut ()) -> bool {

        if PAIN {
            let delta = (*delta) / 5.0;

            if let Some((index, _, _)) = self.mouse_down {
                let ele = &mut self.elements[index];
                
                ele.scale += Vector2::one() * delta;
                
                if ele.scale.x.abs() < 0.01 { ele.scale.x = 1.0 }
                if ele.scale.y.abs() < 0.01 { ele.scale.y = 1.0 }
            } else if let Some((_, ele)) = self.find_ele_under_mouse() {
                ele.scale += Vector2::one() * delta;
                
                if ele.scale.x.abs() < 0.01 { ele.scale.x = 1.0 }
                if ele.scale.y.abs() < 0.01 { ele.scale.y = 1.0 }
            }
        }


        true
    }

    async fn on_key_press(&mut self, key:&Key, _mods:&KeyModifiers, _g:&mut ()) -> bool {
        if key == &Key::V {
            if self.mouse_down.is_none() {
                if let Some((_, ele)) = self.find_ele_under_mouse() {
                    reset_element(ele).await;
                }
            }
        }
        
        
        if key == &Key::S {
            if self.mouse_down.is_none() {
                let y = self.sidebar.get_pos().y;

                if self.sidebar.get_pos().x < -self.window_size.x {
                    self.sidebar.set_pos(Vector2::new(0.0, y));
                } else {
                    self.sidebar.set_pos(Vector2::new(-self.window_size.x * 5.0, y));
                }

                self.sidebar.refresh_layout();
            }
        }

        true
    }

    async fn update(&mut self, _g:&mut ()) {
        self.sidebar.update();

        if let Some(UIElementEvent(name, action)) = self.event_receiver.exploded() {
            for i in self.elements.iter_mut() {
                if i.element_name == name {

                    match action {
                        UIEditorAction::ToggleVisible => i.visible = !i.visible,
                        UIEditorAction::Reset => reset_element(i).await,
                        UIEditorAction::Highlight => self.highlight_name = Some(name.clone()),
                        UIEditorAction::UnHighlight => if &Some(name) == &self.highlight_name { self.highlight_name = None },
                    }

                    break;
                }
            }

        }
    }

    async fn draw(&mut self, args:&RenderArgs, depth: &f64, list: &mut RenderableCollection) {
        self.sidebar.draw(*args, Vector2::zero(), *depth-9999999999999999999999.0, list);
        list.push(Rectangle::new(
            Color::BLACK.alpha(0.8),
            -9999999999999999999999.0,
            self.sidebar.get_pos(),
            self.sidebar.size(),
            None
        ));

        for i in self.elements.iter_mut() {
            i.draw(list);

            let mut bounds = i.get_bounds();
            if (!self.sidebar.get_hover() && bounds.contains(self.mouse_pos)) || Some(i.element_name.clone()) == self.highlight_name {
                bounds.depth = -999999999999999999999.0;
                bounds.current_color = Color::PINK.alpha(0.7);
                list.push(bounds);
            }
        }

        if let Some((i, _, _)) = self.mouse_down {
            let mut bounds = self.elements[i].get_bounds();
            bounds.depth = -999999999999999999999.0;
            bounds.current_color = Color::RED;
            list.push(bounds);
        }
        
    }
}

async fn reset_element(ele: &mut UIElement) {
    ele.pos_offset = ele.default_pos;
    ele.scale = Vector2::one();
    ele.clear_save().await;
}


const TEXT_SIZE:f64 = 30.0;

pub struct UISideBarElement {
    pos: Vector2,
    size: Vector2,
    hover: bool,

    element_name: String,
    display_name: String,
    event_sender: Arc<parking_lot::Mutex<MultiFuze<UIElementEvent>>>,
}
impl ScrollableItemGettersSetters for UISideBarElement {
    fn size(&self) -> Vector2 {self.size}
    fn set_size(&mut self, new_size: Vector2) {self.size = new_size}

    fn get_pos(&self) -> Vector2 { self.pos }
    fn set_pos(&mut self, pos:Vector2) { self.pos = pos }
    
    fn get_hover(&self) -> bool {self.hover }
    fn set_hover(&mut self, hover:bool) {
        self.hover = hover;

        if hover {
            self.event_sender.lock().ignite(UIElementEvent(self.element_name.clone(), UIEditorAction::Highlight));
        } else {
            self.event_sender.lock().ignite(UIElementEvent(self.element_name.clone(), UIEditorAction::UnHighlight));
        }
    }
    
    fn get_selectable(&self) -> bool { false }
    fn get_multi_selectable(&self) -> bool { false }
}

impl UISideBarElement {
    fn new(element_name: String, display_name:&str, event_sender: Arc<parking_lot::Mutex<MultiFuze<UIElementEvent>>>) -> Self {
        Self { 
            pos: Vector2::zero(), 
            size: Vector2::new(WindowSize::get().x/3.0, TEXT_SIZE), 
            hover: false, 
            element_name, 
            event_sender, 
            display_name: display_name.to_owned()
        }
    }
}
impl ScrollableItem for UISideBarElement {
    fn draw(&mut self, _args:RenderArgs, pos_offset:Vector2, parent_depth:f64, list: &mut RenderableCollection) {
        let text = Text::new(Color::WHITE, parent_depth, self.pos + pos_offset, TEXT_SIZE as u32, self.display_name.clone(), get_font());
        
        let color = if self.hover {Color::BLUE} else {Color::RED};
        let mut r = Rectangle::bounds_only(self.pos + pos_offset, text.measure_text());
        r.current_color = color;
        list.push(r);

        list.push(text);
    }

    fn on_click(&mut self, _pos:Vector2, _button:MouseButton, _mods:KeyModifiers) -> bool {
        if self.hover {
            self.event_sender.lock().ignite(UIElementEvent(self.element_name.clone(), UIEditorAction::ToggleVisible));
        }
        
        self.hover
    }
}



#[derive(Clone)]
struct UIElementEvent(String, UIEditorAction);


#[allow(unused)]
#[derive(Copy, Clone, PartialEq, Eq)]
enum UIEditorAction {
    ToggleVisible,
    Reset,
    Highlight,
    UnHighlight,
}

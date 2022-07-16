use crate::prelude::*;
const PAIN:bool = true;

pub struct GameUIEditorDialog {
    pub should_close: bool,
    pub elements: Vec<UIElement>,

    mouse_pos: Vector2,

    /// selected_item_index, original_pos, mouse_start
    mouse_down: Option<(usize, Vector2, Vector2)>,

    window_size: Arc<WindowSize>
}
impl GameUIEditorDialog {
    pub fn new(elements: Vec<UIElement>) -> Self {
        Self {
            should_close: false,
            elements,
            mouse_pos: Vector2::zero(),
            mouse_down: None,

            window_size: WindowSize::get(),
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

        if let Some((index, old_pos, mouse_start)) = self.mouse_down {
            let ele = &mut self.elements[index];
            
            let change = old_pos + (*pos - mouse_start);

            ele.pos_offset = change;
        }
    }

    async fn on_mouse_down(&mut self, _pos:&Vector2, button:&MouseButton, _mods:&KeyModifiers, _g:&mut ()) -> bool {
        let pos = self.mouse_pos;

        if button == &MouseButton::Left {
            if let Some((i, ele)) = self.find_ele_under_mouse() {
                self.mouse_down = Some((i, ele.pos_offset, pos));
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

            // TODO: fix scaling
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
                    ele.pos_offset = ele.default_pos;
                    ele.scale = Vector2::one();
                    ele.clear_save().await;
                }
            }
        }

        true
    }


    async fn draw(&mut self, _args:&RenderArgs, _depth: &f64, list: &mut Vec<Box<dyn Renderable>>) {
        for i in self.elements.iter_mut() {
            i.draw(list);

            let mut bounds = i.get_bounds();
            if bounds.contains(self.mouse_pos) {
                bounds.depth = -99999999999999999999999.0;
                bounds.current_color = Color::PINK.alpha(0.7);
                list.push(Box::new(bounds));
            }
        }

        if let Some((i, _, _)) = self.mouse_down {
            let mut bounds = self.elements[i].get_bounds();
            bounds.depth = -99999999999999999999999.0;
            bounds.current_color = Color::RED;
            list.push(Box::new(bounds));
        }
        
    }
}

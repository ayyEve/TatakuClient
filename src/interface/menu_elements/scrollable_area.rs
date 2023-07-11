use crate::prelude::*;

pub struct ScrollableArea {
    pub items: Vec<Box<dyn ScrollableItem>>,

    /// key is index of the original array
    filtered_out_items: HashMap<usize, Box<dyn ScrollableItem>>,

    /// layout helper
    original_positions: Vec<Vector2>,

    pub scroll_pos: f32,
    elements_height: f32,

    /// if list mode, item positions will be modified based on how many items there are (ie, a list)
    list_mode: ListMode,
    /// when in collapsible mode, is the item list visible?
    expanded: bool,
    /// indicates if the header is hovererd when in collapsible mode
    header_hover: bool,

    /// how many pixels should be between items when in list mode?
    item_margin: f32,
    /// how much should a scroll unit be worth?
    /// 8.0 is good for my laptop's touchpad, but on a mouse wheel its nowwhere near enough
    pub scroll_factor: f32,

    /// where to draw a dragger
    pub dragger: DraggerSide,
    /// how wide is the dragger?
    pub dragger_width: f32,
    pub dragger_dragging: bool,

    // cache of where the mouse is, only used for debugging now
    mouse_pos: Vector2,

    
    /// drag_start, confirmed_drag, last_checked, mods_when_clicked
    /// drag_start is where the original click occurred
    /// confirmed_drag is if the drag as passed a certain threshhold. important if the drag returns to below the threshhold
    mouse_down: Option<(Vector2, bool, MouseButton, Vector2, KeyModifiers)>,
    /// allow scrolling the list with a mouse drag. note this will affect things which rely on a click and release, as the click is only called on release
    /// ie, sliders will be affected by this
    pub allow_drag_scrolling: bool,
    pub drag_threshold: f32,

    // scrollable item properties
    pos: Vector2,
    size: Vector2,
    hover: bool,
    tag: String,
    ui_scale: Vector2,
}
impl ScrollableArea {
    pub fn new(pos: Vector2, mut size: Vector2, list_mode: ListMode) -> ScrollableArea {
        let mut expanded = false; 
        let mut elements_height = 0.0;
        let item_margin = 5.0;

        if let ListMode::Collapsible(info) = &list_mode {
            expanded = info.initially_expanded;
            elements_height = info.header_height + info.first_item_margin.unwrap_or(item_margin);

            if info.auto_height {
                size.y = elements_height;
            }
        };


        ScrollableArea {
            items: Vec::new(),
            original_positions: Vec::new(),
            filtered_out_items: HashMap::new(), 

            list_mode,
            expanded,
            header_hover: false,

            scroll_pos: 0.0,
            elements_height,

            dragger: DraggerSide::None,
            dragger_dragging: false,
            pos,
            tag: String::new(),
            size,
            hover: false,
            mouse_pos: Vector2::ONE * -100.0, // just in case lol
            item_margin,
            scroll_factor: 16.0,
            dragger_width: 10.0,

            mouse_down: None,
            allow_drag_scrolling: false,
            drag_threshold: 50.0,

            ui_scale: Vector2::ONE
        }
    }

    pub fn set_item_margin(&mut self, margin: f32) {
        self.item_margin = margin;
        self.refresh_layout();
    }

    /// returns index
    pub fn add_item(&mut self, mut item:Box<dyn ScrollableItem>) {
        // immediately update the ui scale for every item being added
        item.ui_scale_changed(self.ui_scale);
        let margin = match (&self.list_mode, self.items.is_empty()) {
            (ListMode::Collapsible(info), true) => info.first_item_margin.unwrap_or(self.item_margin),
            _ => self.item_margin
        };
        

        if self.list_mode.is_list() {
            let ipos = item.get_pos();
            self.original_positions.push(ipos);
            item.set_pos(self.pos + Vector2::new(ipos.x, self.elements_height));
            self.elements_height += item.size().y + margin * self.ui_scale.y;
        }

        if let ListMode::Collapsible(info) = &self.list_mode {
            if info.auto_height {
                self.size.y = self.elements_height;
            }
        }

        self.items.push(item);
    }
    pub fn clear(&mut self) {
        self.items.clear();
        self.elements_height = 0.0;
        self.scroll_pos = 0.0;
    }
    pub fn get_tagged(&self, tag: String) -> Vec<&Box<dyn ScrollableItem>> {
        let mut list = Vec::new();
        for i in self.items.iter() {
            if let Some(inner_list) = i.get_inner_tagged(&tag) {
                list.extend(inner_list.into_iter())
            } else {
                if i.get_tag() == tag {
                    list.push(i);
                }
            }
        }

        list
    }

    /// completely refresh the positions for all items in the list (only effective when using a list mode other than None)
    pub fn refresh_layout(&mut self) {
        if !self.list_mode.is_list() { return }
        self.elements_height = 0.0;

        if let ListMode::Collapsible(info) = &self.list_mode {
            self.elements_height += info.header_height + info.first_item_margin.unwrap_or(self.item_margin);
        }

        for (i, item) in self.items.iter_mut().enumerate() {
            let ipos = self.original_positions[i];
            item.set_pos(self.pos + Vector2::new(ipos.x, self.elements_height));
            self.elements_height += item.size().y + self.item_margin * self.ui_scale.y;
        }

        if let ListMode::Collapsible(info) = &self.list_mode {
            if info.auto_height {
                self.size.y = self.elements_height;
            }
        }
    }

    pub fn get_elements_height(&self) -> f32 {
        self.elements_height
    }


    /// scroll to the first selected object, or to the top if no object is selected
    pub fn scroll_to_selection(&mut self) {
        if self.get_selected_index().is_none() {
            self.scroll_pos = 0.0;
        } else {
            let mut y = 0.0;
            for (n, i) in self.items.iter().enumerate() {
                if i.get_selected() { break }
                let margin = match (&self.list_mode, n==0) {
                    (ListMode::Collapsible(info), true) => info.first_item_margin.unwrap_or(self.item_margin),
                    _ => self.item_margin
                };

                y = i.get_pos().y - margin * self.ui_scale.y * 2.0;
            }
            self.scroll_pos = -y;
        }
    }


    /// get the index of the **first** selected item
    pub fn get_selected_index(&self) -> Option<usize> {
        let mut selected_index = None;
        for (n, i) in self.items.iter().enumerate() {
            if i.get_selected() {
                selected_index = Some(n);
                break;
            }
        }

        // make sure we have the index before continuing
        if selected_index >= Some(self.items.len()) {return None}
        selected_index
    }

    
    /// returns the tag of the newly selected item
    pub fn set_selected_by_index(&mut self, index:usize) -> Option<String> {
        if index > self.items.len() { return None }

        // select item and deselect everything else
        for (n, i) in self.items.iter_mut().enumerate() {
            i.set_selected(n == index)
        }

        // refresh the layout
        self.refresh_layout();

        // return tag
        self.items.get_mut(index).map(|i|i.get_tag())
    }

    /// select the next selectable item
    /// returns the tag of the newly selected item
    pub fn select_next_item(&mut self) -> Option<String> {
        // find the selected item's index
        let selected_index = match self.get_selected_index() {
            Some(i) => i,
            // index out of bounds, or no item selected
            None => return None
        };


        let mut next_index = selected_index + 1;
        if next_index == self.items.len() {next_index = 0}

        self.set_selected_by_index(next_index)
    }
    
    /// select the previous selectable item
    /// returns the tag of the newly selected item
    pub fn select_previous_item(&mut self) -> Option<String> {
        // find the selected item's index
        let selected_index = match self.get_selected_index() {
            Some(i) => i,
            // index out of bounds, or no item selected
            None => return None
        };

        
        // if this is first item, loop back to last item
        let next_index = if selected_index == 0 {
            self.items.len() - 1
        } else {
            selected_index - 1
        };
        
        self.set_selected_by_index(next_index)
    }

    fn scroll_to_y(&mut self, y: f32) {
        let y = y.clamp(self.pos.y, self.pos.y + self.size.y);
        self.scroll_pos = -(y - self.pos.y) / self.size.y * self.elements_height;
    }

    fn on_click_real(&mut self, pos:Vector2, button:MouseButton, mods:KeyModifiers) -> Option<String> {
        if !(self.hover || self.header_hover) { return None }

        if self.list_mode.is_collapsible() {
            // check that the header was clicked, if it was, toggle expansion
            if self.header_hover {
                self.expanded = !self.expanded;
                return None;
            }

            // if the header wasnt clicked, and we arent expanded, return false since we werent clicked
            if !self.expanded { return None; }
        }


        // modify pos here
        let pos = pos - Vector2::new(0.0, self.scroll_pos);
        let mut clicked_item = None;

        // find out if an item is selected
        // TODO! should cache these if perf is bad
        // let mut first_selected_item = None;
        // let mut can_select_multiple = false; // set to true if selected item allows multiple
        // for i in 0..self.items.len() {
        //     let item = &self.items[i];
        //     if item.get_selected() {
        //         first_selected_item = Some(i);
        //         if item.get_multi_selectable() {
        //             can_select_multiple = true;
        //         }

        //         // no need to continue
        //         break;
        //     }
        // }

        let mut needs_refresh = false;
        // do the click loop
        for item in self.items.iter_mut() {
            let pre_size = item.size();
            let clicked = item.on_click(pos, button, mods);
            if clicked { clicked_item = Some(item.get_tag()) }

            if !item.get_selectable() { continue }
            if clicked {
                item.set_selected(true);
            } else {
                if !(mods.ctrl && item.get_multi_selectable())  {
                    item.set_selected(false);
                }
            }

            // check if the element's size changed. if so, refresh out layout so things arent wacked
            if item.size() != pre_size {needs_refresh = true}
        }

        if needs_refresh { self.refresh_layout() }

        // for item in self.items.iter_mut() {
        //     if item.on_click(pos, button, mods) {
        //         // return;
        //         clicked_item = Some(item.get_tag());
        //     }
        // }

        clicked_item
    }


    pub fn apply_filter(&mut self, query: &Vec<String>, do_refresh: bool) {
        // rebuild list
        self.rejoin_items();
        if query.is_empty() { 
            if do_refresh {
                self.refresh_layout();
                self.scroll_to_selection();
            }
            return; 
        }

        for (i, item) in std::mem::take(&mut self.items).into_iter().enumerate() {
            if item.check_filter(query, QueryType::Any) {
                self.items.push(item);
            } else {
                self.filtered_out_items.insert(i, item);
            }
        }

        if do_refresh {
            self.refresh_layout();
            self.scroll_to_selection();
        }
    }

    /// rejoins filtered items
    pub fn rejoin_items(&mut self) {
        let mut items: Vec<(usize, Box<dyn ScrollableItem>)> = std::mem::take(&mut self.filtered_out_items).into_iter().collect();
        items.sort_by(|(a,_),(b,_)|a.cmp(b));

        for (key, item) in items {
            self.items.insert(key, item);
        }
    }
    
}

impl ScrollableItemGettersSetters for ScrollableArea {
    fn size(&self) -> Vector2 {
        if let ListMode::Collapsible(info) = &self.list_mode {
            if !self.expanded {
                return Vector2::new(self.size.x, info.header_height);
            }
        }

        self.size
    }
    fn set_size(&mut self, new_size: Vector2) {
        self.size = new_size;
        self.refresh_layout();
    }

    fn get_tag(&self) -> String { self.tag.clone() }
    fn set_tag(&mut self, tag:&str) { self.tag = tag.to_owned() }

    fn get_pos(&self) -> Vector2 { self.pos }
    fn set_pos(&mut self, pos:Vector2) {
        self.pos = pos;

        if self.list_mode.is_list() {
            self.refresh_layout()
        }
    }

    fn get_selected(&self) -> bool {self.hover}
    fn set_selected(&mut self, selected:bool) {self.hover = selected}

    fn get_hover(&self) -> bool {self.hover}
    fn set_hover(&mut self, hover:bool) {self.hover = hover}
}

impl ScrollableItem for ScrollableArea {
    fn window_size_changed(&mut self, new_window_size: Vector2) {
        for i in self.items.iter_mut() {
            i.window_size_changed(new_window_size)
        }

        self.refresh_layout()
    }
    fn ui_scale_changed(&mut self, scale: Vector2) {
        self.ui_scale = scale;

        for i in self.items.iter_mut() {
            i.ui_scale_changed(scale)
        }
        
        self.refresh_layout()
    }

    // input handlers
    fn on_click(&mut self, pos:Vector2, button:MouseButton, mods:KeyModifiers) -> bool {
        if !(self.hover || self.header_hover) { return false }

        let mut was_dragger = false;

        macro_rules! check_dragger {
            ($x:expr, $height:expr) => {
                // at least the track bar was clicked
                // between left (x) and right (x + dragger_width) of dragger
                if pos.x >= $x && pos.x <= $x + self.dragger_width {
                    was_dragger = true;

                    let dragger_bounds = Rectangle::bounds_only(
                        Vector2::new($x, self.pos.y -(self.scroll_pos / self.elements_height) * self.size.y - $height/2.0),
                        Vector2::new(self.dragger_width, $height)
                    );

                    #[cfg(feature="dragger_debug")] println!("dragger: {:?},{:?}", dragger_bounds.pos, dragger_bounds.size);

                    if dragger_bounds.contains(pos) {
                        self.dragger_dragging = true;
                        #[cfg(feature="dragger_debug")] println!("dragger dragging");
                    } else {
                        // scroll to the position clicked
                        self.scroll_to_y(pos.y);
                        self.dragger_dragging = true;
                        // #[cfg(feature="dragger_debug")] println!("dragger not dragging");
                    }
                }
            }
        }

        // check if dragger was clicked first
        match self.dragger {
            DraggerSide::Left(height, _)  => check_dragger!(self.pos.x -self.dragger_width, height),
            DraggerSide::Right(height, _) => check_dragger!(self.pos.x + self.size.x - self.dragger_width, height),
            
            _ => {}
        }

        if !was_dragger {
            if self.allow_drag_scrolling {
                self.mouse_down = Some((pos, false, button, pos, mods));
            } else {
                self.on_click_real(pos, button, mods);
            }
        }

        true
    }


    /// returns the tag of the item which was clicked
    /// this overrides drag scroll behaviour so be careful!
    fn on_click_tagged(&mut self, pos:Vector2, button:MouseButton, mods:KeyModifiers) -> Option<String> {
        self.on_click_real(pos, button, mods)
    }
    fn on_click_release(&mut self, pos:Vector2, button:MouseButton) {
        if self.allow_drag_scrolling {
            let mut was_hold = false;
            let mut mods = None;

            if let Some((_, was_drag, button, _, click_mods)) = self.mouse_down {
                if was_drag {
                    mods = Some((click_mods, button));
                    was_hold = true;
                }
            }
            self.mouse_down = None;

            if self.dragger_dragging { 
                self.dragger_dragging = false 
            } else {
                if !was_hold {
                    let (mods, button) = mods.unwrap_or((KeyModifiers::default(), MouseButton::Left));
                    if !was_hold {
                        self.on_click_real(pos, button, mods);
                    }
                }

                for item in self.items.iter_mut() {
                    item.on_click_release(pos, button);
                }
            }
        } else {
            if self.dragger_dragging { 
                self.dragger_dragging = false 
            } else {
                for item in self.items.iter_mut() {
                    item.on_click_release(pos, button);
                }
            }
        }
    }
    
    fn on_mouse_move(&mut self, pos:Vector2) {
        self.hover = pos.x > self.pos.x && pos.x < self.pos.x + self.size.x && pos.y > self.pos.y && pos.y < self.pos.y + self.size.y;
        self.mouse_pos = pos;

        if self.dragger_dragging {
            self.scroll_to_y(pos.y);
        } else {
            let mut scroll_pos = 0.0;
            let offset_pos = self.get_pos();
            let comp_size = self.size();
            let items_height = self.get_elements_height();

            if let Some((drag_pos, confirmed_drag, button_pressed, last_checked, _)) = &mut self.mouse_down {
                if *confirmed_drag || (pos.y - drag_pos.y).abs() >= self.drag_threshold {
                    *confirmed_drag |= true;

                    if *button_pressed == MouseButton::Right {
                        let y_percent = ((pos.y - offset_pos.y) / comp_size.y).clamp(0.0, 1.0);

                        self.scroll_pos = -items_height * y_percent;
                    } else {
                        let dist = (pos.y - last_checked.y) / self.scroll_factor;
                        scroll_pos = dist;
                    }
                }

                *last_checked = pos;
            }

            // drag acts like scroll
            if scroll_pos != 0.0 {
                self.on_scroll(scroll_pos);
            }
        }


        if let ListMode::Collapsible(info) = &self.list_mode {
            self.header_hover = Rectangle::bounds_only(self.pos, Vector2::new(self.size.x, info.header_height)).contains(pos);
            self.hover |= self.header_hover;

            if !self.expanded { return }
        }

        if !self.hover { return }

        // if !self.hover {return}
        let mut needs_refresh = false;

        let p = pos-Vector2::new(0.0, self.scroll_pos);
        for item in self.items.iter_mut() {
            let pre_size = item.size();
            item.on_mouse_move(p);

            // check if the element's size changed. if so, refresh out layout so things arent wacked
            if item.size() != pre_size { needs_refresh = true }
        }

        if needs_refresh { self.refresh_layout() }
    }

    fn on_scroll(&mut self, delta:f32) -> bool {
        if self.list_mode.is_collapsible() && !self.expanded { return false; }

        if self.hover {
            for item in self.items.iter_mut() {
                if item.on_scroll(delta) { return true; }
            }

            self.scroll_pos += delta * self.scroll_factor;

            let min = -self.elements_height + self.size.y;
            let max = 0.0;
            self.scroll_pos = if min<=max { self.scroll_pos.clamp(min, max) } else {0.0};

            self.on_mouse_move(self.mouse_pos);
        }

        if let ListMode::Collapsible(info) = &self.list_mode { 
            return !info.auto_height; 
        }

        self.hover
    }
    fn on_key_press(&mut self, key:Key, mods:KeyModifiers) -> bool {
        if self.list_mode.is_collapsible() && !self.expanded { return false; }
        
        for item in self.items.iter_mut() {
            if item.on_key_press(key, mods) { return true; }
        }
        false
    }
    fn on_key_release(&mut self, key:Key) {
        if self.list_mode.is_collapsible() && !self.expanded { return; }
        
        for item in self.items.iter_mut() {
            item.on_key_release(key);
        }
    }
    
    fn on_text(&mut self, text:String) {
        if self.list_mode.is_collapsible() && !self.expanded { return; }
        
        for item in self.items.iter_mut() {
            item.on_text(text.clone());
        }
    }
    
    fn update(&mut self) {
        for item in self.items.iter_mut() {
            item.update();
        }
    }
    fn draw(&mut self, pos_offset:Vector2, list: &mut RenderableCollection) {
        // // helpful for debugging positions
        // if self.hover {
        //     list.push(Rectangle::new(self.pos, self.size, Color::TRANSPARENT_WHITE,  Some(Border::new(if self.hover{Color::RED} else {Color::BLACK}, 2.0))));
        //     // mouse
        //     list.push(Circle::new(self.mouse_pos, 5.0, Color::RED));
        //     // mouse relative to scroll pos
        //     list.push(Circle::new(self.mouse_pos + offset, 5.0, Color::BLUE));
        // }
        let offset = pos_offset + Vector2::with_y(self.scroll_pos);

        // if this is a collapsible menu, draw the header
        if let ListMode::Collapsible(info) = &self.list_mode {
            let mut rect = Rectangle::new(self.pos + offset, Vector2::new(self.size.x, info.header_height), info.header_color, info.header_border);
            rect.shape = info.header_shape;
            if self.header_hover {
                rect.color = info.header_color_hover;
                rect.border = info.header_border_hover;
            }

            let mut txt = Text::new(offset, rect.size.y * 0.8, info.header_text.clone(), info.header_text_color, get_font());
            
            match info.header_text_align {
                HorizontalAlign::Center => txt.center_text(&rect),
                HorizontalAlign::Right => txt.pos.x = rect.pos.x + (rect.size.x - txt.measure_text().x),
                _ => {}
            }

            list.push(rect);
            list.push(txt);


            // dont draw any more items if the list isnt expanded
            if !self.expanded { return }
        }


        // setup a clipping context. 
        // this ensures items arent being drawn outside the bounds of the scrollable
        let pos = self.pos + pos_offset;
        list.push_scissor([ pos.x, pos.y, self.size.x, self.size.y ]);
        for item in self.items.iter_mut() {
            // check if item will even be drawn
            let size = item.size();
            let item_pos = item.get_pos();
            if (item_pos.y + size.y) + offset.y < pos.y || item_pos.y + offset.y > pos.y + self.size.y { continue }

            // should be good, draw it
            item.draw(offset, list);
        }
        list.pop_scissor();


        // draw dragger
        let (x, height) = match self.dragger {
            DraggerSide::Left(height, _) => (self.pos.x - self.dragger_width, height),
            DraggerSide::Right(height, _) => (self.pos.x + self.size.x - self.dragger_width, height),
            _ => return
        };

        // trackbar
        list.push(Rectangle::new(
            Vector2::new(x, self.pos.y),
            Vector2::new(self.dragger_width, self.size.y),
            Color::TRANSPARENT_WHITE,
            Some(Border::new(Color::BLACK, 1.0))
        ));

        // dragger
        list.push(Rectangle::new(
            Vector2::new(x, self.pos.y -(self.scroll_pos / self.elements_height) * self.size.y - height/2.0),
            Vector2::new(self.dragger_width, height),
            Color::BLACK,
            Some(Border::new(Color::BLUE, 1.0))
        ));
        
    }
}



pub enum DraggerSide {
    None,
    /// f32 is dragger height, bool is if its auto resized
    Left(f32, bool),
    /// f32 is dragger height, bool is if its auto resized
    Right(f32, bool)
}



#[derive(Clone, Default, Debug)]
pub enum ListMode {
    #[default]
    None,
    /// order elements in a vertical list
    VerticalList,

    /// items in this list can be hidden or shown by clicking the header
    /// forces a vertical layout
    Collapsible(CollapsibleInfo),
}
impl ListMode {
    fn is_list(&self) -> bool {
        match self {
            Self::None => false,
            _ => true,
        }
    }
    fn is_collapsible(&self) -> bool {
        match self {
            Self::Collapsible(_) => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CollapsibleInfo {
    /// text for the header
    pub header_text: String,
    /// color for header text
    pub header_text_color: Color,
    /// should we align the text to the center
    pub header_text_align: HorizontalAlign,
    
    /// header height
    pub header_height: f32,
    /// color for header background
    pub header_color: Color,
    /// color for header background when hovered
    pub header_color_hover: Color,

    /// border for header
    pub header_border: Option<Border>,
    /// border for header when hovered
    pub header_border_hover: Option<Border>,
    /// header shape
    pub header_shape: Shape,

    /// automatically expand the height to fit all objects in the list
    /// 
    /// you'll want this to be false unless this is a sub-element within another list
    pub auto_height: bool,
    /// margin between the header and the first element in the list
    /// if none, uses the list's item margin
    pub first_item_margin: Option<f32>,


    /// should the list be expanded upon creation? (default false)
    pub initially_expanded: bool,
}
impl Default for CollapsibleInfo {
    fn default() -> Self {
        Self {
            header_text: String::new(),
            header_text_color: Color::BLACK,
            header_text_align: HorizontalAlign::Left,
            header_height: 20.0,
            header_color: Color::GRAY,
            header_color_hover: Color::GRAY,

            header_border: None,
            header_border_hover: None,
            header_shape: Shape::Square,

            auto_height: false,
            first_item_margin: None,
            initially_expanded: true,
        }
    }
}

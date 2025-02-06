use crate::prelude::*;

pub struct UIElement {
    // pub default_pos: Vector2,
    pub element_name: String,
    pub pos_offset: Vector2,
    pub scale: Vector2,
    // pub visible: bool,

    pub layout: UiElementLayout,
    pub default_layout: UiElementLayout,

    pub inner: Box<dyn InnerUIElement>,
}

impl UIElement {
    pub fn update(&mut self, manager: &mut GameplayManager) {
        if !self.layout.visible { return }
        self.inner.update(manager);
    }

    #[cfg(feature="graphics")]
    pub fn draw(&mut self, list: &mut RenderableCollection) {
        if !self.layout.visible { return }
        let align = self.layout.inner_align.unwrap_or(self.layout.align);
        self.inner.draw(self.pos_offset, self.scale, align, list);

        // let bounds = self.get_bounds();
        // list.push(Rectangle::new(
        //     bounds.pos,
        //     bounds.size,
        //     Color::TRANSPARENT_WHITE,
        //     Some(Border::new(Color::LIME, 3.0))
        // ));
    }

    pub fn get_bounds(&self) -> Bounds {
        Bounds::new(
            self.pos_offset, 
            self.inner.max_size() * self.scale
        )
    }

    // pub async fn save(&self) {
    //     Database::save_element_info(self.pos_offset, self.scale, self.layout.visible, &self.element_name).await;
    // }

    // pub async fn clear_save(&self) {
    //     Database::clear_element_info(&self.element_name).await;
    // }

    pub fn reset_element(&mut self) {
        self.inner.reset();
    }

    #[cfg(feature="graphics")]
    pub async fn reload_skin(&mut self, source: &TextureSource, skin_manager: &mut dyn SkinProvider) {
        self.inner.reload_skin(source, skin_manager).await;
    }
}

#[async_trait]
pub trait InnerUIElement: Send + Sync {
    fn display_name(&self) -> &'static str;

    /// the max size of the element (before scaling)
    fn max_size(&self) -> Vector2;
    fn update(&mut self, manager: &mut GameplayManager);

    #[cfg(feature="graphics")]
    fn draw(
        &mut self, 
        pos_offset: Vector2, 
        scale: Vector2, 
        align: Alignment,
        list: &mut RenderableCollection
    );
    
    fn reset(&mut self) {}

    #[cfg(feature="graphics")]
    async fn reload_skin(
        &mut self, 
        _source: &TextureSource, 
        _skin_manager: &mut dyn SkinProvider
    ) {}
}

#[derive(Clone, Debug, Default, PartialEq)]
#[derive(Serialize, Deserialize)]
pub enum UiElementAnchor {
    /// Anchored to the screen 
    /// 
    /// Position can be absolute with this + UIElementAlign::TopLeft
    #[default]
    Screen,

    /// Anchored to the playfield
    /// 
    /// field is size of screen when saved (if element should scale with playfield)
    Playfield {
        saved_size: Option<Vector2>,

        /// Where should this element be relative to the playfield
        relative: UiElementAlign,
    },

    /// Anchored to an element, scaling is determined from the parent element
    Element {
        /// What element to anchor to
        element: String,

        /// Where should this element be relative to the parent
        relative: UiElementAlign,
    },
}
impl UiElementAnchor {
    pub fn element(element: impl ToString, relative: UiElementAlign) -> Self {
        Self::Element {
            element: element.to_string(),
            relative
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
#[derive(Serialize, Deserialize)]
pub struct UiElementLayout {
    /// Where this element is anchored
    pub anchor: UiElementAnchor,

    /// How to align this element
    pub align: Alignment,

    /// How to align the inner element (if needed)
    pub inner_align: Option<Alignment>,

    /// Offset in pixels
    pub offset: Vector2,

    /// What screen size the offset was saved with
    pub screen_size: Vector2,

    /// What scale is this element set to?
    pub scale: Vector2,

    /// Is this element visible?
    pub visible: bool
}
impl UiElementLayout {
    pub fn new_default(
        anchor: UiElementAnchor,
        align: Alignment,
        inner_align: Option<Alignment>,
        offset: Option<Vector2>,
    ) -> Self {
        Self {
            anchor,
            align,
            inner_align,
            offset: offset.unwrap_or_default(),
            screen_size: Vector2::new(1920.0, 1080.0),
            scale: Vector2::ONE,
            visible: true
        }
    }
}

#[derive(Debug)]
pub enum LayoutError {
    CyclicDependencyDetected,
    InvalidElementReference(String),
}

pub struct UiElementLayoutinator;
impl UiElementLayoutinator {
    pub fn layout(
        elements: &mut [UIElement],
        playfield: Bounds, 
        screen_size: Vector2,
    ) -> Result<(), LayoutError> {
        // list of elements that have been layed out
        let mut layed_out = Vec::with_capacity(elements.len());
        let mut remaining = Vec::with_capacity(elements.len());

        // lay out everything with a screen or absolute anchor
        for e in elements.iter_mut() {
            e.pos_offset = Vector2::ZERO;
            let bounds = e.get_bounds();
            let mut scale = (screen_size / e.layout.screen_size).min_component();

            match e.layout.anchor {
                UiElementAnchor::Screen => {
                    let offset = e.layout.offset * scale;
                    e.scale = e.layout.scale * scale;

                    e.pos_offset = offset + bounds.pos + e.layout.align.resolve(
                        &Bounds::new(Vector2::ZERO, screen_size),
                        bounds.size,
                        true,
                        true,
                    );

                    layed_out.push((e, scale));
                }
                
                UiElementAnchor::Playfield {
                    saved_size,
                    relative
                } => {
                    if let Some(saved_size) = saved_size {
                        scale = (playfield.size / saved_size).min_component();
                    }

                    let offset = e.layout.offset * scale;
                    e.scale = e.layout.scale * scale;
                    
                    e.pos_offset = offset + bounds.pos + e.layout.align.resolve(
                        &playfield,
                        bounds.size,
                        [UiElementAlign::Above, UiElementAlign::Below, UiElementAlign::Inside].contains(&relative),
                        [UiElementAlign::Left, UiElementAlign::Right, UiElementAlign::Inside].contains(&relative),
                    );
                    
                    layed_out.push((e, scale));
                }

                // skip elements in the first pass
                _ => {
                    remaining.push(e);
                    continue
                }
            }
        }

        // there is likely a better way of doing this
        loop {
            if remaining.is_empty() { break }
            let remaining_count = remaining.len();

            for i in 0..remaining_count {
                let Some(e) = remaining.get_mut(i) else { break };
                let UiElementAnchor::Element { element: anchor_ele, relative } = &e.layout.anchor else { unreachable!() };

                let Some((anchor_ele, scale)) = layed_out
                    .iter()
                    .find(|(e, _)| &e.element_name == anchor_ele) 
                    else { continue };

                let scale = *scale;
                let anchor_bounds = anchor_ele.get_bounds();
                
                let offset = e.layout.offset * scale;
                e.scale = e.layout.scale * scale;
                
                if relative == &UiElementAlign::Inside {
                    // elements should not be inside other elements
                    warn!("WARNING!! element {} is set to be inside element {}", e.element_name, anchor_ele.element_name);
                }

                let bounds = e.get_bounds();
                e.pos_offset = offset + bounds.pos + e.layout.align.resolve(
                    &anchor_bounds,
                    bounds.size,
                    [UiElementAlign::Above, UiElementAlign::Below].contains(relative),
                    [UiElementAlign::Left, UiElementAlign::Right].contains(relative),
                );

                layed_out.push((remaining.swap_remove(i) , scale));
            }
            
            if remaining_count == remaining.len() {

                for r in &remaining {
                    let UiElementAnchor::Element { element: anchor_ele, .. } = &r.layout.anchor else { unreachable!() }; 

                    let found = layed_out
                        .iter()
                        .map(|(e, _)| e).chain(&remaining)
                        .any(|e| &e.element_name == anchor_ele);

                    if !found {
                        return Err(LayoutError::InvalidElementReference(anchor_ele.clone()));
                    }
                }

                return Err(LayoutError::CyclicDependencyDetected);
            }
        }

        Ok(())
    }
}


/// TODO: somehow merge this with alignment? 
/// my brain just isnt working properly enough to manually calculate this 
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[derive(Serialize, Deserialize)]
pub enum UiElementAlign {
    /// Inside the parent
    Inside, 

    /// Above the parent
    Above, 

    /// Below the parent
    Below,

    /// To the left of the parent
    Left, 

    /// To the right of the parent
    Right
}

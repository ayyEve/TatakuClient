use crate::prelude::*;

pub struct GameplayWidgetContainer {
    // pub default_pos: Vector2,
    pub element_name: String,
    pub pos_offset: Vector2,
    pub scale: Vector2,
    // pub visible: bool,

    pub layout: GameplayWidgetLayout,
    pub default_layout: GameplayWidgetLayout,

    pub inner: Box<dyn GameplayWidget>,
}
impl GameplayWidgetContainer {
    pub fn update(&mut self, manager: &mut dyn GameplayManagerTrait) {
        if !self.layout.visible { return }
        self.inner.update(manager);
    }

    #[cfg(feature="graphics")]
    pub fn draw(&mut self, list: &mut RenderableCollection) {
        if !self.layout.visible { return }
        let align = self.layout.inner_align.unwrap_or(self.layout.align);
        self.inner.draw(self.pos_offset, self.scale, align, list);
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
    pub fn reload_skin(&mut self, source: &TextureSource, skin_manager: &mut dyn SkinProvider) {
        self.inner.reload_skin(source, skin_manager);
    }



    pub fn layout(
        elements: &mut [GameplayWidgetContainer],
        playfield: Bounds, 
        screen_size: Vector2,
    ) -> Result<(), GameplayWidgetLayoutError> {
        // list of elements that have been layed out
        let mut layed_out = Vec::with_capacity(elements.len());
        let mut remaining = Vec::with_capacity(elements.len());

        // lay out everything with a screen or absolute anchor
        for e in elements.iter_mut() {
            e.pos_offset = Vector2::ZERO;
            let bounds = e.get_bounds();
            let mut scale = (screen_size / e.layout.screen_size).min_component();

            match e.layout.anchor {
                GameplayWidgetAnchor::Screen => {
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
                
                GameplayWidgetAnchor::Playfield {
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
                        [GameplayWidgetAlign::Above, GameplayWidgetAlign::Below, GameplayWidgetAlign::Inside].contains(&relative),
                        [GameplayWidgetAlign::Left, GameplayWidgetAlign::Right, GameplayWidgetAlign::Inside].contains(&relative),
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
                let GameplayWidgetAnchor::Element { element: anchor_ele, relative } = &e.layout.anchor else { unreachable!() };

                let Some((anchor_ele, scale)) = layed_out
                    .iter()
                    .find(|(e, _)| &e.element_name == anchor_ele) 
                    else { continue };

                let scale = *scale;
                let anchor_bounds = anchor_ele.get_bounds();
                
                let offset = e.layout.offset * scale;
                e.scale = e.layout.scale * scale;
                
                if relative == &GameplayWidgetAlign::Inside {
                    // elements should not be inside other elements
                    warn!("WARNING!! element {} is set to be inside element {}", e.element_name, anchor_ele.element_name);
                }

                let bounds = e.get_bounds();
                e.pos_offset = offset + bounds.pos + e.layout.align.resolve(
                    &anchor_bounds,
                    bounds.size,
                    [GameplayWidgetAlign::Above, GameplayWidgetAlign::Below].contains(relative),
                    [GameplayWidgetAlign::Left, GameplayWidgetAlign::Right].contains(relative),
                );

                layed_out.push((remaining.swap_remove(i) , scale));
            }
            
            if remaining_count == remaining.len() {
                for r in &remaining {
                    let GameplayWidgetAnchor::Element { element: anchor_ele, .. } = &r.layout.anchor else { unreachable!() }; 

                    let found = layed_out
                        .iter()
                        .map(|(e, _)| e).chain(&remaining)
                        .any(|e| &e.element_name == anchor_ele);

                    if !found {
                        return Err(GameplayWidgetLayoutError::InvalidElementReference(anchor_ele.clone().into_owned()));
                    }
                }

                return Err(GameplayWidgetLayoutError::CyclicDependencyDetected);
            }
        }

        Ok(())
    }

}
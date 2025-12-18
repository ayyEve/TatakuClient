use crate::prelude::*;
use input::InputEvent;
use tataku::{
    Vector2,
    Color,
    Border,
};
use ui::{
    tree::*,
    style::*,
    widget::*,
};

// TODO: move button (etc) active/hover/etc to states, and use css selectors to set the states

pub struct WidgetBase<T = Box<dyn Widget<actions::Action>>> {
    element_name: ArcStr,
    id: Option<ArcStr>,
    style_str: ArcStr,
    class: ClassList,
    pub inner: T,
}
impl<T> WidgetBase<T> {
    pub fn new(
        style: ArcStr,
        element_name: impl Into<ArcStr>,
        id: Option<ArcStr>,
        class: ClassList,
        inner: T,
    ) -> Self {
        Self {
            style_str: style,
            element_name: element_name.into(),
            id,
            class,
            inner,
        }
    }

    // pub fn new_boxed(
    //     style: ArcStr,
    //     element_name: impl Into<ArcStr>,
    //     id: Option<ArcStr>,
    //     class: ClassList,
    //     inner: Box<dyn Widget<actions::Action>>,
    // ) -> Box<dyn Widget<actions::Action>> {
    //     Self::new(
    //         style,
    //         element_name,
    //         id,
    //         class,
    //         inner
    //     )
    //     .boxed()
    // }
    
}
impl<T> Widget<actions::Action> for WidgetBase<T>
where
    T: Widget<actions::Action>,
{
    fn name(&self) -> CowStr { self.inner.name() }
    fn node_id(&self) -> NodeId { self.inner.node_id() }

    fn children(&self) -> WidgetChildren<'_, actions::Action> {
        WidgetChildren::Single(&self.inner)
    }
    fn children_mut(&mut self) -> WidgetChildrenMut<'_, actions::Action> {
        WidgetChildrenMut::Single(&mut self.inner)
    }

    fn init_style(&mut self, shell: &mut LayoutShell<actions::Action>) {
        let node = self.node_id();
        let styles = shell.resolver.resolve_style(
            &self.style_str, 
            node, 
            shell.tree,
        );

        let ctx = shell.tree.get_context_mut(node).unwrap();
        ctx.set_styles(styles, shell.values);

        self.inner.init_style(shell);
    }

    fn layout(&mut self, shell: &mut LayoutShell<actions::Action>) -> taffy::TaffyResult<NodeId> {
        let id = self.inner.layout(shell)?;
        shell.with_context(id, |ctx| {
            ctx.element_data = ElementData {
                state: ElementState::None,
                element_name: self.element_name.clone(),
                id: self.id.clone(),
                class_list: self.class.0.clone(),
                ..Default::default()
            }
        });

        Ok(id)
    }

    fn input(&mut self, event: &InputEvent, shell: &mut InputShell<actions::Action>) {
        // let node = self.node_id();
        // let previous_state = shell.tree
        //     .get_context(node)
        //     .unwrap()
        //     .element_data
        //     .state;
        
        self.inner.input(event, shell);
        
        // update the style if the state changed
        // let ctx = shell.tree.get_context(node).unwrap();
        // if previous_state != ctx.element_data.state {
        //     let current = &ctx.element_data.style().0;
        //     // shell.actions.push(UiAction::new(
        //     //     node, 
        //     //     UiActionType::UpdateStyle(Box::new(current.clone()))
        //     // ));

        //     self.inner.set_text_style(current.text_style(shell.values));
        // }
    }
    
    fn draw(&self, shell: &mut DrawShell<actions::Action>) {
        let node = self.node_id();
        let Some(bounds) = shell.tree.absolute_bounds(node) else { return };
        let Some(layout) = shell.tree.get_layout(node) else { return };
        let Some(ctx) = shell.tree.get_context(node) else { return };

        let (style, image) = ctx.current_style();

        // background
        let mut border = style.border_color
            .resolve(shell.values)
            .map(|color| Border::new(*color, 2.0));

        let border_top = layout.border.top;
        if border_top > 0.0 {
            if let Some(border) = &mut border {
                border.width = border_top;
            } else {
                border = Some(Border::new(Color::BLACK, border_top));
            }
        }

        let shape = style
            .border_radius
            .resolve(shell.values)
            .as_deref()
            .copied()
            .map(graphics::Shape::Round);

        if let Some(bg) = style
            .background_color
            .resolve_copied(shell.values)
        {
            shell.list.push(graphics::Rectangle::new_bounds(
                    bounds,
                    bg,
                )
                .border_maybe(border)
                .shape_maybe(shape)
            );
        } else if let Some(border) = border {
            shell.list.push(
                graphics::Rectangle::new_bounds(
                    bounds,
                    Color::TRANSPARENT,
                )
                .border(border)
                .shape_maybe(shape)
            );
        }

        // image
        if let Some(mut image) = image.clone() {
            let alignment = style
                .image_alignment
                .resolve_copied(shell.values)
                .unwrap_or(tataku::Alignment::CENTER);

            if let Some(&fill_mode) = style.image_stretch.value() {
                image.fit_to(fill_mode, bounds);
            }

            image.pos = alignment.resolve(
                &bounds,
                image.size(),
                true, true
            );

            shell.list.push(image);
        }

        // blur
        let blur_amount = style.blur_amount
            .resolve_copied(shell.values)
            .unwrap_or_default();

        let blur_type = style.blur_type
            .resolve_copied(shell.values)
            .unwrap_or(CssBlurType::Box);

        let blur_location = style.blur_location
            .resolve_copied(shell.values)
            .unwrap_or_default();

        let blur = if blur_amount > 0.0 {
            Some(blur_type.into_blur(blur_amount))
        } else { None };

        if let Some(blur) = blur
        && blur_location == BlurLocation::Below {
            shell.list.push(graphics::Blur::new(bounds, blur));
        }

        // draw the inner
        self.inner.draw(shell);

        if ctx.selected == Some(true) {
            shell.list.push(
                graphics::Rectangle::new_bounds(
                    bounds,
                    Color::TRANSPARENT,
                )
                .border(Border::new(Color::RED, 3.0))
            );
        }

        if let Some(blur) = blur
        && blur_location == BlurLocation::Above {
            shell.list.push(graphics::Blur::new(bounds, blur));
        }
    }
    

    fn reload_skin(&mut self, shell: &mut UpdateShell<actions::Action>) {
        let Some(ctx) = shell.tree
            .get_context_mut(self.node_id()) 
        else { return };

        for (style, img) in ctx
            .element_data.styles.all_mut()
        {
            if let Some(image) = style.image
                .resolve(shell.values)
            {
                let source = style.image_source
                    .resolve_cloned(shell.values)
                    .unwrap_or(graphics::TextureSource::Skin);

                *img = shell.skin_manager.get_texture_then(
                    &image, 
                    &source, 
                    graphics::SkinUsage::Game, 
                    style.image_grayscale
                        .resolve_copied(shell.values)
                        .unwrap_or_default(), 
                    |image| image.origin = Vector2::ZERO,
                );
            }
        }

        self.inner.reload_skin(shell);
    }

}

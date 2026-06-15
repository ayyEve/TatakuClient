use crate::prelude::*;
use input::InputEvent;
use tataku::{
    Vector2,
    Color,
    Border,
};
use tataku_ui::style::css::CssProperty;
use ui::{
    tree::*,
    style::*,
    widget::*,
};

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
        element_name: ArcStr,
        id: Option<ArcStr>,
        class: ClassList,
        inner: T,
    ) -> Self {
        Self {
            style_str: style,
            element_name,
            id,
            class,
            inner,
        }
    }
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

        shell.tree.set_styles(node, styles);
        self.inner.init_style(shell);
    }

    fn layout(&mut self, shell: &mut LayoutShell<actions::Action>) -> taffy::TaffyResult<NodeId> {
        let id = self.inner.layout(shell)?;
        shell.with_context(id, |ctx| {
            ctx.element_data = ElementData {
                state: ElementState::None,
                default_style: self.style_str.clone(),
                element_name: self.element_name.clone(),
                id: self.id.clone(),
                class_list: self.class.0.clone(),
                ..Default::default()
            }
        });

        Ok(id)
    }

    fn input(
        &mut self, 
        event: &InputEvent, 
        shell: &mut InputShell<actions::Action>
    ) {
        let node = self.node_id();

        let Some(state) = shell.state(node) 
        else { return };
        self.inner.input(event, shell);

        let new_state = shell.state(node).unwrap();
        if new_state != state {
            shell.tree.mark_dirty(node);
        }
        
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

        // let (_, image) = ctx.element_data.style(); //.current_style();
        let style = shell.tree.get_style(node).unwrap();

        // background
        let mut border = style
            .get_resolved::<Color>(CssProperty::BorderColor, shell.values)
            .map(|color| Border::new(color, 2.0));

        let border_top = layout.border.top;
        if border_top > 0.0 {
            if let Some(border) = &mut border {
                border.width = border_top;
            } else {
                border = Some(Border::new(Color::BLACK, border_top));
            }
        }

        let shape = style
            .get_resolved::<f32>(CssProperty::BorderRadius, shell.values)
            .map(graphics::Shape::Round);

        if let Some(bg) = style.get_resolved(CssProperty::BackgroundColor, shell.values)
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

        // FIXME:
        // // image
        // if let Some(mut image) = image.clone() {
        //     let alignment = style
        //         .get_resolved(CssProperty::ImageAlignment, shell.values)
        //         .unwrap_or(tataku::Alignment::CENTER);

        //     if let Some(&fill_mode) = style.image_stretch.value() {
        //         image.fit_to(fill_mode, bounds);
        //     }

        //     image.pos = alignment.resolve(
        //         &bounds,
        //         image.size(),
        //         true, true
        //     );

        //     shell.list.push(image);
        // }

        // blur
        let blur_amount = style
            .get_resolved::<f32>(CssProperty::BlurAmount, shell.values)
            .unwrap_or_default();

        let blur_type = style
            .get_resolved(CssProperty::BlurType, shell.values)
            .unwrap_or(CssBlurType::Box);

        let blur_location = style
            .get_resolved::<BlurLocation>(CssProperty::BlurLocation, shell.values)
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
    

    // FIXME:
    fn reload_skin(&mut self, shell: &mut UpdateShell<actions::Action>) {
        let Some(style) = shell.tree.get_style(self.node_id()) 
        else { return };

        // let image = style.image.resolve_cloned(shell.values);
        // let image_source = style.image_source
        //     .resolve_cloned(shell.values)
        //     .unwrap_or(graphics::TextureSource::Skin);
        // let image_grayscale = style.image_grayscale
        //     .resolve_copied(shell.values)
        //     .unwrap_or_default();

        // let Some(ctx) = shell.tree
        //     .get_context_mut(self.node_id()) 
        // else { return };


        

        // for (_, img) in ctx
        //     .element_data.styles.all_mut()
        // {
        //     if let Some(image) = &image {
        //         *img = shell.skin_manager.get_texture_then(
        //             Path::new(image), 
        //             &image_source, 
        //             graphics::SkinUsage::Game, 
        //             image_grayscale, 
        //             |image| image.origin = Vector2::ZERO,
        //         );
        //     }
        // }

        self.inner.reload_skin(shell);
    }

}

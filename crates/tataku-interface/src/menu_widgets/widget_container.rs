use crate::prelude::*;
use crate::prelude::ui::*;

// TODO: move button (etc) active/hover/etc to states, and use css selectors to set the states

// TODO: rename this please
pub struct WidgetContainer {
    element_name: String,
    id: Option<String>,
    style_str: String,
    class: ClassList,

    inner: Box<dyn Widget>,
}
impl WidgetContainer {
    pub fn new(
        style: String,
        element_name: impl Into<String>,
        id: Option<String>,
        class: ClassList,
        inner: Box<dyn Widget>,
    ) -> Self {
        Self {
            style_str: style,
            element_name: element_name.into(),
            id,
            class,
            inner,
        }
    }

    pub fn new_boxed(
        style: String,
        element_name: impl Into<String>,
        id: Option<String>,
        class: ClassList,
        inner: Box<dyn Widget>,
    ) -> Box<dyn Widget> {
        Self::new(
            style,
            element_name,
            id,
            class,
            inner
        )
        .boxed()
    }
    
}
impl Widget for WidgetContainer {
    fn name(&self) -> Cow<'static, str> { self.inner.name() }
    fn node_id(&self) -> NodeId { self.inner.node_id() }

    fn update_styles(
        &mut self, 
        shell: &mut StyleShell,
        display_override: Option<ui::Display>
    ) {
        let node = self.node_id();
        let a = shell.resolver.resolve_style(
            &self.style_str, 
            node, 
            shell.tree,
        );

        let ctx = shell.tree.get_context_mut(node).unwrap();
        ctx.element_data.styles = a.transpose();

        let current = &ctx.element_data.style().0;
        self.inner.set_text_style(current.text_style(shell.values));
        
        let mut taffy_style = current.taffy_style();
        if let Some(display_override) = display_override {
            taffy_style.display = display_override;
        }
        shell.tree.set_style(node, taffy_style);

        self.inner.update_styles(shell, display_override);
    }

    fn layout(&mut self, shell: &mut LayoutShell) -> TaffyResult<NodeId>  {
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
    
    fn input(&mut self, event: &InputEvent, shell: &mut InputShell) {
        let node = self.node_id();
        let previous_state = shell.tree
            .get_context(node)
            .unwrap()
            .element_data
            .state;
        
        self.inner.input(event, shell);
        
        // update the style if the state changed
        let ctx = shell.tree.get_context(node).unwrap();
        if previous_state != ctx.element_data.state {
            let current = &ctx.element_data.style().0;
            shell.actions.push(UiAction::new(
                node, 
                UiActionType::UpdateStyle(Box::new(current.taffy_style()))
            ));

            self.inner.set_text_style(current.text_style(shell.values));
        }
    }
    
    fn draw(&self, shell: &mut DrawShell) {
        let node = self.node_id();
        let Some(bounds) = shell.tree.absolute_bounds(node) else { return };
        let Some(layout) = shell.tree.get_layout(node) else { return };
        let Some(ctx) = shell.tree.get_context(node) else { return };

        let (style, image) = ctx
            .element_data
            .styles
            .get_style(ctx.element_data.state);

        // background
        let mut border = style.border_color
            .value_var(shell.values)
            .map(|color| Border::new(*color, 2.0));

        if style.border_width.value().is_some() {
            let width = layout.border.top;

            if let Some(border) = &mut border {
                border.radius = width;
            } else {
                border = Some(Border::new(Color::BLACK, width));
            }
        }

        let shape = style
            .border_radius
            .value_var(shell.values)
            .as_deref()
            .copied()
            .map(Shape::Round);

        if let Some(bg) = style.background_color
            .value_var(shell.values) {
            shell.list.push(Rectangle::new_bounds(
                    bounds,
                    *bg,
                    border
                )
                .shape_maybe(shape)
            );
        } else if let Some(border) = border {
            shell.list.push(Rectangle::new_bounds(
                    bounds,
                    Color::TRANSPARENT,
                    Some(border)
                )
                .shape_maybe(shape)
            );
        }

        // image
        if let Some(mut image) = image.clone() {
            let alignment = style.image_alignment.value()
            .copied()
            .unwrap_or(Alignment::CENTER);

            if let Some(&fill_mode) = style.image_fit.value() {
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
        let blur_amount = style.blur
            .value_var(shell.values)
            .as_deref()
            .copied()
            .unwrap_or_default();

        let blur_location = style.blur_location.value()
            .copied()
            .unwrap_or_default();

        let should_blur = blur_amount > 0.0;
        if should_blur && blur_location == BlurLocation::Below {
            shell.list.push(Blur::new(bounds, blur_amount));
        }

        // draw the inner
        self.inner.draw(shell);

        if should_blur && blur_location == BlurLocation::Above {
            shell.list.push(Blur::new(bounds, blur_amount));
        }
    }
    
    fn draw_overlay(&self, shell: &mut DrawShell) {
        self.inner.draw_overlay(shell);
    }

    fn update(&mut self, shell: &mut UpdateShell) {
        self.inner.update(shell);
    }
    
    fn handle_message(
        &mut self, 
        message: &Message, 
        shell: &mut MessageShell,
    ) {
        self.inner.handle_message(message, shell);
    }
    
    fn handle_event(
        &mut self, 
        event: TatakuEventType, 
        event_value: Option<TatakuValue>, 
        shell: &mut MessageShell,
    ) {
        self.inner.handle_event(event, event_value, shell);
    }

    fn reload_skin(&mut self, shell: &mut UpdateShell) {
        let Some(ctx) = shell.tree
            .get_context_mut(self.node_id()) 
        else { return };

        for (style, img) in ctx.element_data.styles
            .all_mut() {
            if let Some(image) = style.image
                .value_var(shell.values) {
                let source = style.image_source.value()
                    .cloned()
                    .unwrap_or(TextureSource::Skin);

                *img = shell.skin_manager.get_texture_then(
                    &image, 
                    &source, 
                    SkinUsage::Game, 
                    style.image_grayscale.value()
                        .copied()
                        .unwrap_or_default(), 
                    |image| image.origin = Vector2::ZERO,
                );
            }
        }

        self.inner.reload_skin(shell);
    }

}

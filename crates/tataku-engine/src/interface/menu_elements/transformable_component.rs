use crate::prelude::*;

#[derive(Default)]
pub struct TransformableComponent {
    /// we let a TransformGroup handle the transforms to avoid duplicating code
    manager: TransformManager,
    triggers: Vec<AnimatableTrigger>,
    actions: HashMap<String, Vec<AnimatableAction>>,
}
impl TransformableComponent {
    pub fn new(
        triggers: Vec<AnimatableTrigger>,
        actions: HashMap<String, Vec<AnimatableAction>>,
    ) -> Self {
        Self {
            manager: TransformManager::new(Vector2::ZERO),
            triggers,
            actions,
        }
    }

    pub fn add_transform(&mut self, transform: Transformation) {
        self.manager.push_transform(transform);
    }

    pub fn view(&self, content: IcedElement) -> IcedElement {
        let transform = Transform::from_manager(&self.manager);

        Trimmed {
            content,
            transform,
        }.into_element()
    }


}

#[async_trait]
impl Widgetable for TransformableComponent {
    async fn update(&mut self, values: &mut dyn Reflect, actions: &mut ActionQueue) {
        let game_time = values.reflect_get::<f32>("game.time").unwrap_or(MaybeOwned::Owned(0.0));
        let time = match game_time {
            MaybeOwned::Borrowed(n) => *n,
            MaybeOwned::Owned(n) => n
        };

        self.manager.update(time);
        actions.push(GameAction::ForceUiRefresh);
    }

    // async fn handle_message(&mut self, message: &Message, _values: &mut dyn Reflect) -> Vec<TatakuAction> {
    //     Vec::new()
    // }


    async fn handle_event(&mut self, event: TatakuEvent, _event_value: Option<TatakuValue>, _values: &mut dyn Reflect) {
        // for i in self.triggers.iter() {
        //     let AnimatableTrigger { trigger: AnimatableTriggerEvent::Event(e), action } = i else { continue };
        // }
    }

}

struct Trimmed {
    content: IcedElement,
    transform: Transform,
}

impl Trimmed {
    fn transform_point<P: Into<Vector2> + From<Vector2>>(
        &self,
        layout: &iced_core::Layout,
        point: P
    ) -> P {
        let mut point = point.into();
        let pos: Vector2 = layout.position().into();

        point -= pos;
        point = self.transform.matrix().inverse().unwrap() * point;
        point += pos;

        point.into()
    }
}

impl iced::advanced::Widget<Message, iced::Theme, IcedRenderer> for Trimmed {
    fn size(&self) -> iced::Size<iced::Length> {
        iced::Size::new(iced::Length::Shrink, iced::Length::Shrink)
    }

    fn children(&self) -> Vec<iced_core::widget::Tree> {
        vec![iced_core::widget::Tree::new(self.content.as_widget())]
    }
    fn diff(&self, tree: &mut iced_core::widget::Tree) {
        tree.diff_children(&[self.content.as_widget()]);
        // self.content.as_widget().diff(tree);
    }

    fn operate(
        &self,
        state: &mut iced_core::widget::Tree,
        layout: iced_core::Layout<'_>,
        renderer: &IcedRenderer,
        operation: &mut dyn iced_core::widget::Operation,
    ) {
        operation.container(None, layout.bounds(), &mut |operation| {
            self.content.as_widget().operate(
                &mut state.children[0],
                layout.children().next().unwrap(),
                renderer,
                operation,
            );
        });
    }

    fn on_event(
        &mut self,
        state: &mut iced_core::widget::Tree,
        event: iced::Event,
        layout: iced_core::Layout<'_>,
        cursor: iced_core::mouse::Cursor,
        renderer: &IcedRenderer,
        clipboard: &mut dyn iced_core::Clipboard,
        shell: &mut iced_core::Shell<'_, Message>,
        viewport: &iced::Rectangle,
    ) -> iced::event::Status {
        use iced::mouse::{
            Event as MouseEvent,
            Cursor::Available,
        };

        let cursor = match cursor {
            Available(position) => {
                Available(self.transform_point(&layout, position))
            },

            other => other
        };


        let event = match event {
            iced::Event::Mouse(MouseEvent::CursorMoved { position }) => {
                iced::Event::Mouse(MouseEvent::CursorMoved { position: self.transform_point(&layout, position) })
            },
            // iced::Event::Window(event) => todo!(),
            // iced::Event::Touch(event) => todo!(),

            other => other
        };


        self.content.as_widget_mut().on_event(
            &mut state.children[0],
            event,
            layout.children().next().unwrap(),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport
        )
    }

    fn layout(
        &self,
        tree: &mut iced_core::widget::Tree,
        renderer: &IcedRenderer,
        limits: &iced_core::layout::Limits,
    ) -> iced_core::layout::Node {
        let layout = self.content.as_widget()
            .layout(&mut tree.children[0], renderer, limits)
            ;
        let bounds = layout.bounds();
        use iced::{ Point, Size };

        let bounds: Bounds = bounds.into();
        let transform = self.transform.matrix();

        let tl = transform * Vector2::ZERO;
        let br = transform * bounds.size;

        let size = br - tl;

        let tr = tl + Vector2::new(size.x, 0.0);
        let bl = tl + Vector2::new(0.0, size.y);

        let x = tl.x.min(tr.x).min(bl.x).min(br.x);
        let y = tl.y.min(tr.y).min(bl.y).min(br.y);

        let width = tl.x.max(tr.x).max(bl.x).max(br.x) - x;
        let height = tl.y.max(tr.y).max(bl.y).max(br.y) - y;

        let bounds = iced::Rectangle::new(Point::new(x, y), Size::new(width, height));
        let max_bounds = iced::Rectangle::new(Point::ORIGIN, limits.max());

        let intersection = bounds.intersection(&max_bounds).unwrap_or_default();


        iced_core::layout::Node::with_children(intersection.size(), vec![layout])
    }

    fn draw(
        &self,
        state: &iced_core::widget::Tree,
        renderer: &mut IcedRenderer,
        theme: &iced::Theme,
        style: &iced_core::renderer::Style,
        layout: iced_core::Layout<'_>,
        cursor: iced_core::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        renderer.start_transform(&self.transform);

        self.content.as_widget().draw(
            &state.children[0],
            renderer,
            theme,
            style,
            layout.children().next().unwrap(),
            cursor,
            viewport
        );

        renderer.end_transform();
    }

    fn mouse_interaction(
        &self,
        state: &iced_core::widget::Tree,
        layout: iced_core::Layout<'_>,
        cursor: iced_core::mouse::Cursor,
        viewport: &iced::Rectangle,
        renderer: &IcedRenderer,
    ) -> iced_core::mouse::Interaction {
        use iced_core::mouse::Cursor::Available;

        let cursor = match cursor {
            Available(position) => Available(self.transform_point(&layout, position)),
            other => other
        };

        self.content.as_widget().mouse_interaction(
            &state.children[0],
            layout.children().next().unwrap(),
            cursor,
            viewport,
            renderer
        )
    }

    fn overlay<'a>(
        &'a mut self,
        state: &'a mut iced_core::widget::Tree,
        layout: iced_core::Layout<'_>,
        renderer: &IcedRenderer,
        translation: iced::Vector,
    ) -> Option<iced_core::overlay::Element<'a, Message, iced::Theme, IcedRenderer>> {
        self.content
            .as_widget_mut()
            .overlay(
                &mut state.children[0],
                layout.children().next().unwrap(),
                renderer,
                translation
            )
    }
}

impl From<Trimmed> for IcedElement {
    fn from(value: Trimmed) -> Self {
        Self::new(value)
    }
}

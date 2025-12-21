use crate::prelude::*;
use common::reflect::*;
use tataku::{
    Color,
    HorizontalAlign,
};
use ui::{
    tree::*,
    style::*,
    widget::*,
};

#[derive(ChainableInitializer)]
pub struct Text {
    pub text: WidgetText,

    node_id: NodeId,

    layout: parley::Layout<Color>,
    old_x: f32,
    old_width: f32,
}
impl Text {
    pub fn new(text: WidgetText) -> Self {
        Self {
            text,
            node_id: ui::EMPTY_NODE,

            layout: parley::Layout::default(),
            old_x: 0.0,
            old_width: 0.0,
        }
    }

    pub fn text_layout(&self) -> &parley::Layout<Color> {
        &self.layout
    }

    fn recreate_layout(
        &mut self,
        tree: &mut Tree<actions::Action>,
        text_layout_contexts: &mut TextLayoutContexts,
    ) {
        let text = self.text.get();
        let text_style = tree.get_text_style(&self.node_id).unwrap();

        self.layout = text_layout_contexts.simple_text(
            &text,
            text_style,
        );

        if !text.is_empty() {
            let widths = self.layout.calculate_content_widths();

            tree.update_style(&self.node_id, |style| {
                // fixme: there is an off-by-one somewhere
                style.min_width = CssUnit::Pixels(f16::from_f32(widths.min.ceil() + 1.0)).into();
                style.max_width = CssUnit::Pixels(f16::from_f32(widths.max.ceil() + 1.0)).into();
            });
        }
    }

    fn wrap_and_align(
        &mut self,
        tree: &mut Tree<actions::Action>,
        container_width: f32,
    ) {
        let text = self.text.get();
        let text_style = tree.get_text_style(&self.node_id).unwrap();

        if !text.is_empty() {
            self.layout.break_all_lines(Some(container_width));

            let alignment = match text_style.alignment {
                HorizontalAlign::Left => parley::Alignment::Start,
                HorizontalAlign::Center => parley::Alignment::Middle,
                HorizontalAlign::Right => parley::Alignment::End,
            };

            self.layout.align(
                None,
                alignment,
                parley::AlignmentOptions::default(),
            );

            let height = self.layout.height();

            tree.update_style(&self.node_id, |style| {
                style.height = CssUnit::Pixels(f16::from_f32(height)).into();
            });
        }
    }
}
impl Widget<actions::Action> for Text {
    fn name(&self) -> CowStr { "text_widget".into() }
    fn node_id(&self) -> &NodeId { &self.node_id }

    fn layout(
        &mut self,
        shell: &mut LayoutShell<actions::Action>
    ) -> taffy::TaffyResult<NodeId> {
        self.node_id = shell.tree.new_leaf()?;
        Ok(self.node_id)
    }
    fn init_style(&mut self, shell: &mut LayoutShell<actions::Action>) {
        self.recreate_layout(
            shell.tree,
            shell.text_layout_contexts
        );

        let text_style = shell.tree
            .get_text_style(&self.node_id)
            .unwrap();
        let min_height = text_style.line_height;

        self.wrap_and_align(
            shell.tree,
            f32::MAX,
        );

        shell.tree.update_style(&self.node_id, |style| {
            style.min_height = CssUnit::Pixels(f16::from_f32(min_height)).into();
        });
    }

    fn update(&mut self, shell: &mut UpdateShell<actions::Action>) {
        let bounds = shell.tree.absolute_bounds(&self.node_id).unwrap();

        let refresh_text = self.text.update(shell.values);

        if refresh_text {
            self.recreate_layout(
                shell.tree,
                shell.text_layout_contexts,
            );
        }

        let refresh_layout = refresh_text
            || bounds.pos.x != self.old_x
            || bounds.size.x != self.old_width;

        if refresh_layout {
            self.old_x = bounds.pos.x;
            self.old_width = bounds.size.x;

            self.wrap_and_align(
                shell.tree,
                bounds.size.x,
            );
        }
    }

    fn draw(&self, shell: &mut DrawShell<actions::Action>) {
        // todo: handle this better
        let bounds = shell.tree.bounds(&self.node_id).unwrap();
        let context = shell.tree.get_context(&self.node_id).unwrap();
        let transform = context.global_transform
            * context.local_transform.matrix()
            * tataku::Matrix::identity().trans(bounds.pos);

        shell.list.push(graphics::Transformed {
            transform,
            drawable: Box::new(graphics::Text::new(self.layout.clone())),
        });
    }
}


// TODO: rename?
#[derive(Clone)]
pub enum WidgetText {
    String {
        value: CowStr,
        updated: bool,
    },
    Custom {
        custom: Vec<BuildableText>,
        cached: String,
    },
}
impl WidgetText {
    pub fn get(&self) -> Cow<'_, str> {
        match self {
            Self::String { value: Cow::Borrowed(s), .. } => Cow::Borrowed(*s),
            Self::String { value: Cow::Owned(s), .. } => Cow::Borrowed(s),
            Self::Custom { cached, .. } => Cow::Borrowed(cached),
        }
    }
    pub fn set(&mut self, value: String) {
        let old = self.get();
        let updated = old != value;

        *self = Self::String {
            value: Cow::Owned(value),
            updated,
        };
    }

    pub fn update(
        &mut self,
        values: &dyn Reflect
    ) -> bool {
        match self {
            Self::String { updated, .. } => updated.take(),
            Self::Custom { custom, cached } => {
                let new = custom.iter()
                    .map(|text| text.to_string(values))
                    .collect();

                if *cached != new {
                    *cached = new;
                    true
                } else {
                    false
                }
            },
        }
    }

    pub fn from_buildable_iter(iter: impl IntoIterator<Item = BuildableText>) -> Self {
        let values = iter.into_iter()
            .map(|mut text| {
                if let Err(e) = text.compute() {
                    error!("error parsing CustomElementText: {e:?}");

                    let BuildableText::Calc { calc } = text else { unreachable!() } ;

                    BuildableText::Text(format!("[Invalid calc {calc}]").into())
                } else {
                    text
                }
            })
            .collect::<Vec<_>>();

        // todo: if this is only called from TextElement (parsed from xml)
        // this will only ever have one text element.
        let combined_string = values.iter()
            .try_fold(
                String::new(),
                |acc, text| match text {
                    BuildableText::Text(text) => Some(acc + text),
                    _ => None,
                }
            );

        if let Some(string) = combined_string {
            Self::String {
                value: string.into(),
                updated: false,
            }
        } else {
            Self::Custom {
                custom: values,
                cached: String::new(),
            }
        }
    }
}
impl From<&str> for WidgetText {
    fn from(value: &str) -> Self {
        Self::String {
            value: Cow::Owned(value.to_owned()),
            updated: false,
        }
    }
}
impl From<String> for WidgetText {
    fn from(value: String) -> Self {
        Self::String {
            value: value.into(),
            updated: false,
        }
    }
}
impl From<BuildableText> for WidgetText {
    fn from(value: BuildableText) -> Self {
        Self::from_buildable_iter([value])
    }
}

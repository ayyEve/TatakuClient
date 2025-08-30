use crate::prelude::*;

use parley::{
    FontContext, LayoutContext, Layout, Alignment,
    swash::{
        FontRef,
        scale::{
            ScaleContext, Render, Source, StrikeWith,
        }
    },
};

#[derive(ChainableInitializer)]
pub struct TextWidget {
    text: WidgetText,
    node_id: NodeId,

    layout: Layout<[u8; 4]>,
    old_x: f32,
    old_width: f32,
}
impl TextWidget {
    pub fn new(text: impl Into<WidgetText>) -> Self {
        Self {
            text: text.into(),
            node_id: EMPTY_NODE,

            layout: Layout::default(),
            old_x: 0.0,
            old_width: 0.0,
        }
    }

    fn update_layout(
        &mut self,
        tree: &mut Tree<TatakuAction>,
        container_width: f32,
        font_context: &mut FontContext,
        text_layout_context: &mut LayoutContext,
    ) {
        let text = self.text.get();

        let text_style = tree.get_text_style(self.node_id).unwrap();

        if !text.is_empty() {
            self.layout = simple_text(
                &text,
                text_style,
                container_width,
                font_context,
                text_layout_context,
            );

            let widths = self.layout.calculate_content_widths();

            tree.update_style(self.node_id, |style| {
                style.min_width = CssUnit::Pixels(f16::from_f32(widths.min.ceil())).into();
                style.max_width = CssUnit::Pixels(f16::from_f32(widths.max.ceil())).into();
            });
        }
    }
}
impl Widget<TatakuAction> for TextWidget {
    fn name(&self) -> CowStr { "text_widget".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn layout(&mut self, shell: &mut LayoutShell<TatakuAction>) -> taffy::TaffyResult<NodeId> {
        self.node_id = shell.tree.new_leaf()?;
        Ok(self.node_id)
    }
    fn init_style(&mut self, shell: &mut LayoutShell<TatakuAction>) {
        self.update_layout(
            shell.tree,
            9999.0,
            shell.font_context,
            shell.text_layout_context
        );

        let text_style = shell.tree.get_text_style(self.node_id).unwrap();
        let min_height = text_style.line_height;

        shell.tree.update_style(self.node_id, |style| {
            style.min_height = CssUnit::Pixels(f16::from_f32(min_height)).into();
        });
    }

    fn update(&mut self, shell: &mut UpdateShell<TatakuAction>) {
        let bounds = shell.tree.absolute_bounds(self.node_id).unwrap();

        let parent_bounds = shell.tree.parent(self.node_id)
            .and_then(|parent| shell.tree.absolute_bounds(parent));

        if let Some(parent_bounds) = parent_bounds {
            if parent_bounds.intersection(bounds).is_none() {
                return;
            }
        }

        let refresh = self.text.update(shell.values)
            || bounds.pos.x != self.old_x
            || bounds.size.x != self.old_width;

        if refresh {
            self.old_x = bounds.pos.x;
            self.old_width = bounds.size.x;

            self.update_layout(
                shell.tree,
                bounds.size.x,
                shell.font_context,
                shell.text_layout_context
            );
        }
    }
    
    fn draw(&self, shell: &mut DrawShell<TatakuAction>) {
        let transform = shell.tree.get_context(self.node_id).unwrap().global_transform;
        let text_style = shell.tree.get_text_style(self.node_id).unwrap();

        let glyphs = rasterize_layout(
            &self.layout,
            text_style.color,
            shell.scale_context,
        );

        for glyph in glyphs {
            shell.list.push(Transformed {
                transform,
                drawable: Box::new(glyph),
            });
        }
    }
}


// TODO: rename?
pub enum WidgetText {
    String(CowStr),
    Custom {
        custom: BuildableText,
        cached: String,
    },
}
impl WidgetText {
    pub fn get(&self) -> Cow<'_, str> {
        match self {
            Self::String(Cow::Borrowed(s)) => Cow::Borrowed(*s),
            Self::String(Cow::Owned(s)) => Cow::Borrowed(s),
            Self::Custom { cached, .. } => Cow::Borrowed(cached),
        }
    }
    pub fn set(&mut self, value: String) {
        match self {
            Self::String(cow) => *cow = Cow::Owned(value),
            Self::Custom { cached, .. } => *cached = value,
        }
    }

    pub fn update(
        &mut self,
        values: &dyn Reflect
    ) -> bool {
        let Self::Custom { custom, cached } = self else { return false };
        let new = custom.to_string(values);
        if *cached != new {
            *cached = new;
            true
        } else {
            false
        }
    }
}
impl From<&str> for WidgetText {
    fn from(value: &str) -> Self {
        Self::String(Cow::Owned(value.to_owned()))
    }
}
impl From<String> for WidgetText {
    fn from(value: String) -> Self {
        Self::String(value.into())
    }
}
impl From<BuildableText> for WidgetText {
    fn from(mut value: BuildableText) -> Self {
        if let Err(e) = value.compute() {
            error!("error parsing CustomElementText: {e:?}");
        }

        match value {
            BuildableText::Text { text } => Self::String(text.to_string().into()),
            custom => Self::Custom {
                custom,
                cached: String::new()
            }
        }
    }
}

pub fn simple_text(
    text: &str,
    style: &TextStyle,

    container_width: f32,

    font_context: &mut FontContext,
    text_layout_context: &mut LayoutContext,
) -> Layout<[u8; 4]> {
    let mut builder = text_layout_context.tree_builder(
        font_context,
        1.0, // gui/dpi scale
        true,
        &style.into(),
    );

    builder.push_text(text);

    let (mut layout, _text) = builder.build();

    layout.break_all_lines(Some(container_width));

    let alignment = match style.alignment.horizontal {
        HorizontalAlign::Left => Alignment::Start,
        HorizontalAlign::Center => Alignment::Middle,
        HorizontalAlign::Right => Alignment::End,
    };

    layout.align(
        None,
        alignment,
        parley::AlignmentOptions::default(),
    );

    layout
}

pub fn rasterize_layout(
    layout: &Layout<[u8; 4]>,

    color: Color,

    scale_context: &mut ScaleContext,
) -> Vec<Transformed> {
    let runs = layout.lines()
        .flat_map(|line| line.items())
        .flat_map(|item| match item {
            parley::PositionedLayoutItem::GlyphRun(glyph_run) => Some(glyph_run),
            parley::PositionedLayoutItem::InlineBox(_) => None,
        });

    let mut render = Render::new(&[
        // Color outline with the first palette
        Source::ColorOutline(0),
        // Color bitmap with best fit selection mode
        Source::ColorBitmap(StrikeWith::BestFit),
        // Standard scalable outline
        Source::Outline,
    ]);

    let mut glyphs = Vec::new();

    for run in runs {
        let font = run.run().font();
        let size = run.run().font_size();

        let mut scaler = scale_context.builder(FontRef::from_index(font.data.data(), font.index as usize).unwrap())
            .size(size)
            .build();

        for glyph in run.positioned_glyphs() {
            let offset = [
                glyph.x.fract(),
                0.0, // quantize = true
            ];

            render.offset(offset.into());

            let Some(image) = render.render(&mut scaler, glyph.id) else { continue; };

            let x = glyph.x.floor() as i32;
            let y = glyph.y.floor() as i32;

            // convert from bottom-left to top-left image
            let x = x + image.placement.left;
            let y = y - image.placement.top;

            let glyph = Glyph {
                alpha_mask: image.data,
                color,
                size: [image.placement.width, image.placement.height],
            };

            let transform = Transform::default()
                .translate(Vector2::new(x as f32, y as f32));

            glyphs.push(Transformed::new(transform, Box::new(glyph)));
        }
    }

    glyphs
}

pub struct Glyph {
    alpha_mask: Vec<u8>,
    color: Color,
    size: [u32; 2],
}

impl TatakuRenderable for Glyph {
    fn get_blend_mode(&self) -> Pipeline { Pipeline::AlphaBlending }
    fn set_blend_mode(&mut self, _blend_mode: Pipeline) {}

    fn draw(
        &self,
        options: &DrawOptions,
        transform: Matrix,
        g: &mut dyn GraphicsEngine,
    ) {
        let data = self.alpha_mask.iter()
            .map(|&alpha| self.color.alpha8(alpha))
            .flat_map(|color| [color.r, color.g, color.b, color.a])
            .collect::<Vec<_>>();

        let tex = g.load_texture_rgba(&data, self.size).unwrap();

        g.draw_tex(
            &tex,
            Color::WHITE,
            false,
            false,
            transform,
            Pipeline::AlphaBlending,
        );

        g.free_tex(tex, true);
    }
}

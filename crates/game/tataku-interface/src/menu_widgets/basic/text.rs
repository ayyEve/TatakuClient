use crate::prelude::*;

use parley::{
    FontContext, LayoutContext, Font, Layout,
    swash::{
        FontRef, GlyphId, CacheKey,
        scale::{
            ScaleContext, Render, Source, StrikeWith,
            image::Image as SwashImage,
        }
    },
};

#[derive(ChainableInitializer)]
pub struct TextWidget {
    text: WidgetText,
    node_id: NodeId,

    glyphs: Vec<Image>,
}
impl TextWidget {
    pub fn new(text: impl Into<WidgetText>) -> Self {
        Self {
            text: text.into(),
            node_id: EMPTY_NODE,

            glyphs: Vec::new(),
        }
    }
}
impl Widget<TatakuAction> for TextWidget {
    fn name(&self) -> CowStr { "text_widget".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn layout(&mut self, shell: &mut LayoutShell<TatakuAction>) -> taffy::TaffyResult<NodeId> {
        // self.text_style.font_size *= shell.ui_scale;
        self.node_id = shell.tree.new_leaf()?;
        Ok(self.node_id)
    }
    fn init_style(&mut self, shell: &mut LayoutShell<TatakuAction>) {
        // let min_size = self.min_size(shell.tree);
        // shell.tree.update_style(self.node_id, |style| {
        //     style.min_width = min_size[0].into();
        //     style.min_height = min_size[1].into();
        // });
    }

    fn update(&mut self, shell: &mut UpdateShell<TatakuAction>) {
        if self.glyphs.is_empty() || self.text.update(shell.values) {
            let text = self.text.get();

            let bounds = shell.tree.content_bounds(self.node_id).unwrap();
            let text_style = shell.tree.get_text_style(self.node_id).unwrap();

            // shell.actions.push(UiAction::new(
            //     self.node_id,
            //     UiActionType::UpdateStyleWith(Arc::new(
            //         move |style| {
            //             style.min_width = CssUnit::Pixels(f16::from_f32(widths.min)).into();
            //             style.max_width = CssUnit::Pixels(f16::from_f32(widths.max)).into();
            //             // style.min_height = min[1].into();
            //         }
            //     ))
            // ));
            // shell.actions.push(UiAction::new(
            //     self.node_id,
            //     UiActionType::MarkDirty,
            // ));
        }

        // self.text_style = shell
        //     .tree
        //     .get_context(self.node_id)
        //     .unwrap()
        //     .element_data
        //     .style()
        //     .0
        //     .text_style(shell.values);
    }
    
    fn draw(&self, shell: &mut DrawShell<TatakuAction>) {
        for glyph in self.glyphs.iter() {
            shell.list.push(glyph.clone());
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

        Self::Custom {
            custom: value,
            cached: String::new()
        }
    }
}

pub fn simple_text(
    text: &str,
    style: &TextStyle,

    container_width: f32,

    font_context: &mut parley::FontContext,
    text_layout_context: &mut parley::LayoutContext,
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

    layout.align(
        None,
        parley::Alignment::default(), // todo:
        parley::AlignmentOptions::default(),
    );

    layout
}

pub fn rasterize_layout(
    layout: Layout<[u8; 4]>,

    color: Color,

    scale_context: &mut parley::swash::scale::ScaleContext,
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

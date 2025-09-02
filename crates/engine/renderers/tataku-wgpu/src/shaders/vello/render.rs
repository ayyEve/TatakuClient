use crate::prelude::*;
use vello::kurbo::Stroke;
use vello::peniko::{ Brush, Fill };

pub(crate) struct RenderEngine<'window, 'a> {
    wgpu: &'a mut WgpuEngine<'window>,
}
impl<'window, 'a> RenderEngine<'window, 'a> {
    pub fn new(wgpu: &'a mut WgpuEngine<'window>) -> Self {
        Self { wgpu }
    }
    fn scene(&mut self, blend_mode: tataku::GraphicsPipeline) -> Option<shaders::vello::ReserveData<'_>> {
        self.wgpu.reserve_vello(blend_mode)
    }
}
impl tataku_graphics::DrawEngine for RenderEngine<'_, '_> {
    fn push_scissor(&mut self, scissor: [f32; 4]) {
        self.wgpu.push_scissor(scissor);
        // let reserve = self.scene().unwrap();
        // reserve.scene.push_layer(
        //     vello::peniko::BlendMode::default(),
        //     1.0,
        //     vello::kurbo::Affine::IDENTITY,
        //     &map_rect(scissor),
        // );
    }

    fn pop_scissor(&mut self) {
        self.wgpu.pop_scissor();
        // self.scene()
        //     .unwrap()
        //     .scene
        //     .pop_layer();
    }

    fn draw_arc(
        &mut self,
        _start: f32,
        _end: f32,
        _radius: f32,
        _color: tataku::Color,
        _resolution: u32,
        _transform: tataku::Matrix,
        _blend_mode: tataku::GraphicsPipeline
    ) {
        
    }

    fn draw_circle(
        &mut self,
        radius: f32,
        color: tataku::Color,
        border: Option<tataku::Border>,
        _resolution: u32,
        transform: tataku::Matrix,
        blend_mode: tataku::GraphicsPipeline,
    ) {
        let reserve = self.scene(blend_mode).unwrap();
        let transform = map_transform(transform);
        let shape = vello::kurbo::Circle::new(
            (0.0, 0.0),
            radius as f64,
        );

        reserve.scene.fill(
            vello::peniko::Fill::NonZero,
            transform,
            map_color(color),
            None,
            &shape
        );

        if let Some(border) = border {
            reserve.scene.stroke(
                &Stroke {
                    width: border.width as f64,
                    ..Default::default()
                },
                transform,
                map_color(border.color),
                None,
                &shape
            );
        }

    }

    fn draw_line(
        &mut self,
        p: tataku::Vector2,
        thickness: f32,
        color: tataku::Color,
        transform: tataku::Matrix,
        blend_mode: tataku::GraphicsPipeline,
    ) {
        let reserve = self.scene(blend_mode).unwrap();

        let shape = vello::kurbo::Line::new(
            (0.0, 0.0),
            vello::kurbo::Point::new(
                p.x as f64,
                p.y as f64,
            )
        );

        reserve.scene.stroke(
            &vello::kurbo::Stroke {
                width: thickness as f64,
                ..Default::default()
            },
            map_transform(transform),
            map_color(color),
            None,
            &shape
        );
    }

    fn draw_rect(
        &mut self,
        rect: [f32; 4],
        border: Option<tataku::Border>,
        shape: tataku::Shape,
        color: tataku::Color,
        transform: tataku::Matrix,
        blend_mode: tataku::GraphicsPipeline
    ) {
        let reserve = self.scene(blend_mode).unwrap();
        let transform = map_transform(transform);
        let rect = map_rect(rect);
        
        match shape {
            tataku::Shape::Square => draw(
                &rect,
                border,
                transform,
                color,
                reserve,
            ),
            tataku_graphics::Shape::Round(radius) => draw(
                &vello::kurbo::RoundedRect::from_rect(rect, radius as f64),
                border,
                transform,
                color,
                reserve,
            ),
            tataku_graphics::Shape::RoundSep([a,b,c,d]) => draw(
                &vello::kurbo::RoundedRect::from_rect(
                    rect, 
                    (a as f64, b as f64, c as f64, d as f64)
                ),
                border,
                transform,
                color,
                reserve,
            ),
        };

        fn draw(
            shape: &impl vello::kurbo::Shape,
            border: Option<tataku::Border>,
            transform: Affine,
            color: tataku::Color,
            reserve: shaders::vello::ReserveData<'_>,
        ) {
            if color.a > 0 {
                reserve.scene.fill(
                    Fill::NonZero,
                    transform,
                    &Brush::Solid(map_color(color)),
                    None,
                    shape,
                );
            }

            let Some(border) = border.filter(tataku::Border::is_nonzero) 
            else { return };

            reserve.scene.stroke(
                &Stroke {
                    width: border.width as f64,
                    ..Default::default()
                },
                transform,
                &Brush::Solid(map_color(border.color)),
                None,
                shape,
            );
        }

    }

    fn draw_tex(
        &mut self,
        tex: &tataku_engine::prelude::TextureReference,
        color: tataku_engine::prelude::Color,
        h_flip: bool,
        v_flip: bool,
        transform: tataku_engine::prelude::Matrix,
        blend_mode: tataku_engine::prelude::GraphicsPipeline
    ) {
        self.wgpu.draw_tex(
            tex,
            color,
            h_flip,
            v_flip,
            transform,
            blend_mode
        );
    }

    fn draw_text(
        &mut self,
        transform: tataku::Matrix,
        blend_mode: tataku::GraphicsPipeline,
        layout: &parley::Layout<tataku::Color>,
    ) {
        let reserve = self.scene(blend_mode).expect("no scene!");

        let runs = layout.lines()
            .flat_map(|line| line.items())
            .flat_map(|item| match item {
                parley::PositionedLayoutItem::GlyphRun(glyph_run) => Some(glyph_run),
                parley::PositionedLayoutItem::InlineBox(_) => None,
            });

        for run in runs {
            let inner = run.run();
            let font = inner.font(); // todo: map

            reserve.scene
                .draw_glyphs(font)
                .brush(map_color(run.style().brush))
                .font_size(inner.font_size())
                .transform(map_transform(transform))
                .draw(
                    vello::peniko::Fill::NonZero,
                    run.positioned_glyphs()
                        .map(map_glyph)
                );
        }
    }

    fn draw_slider(
        &mut self,
        quad: [tataku::Vector2; 4],
        transform: tataku::Matrix,

        slider_data: tataku::SliderData,
        slider_grids: Vec<tataku::GridCell>,
        grid_cells: Vec<u32>,
        line_segments: Vec<tataku::LineSegment>
    ) {
        self.wgpu.draw_slider(
            quad,
            transform,
            slider_data,
            slider_grids,
            grid_cells,
            line_segments
        );
    }

    fn draw_flashlight(
        &mut self,
        quad: [tataku::Vector2; 4],
        transform: tataku::Matrix,
        flashlight_data: tataku::FlashlightData
    ) {
        self.wgpu.draw_flashlight(
            quad,
            transform,
            flashlight_data
        );
    }

    fn draw_gaussian_blur(
        &mut self,
        bounds: tataku::Bounds,
        sigma: f32,
        rounds: u32,
    ) {
        self.wgpu.draw_gaussian_blur(
            bounds,
            sigma,
            rounds
        );
    }

    fn draw_box_blur(
        &mut self,
        bounds: tataku::Bounds,
        size: u32,
    ) {
        self.wgpu.draw_box_blur(bounds, size);
    }
}


use vello::peniko::color::AlphaColor;
use vello::peniko::color::Srgb;

fn map_color(color: tataku::Color) -> AlphaColor<Srgb> {
    AlphaColor::from_rgba8(
        color.r,
        color.g,
        color.b,
        color.a
    )
}

use vello::kurbo::Affine;
fn map_transform(transform: tataku::Matrix) -> Affine {
    Affine::new(
        [
            transform.x.x as f64,
            transform.x.y as f64,

            transform.y.x as f64,
            transform.y.y as f64,

            transform.w.x as f64,
            transform.w.y as f64,
        ]
    )
}

fn map_glyph(parley: parley::Glyph) -> vello::Glyph {
    vello::Glyph {
        id: parley.id as u32,
        x: parley.x,
        y: parley.y
    }
}

// fn map_blend_mode(
//     blend: tataku::GraphicsPipeline,
// ) -> vello::peniko::BlendMode {
//     vello::peniko::BlendMode::default()
// }

fn map_rect([x, y, w, h]: [f32; 4]) -> vello::kurbo::Rect {
    let x = x as f64;
    let y = y as f64;

    vello::kurbo::Rect::new(
        x,
        y,
        x + w as f64,
        y + h as f64,
    )
}

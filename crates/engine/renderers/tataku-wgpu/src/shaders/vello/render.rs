use crate::prelude::*;
use vello::kurbo::Stroke;
use vello::peniko::{ Brush, Fill };
#[cfg(feature="vello_layers")] use layers::*;

pub(crate) struct RenderEngine<'window, 'a> {
    wgpu: &'a mut WgpuEngine<'window>,

    #[cfg(feature="vello_layers")] layers: Vec<RenderEngineLayer>,
    #[cfg(feature="vello_layers")] reset_pending: bool,
}
impl<'window, 'a> RenderEngine<'window, 'a> {
    pub fn new(wgpu: &'a mut WgpuEngine<'window>) -> Self {
        Self { 
            wgpu, 
            #[cfg(feature="vello_layers")] layers: vec![ RenderEngineLayer::default() ],
            #[cfg(feature="vello_layers")] reset_pending: false,
        }
    }

    
    #[cfg(not(feature="vello_layers"))]
    fn scene(
        &mut self, 
        _blend_mode: tataku::BlendMode
    ) -> Option<shaders::vello::ReserveData<'_>> {
        self.wgpu.reserve_vello()
    }
}

#[cfg(feature="vello_layers")]
impl<'window, 'a> RenderEngine<'window, 'a> {
    fn reset_layers(&mut self) {
        self.reset_pending = true;

        let reserve = self.wgpu.reserve_vello().unwrap();
        for _ in 0..self.layers.iter().filter(|l| !l.any_unset()).count() {
            reserve.scene.pop_layer();
        }
    }

    fn vello_layer(
        reserve: &mut shaders::vello::ReserveData,
        scissor: tataku::Scissor,
        blend_mode: tataku::BlendMode,
    ) {
        let scissor = scissor.unwrap_or([
            0.0, 0.0,
            5_000.0, 5_000.0
        ]);

        reserve.scene.push_layer(
            map_blend_mode(blend_mode),
            1.0,
            vello::kurbo::Affine::IDENTITY,
            &map_rect(scissor),
        );
    }

    fn check_layer(
        &mut self,
        blend_mode: tataku::BlendMode,
        scissor: tataku::Scissor,
    ) -> Option<shaders::vello::ReserveData<'_>> {
        let mut reserve = self.wgpu.reserve_vello()?;

        if self.reset_pending {
            self.reset_pending = false;
            for layer in self.layers.iter().filter(|l| !l.any_unset()) {
                Self::vello_layer(&mut reserve, layer.scissor.unwrap(), layer.blend_mode.unwrap());
            }
        }


        if self.layers.is_empty() {
            self.layers.push(RenderEngineLayer::default());
        }

        let last_layer = self.layers.last_mut().unwrap();
        let any_unset = last_layer.any_unset();
        if last_layer.check(blend_mode, scissor) {
            if any_unset {
                Self::vello_layer(&mut reserve, scissor, blend_mode);
            }
        } else {
            self.layers.push(RenderEngineLayer {
                blend_mode: LayerValue::Set(blend_mode),
                scissor: LayerValue::Set(scissor),
            });
            Self::vello_layer(&mut reserve, scissor, blend_mode);
        }

        Some(reserve)
    }

    fn scene(
        &mut self, 
        blend_mode: tataku::BlendMode
    ) -> Option<shaders::vello::ReserveData<'_>> {
        let scissor = self.wgpu.scissors.current_scissor();
        return self.check_layer(blend_mode, scissor);
    }
}

impl tataku_graphics::DrawEngine for RenderEngine<'_, '_> {
    fn push_scissor(&mut self, scissor: [f32; 4]) {
        self.wgpu.push_scissor(scissor);

        #[cfg(feature="vello_layers")] {
            if let Some(last) = self.layers.last_mut()
            && last.scissor.is_unset() {
                last.scissor = LayerValue::Set(Some(scissor));
                return;
            }
    
            self.layers.push(RenderEngineLayer {
                scissor: LayerValue::Set(Some(scissor)),
                blend_mode: LayerValue::Unset,
            });
        }
    }

    fn pop_scissor(&mut self) {
        self.wgpu.pop_scissor();
        
        #[cfg(feature="vello_layers")] {
            let Some(last) = self.layers.pop() 
            else { return };
            if !last.any_unset() {
                self.wgpu
                    .reserve_vello()
                    .unwrap()
                    .scene
                    .pop_layer();
            }
        }
    }

    fn draw_arc(
        &mut self,
        _start: f32,
        _end: f32,
        _radius: f32,
        _color: tataku::Color,
        _resolution: u32,
        _transform: tataku::Matrix,
        _blend_mode: tataku::BlendMode
    ) {
        
    }

    fn draw_circle(
        &mut self,
        radius: f32,
        color: tataku::Color,
        border: Option<tataku::Border>,
        _resolution: u32,
        transform: tataku::Matrix,
        blend_mode: tataku::BlendMode,
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
        blend_mode: tataku::BlendMode,
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
        blend_mode: tataku::BlendMode
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
        tex: tataku::TextureDraw,
        transform: tataku_engine::prelude::Matrix,
        blend_mode: tataku_engine::prelude::BlendMode
    ) {
        #[cfg(feature="vello_layers")] self.reset_layers();
        self.wgpu.draw_tex(
            tex,
            transform,
            blend_mode
        );
    }

    fn draw_text(
        &mut self,
        transform: tataku::Matrix,
        blend_mode: tataku::BlendMode,
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
        #[cfg(feature="vello_layers")] self.reset_layers();
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
        #[cfg(feature="vello_layers")] self.reset_layers();
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
        #[cfg(feature="vello_layers")] self.reset_layers();
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
        #[cfg(feature="vello_layers")] self.reset_layers();
        self.wgpu.draw_box_blur(bounds, size);
    }
}


use vello::peniko::color::{ AlphaColor, Srgb };

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

#[cfg(feature="vello_layers")]
fn map_blend_mode(
    blend: tataku::BlendMode,
) -> vello::peniko::BlendMode {
    use vello::peniko;

    // TODO: actually verify these
    match blend {
        tataku::BlendMode::AlphaBlending => peniko::BlendMode {
            mix: peniko::Mix::Normal,
            compose: peniko::Compose::SrcOver,
        },
        tataku::BlendMode::AlphaOverwrite => peniko::BlendMode {
            mix: peniko::Mix::Normal,
            compose: peniko::Compose::Copy,
        },

        _ => panic!("using {blend:?} for non-image!"),
        // tataku::BlendMode::PremultipliedAlpha => peniko::BlendMode {
        //     mix: peniko::Mix::Normal,
        //     compose: peniko::Compose::Plus,
        // },
        // tataku::BlendMode::AdditiveBlending => peniko::BlendMode {
        //     mix: peniko::Mix::Normal,
        //     compose: peniko::Compose::Plus,
        // },
        // tataku::BlendMode::SourceAlphaBlending => peniko::BlendMode {
        //     mix: peniko::Mix::Normal,
        //     compose: peniko::Compose::SrcOver,
        // },
        // tataku::BlendMode::OsuAdditiveBlending => peniko::BlendMode {
        //     mix: peniko::Mix::Normal,
        //     compose: peniko::Compose::SrcOver,
        // },
    }
}

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


#[cfg(feature="vello_layers")]
mod layers {
    #[derive(Default)]
    pub(super) struct RenderEngineLayer {
        pub blend_mode: LayerValue<tataku::BlendMode>,
        pub scissor: LayerValue<tataku::Scissor>,
    }
    impl RenderEngineLayer {
        // returns true if should continue, false if needs new layer
        pub fn check(
            &mut self, 
            blend_mode: tataku::BlendMode,
            scissor: tataku::Scissor,
        ) -> bool {
            if self.scissor.is_unset() {
                self.scissor = LayerValue::Set(scissor);
            } else if self.scissor != LayerValue::Set(scissor) {
                return false;
            }
    
            if self.blend_mode.is_unset() {
                self.blend_mode = LayerValue::Set(blend_mode);
            } else if self.blend_mode != LayerValue::Set(blend_mode) {
                return false;
            }
    
            true
        }
    
        pub fn any_unset(&self) -> bool {
            self.blend_mode.is_unset() || self.scissor.is_unset()
        }
    }
    
    /// Option<Option<...>> seemed unintuitive
    #[derive(Copy, Clone, Default, PartialEq, Eq)]
    pub(super) enum LayerValue<T> {
        Set(T),
        #[default]
        Unset,
    }
    impl<T> LayerValue<T> {
        const fn is_set(&self) -> bool {
            matches!(self, Self::Set(_))
        }
        const fn is_unset(&self) -> bool {
            matches!(self, Self::Unset)
        }
    
        fn unwrap(self) -> T {
            match self {
                Self::Set(v) => v,
                Self::Unset => panic!("trying to unwrap value from unset!"), 
            }
        }
    }
    impl<T> From<Option<T>> for LayerValue<T> {
        fn from(value: Option<T>) -> Self {
            match value {
                Some(v) => Self::Set(v),
                None => Self::Unset
            }
        }
    }

}

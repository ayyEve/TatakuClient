use crate::*;

#[derive(Clone, Default)]
pub struct RenderTarget {
    pub blend_mode: BlendMode,
    pub color: Color,
    pub pos: Vector2,
    pub scale: Vector2,
    pub rotation: f32,
    
    /// origin of rotation/scale in px, relative to image position
    /// 
    /// BEFORE SCALE
    pub origin: Vector2,

    pub flip: ImageFlip,

    pub data: Arc<RwLock<RenderTargetData>>,
    pub callback: Arc<Mutex<Option<Arc<dyn Fn(&mut dyn DrawEngine, Matrix) + Send + Sync>>>>,
}
impl RenderTarget {
    pub fn new(
        data: RenderTargetData,
        callback: impl Fn(&mut dyn DrawEngine, Matrix) + Send + Sync + 'static,
    ) -> Self {
        Self {

            data: Arc::new(RwLock::new(data)),
            callback: Arc::new(Mutex::new(Some(Arc::new(callback)))),
            ..Default::default()
        }
    }
    pub fn new_arced_callback(
        data: RenderTargetData,
        callback: Arc<dyn Fn(&mut dyn DrawEngine, Matrix) + Send + Sync>,
    ) -> Self {
        Self {

            data: Arc::new(RwLock::new(data)),
            callback: Arc::new(Mutex::new(Some(callback))),
            ..Default::default()
        }
    }

    pub fn as_image(&self) -> Image {
        Image {
            blend_mode: self.blend_mode,
            tex: self.data.read().tex.clone(),
            base_scale: Vector2::ONE,
            origin: self.origin,
            color: self.color,
            pos: self.pos,
            scale: self.scale,
            rotation: self.rotation,
            flip: self.flip,
            draw_debug: false,
        }
    }
    pub fn update(
        &self, 
        callback: impl Fn(&mut dyn DrawEngine, Matrix) + Send + Sync + 'static
    ) {
        self.update_arced(Arc::new(callback));
    }
    pub fn update_arced(
        &self, 
        callback: Arc<dyn Fn(&mut dyn DrawEngine, Matrix) + Send + Sync>
    ) {
        *self.callback.lock() = Some(callback);
    }
}

impl TatakuRenderable for RenderTarget {
    fn get_pipeline(&self) -> GraphicsPipeline {
        GraphicsPipeline::Standard(self.blend_mode)
    }

    fn set_pipeline(&mut self, pipeline: GraphicsPipeline) {
        let GraphicsPipeline::Standard(bm) = pipeline else { return };
        self.blend_mode = bm;
    }

    fn draw(
        &self, 
        options: &DrawOptions,
        transform: Matrix, 
        g: &mut dyn DrawEngine,
    ) {
        if self.data.read().tex.is_empty() {
            let mut data = self.data.write();
            let callback = self.callback
                .lock()
                .take()
                .unwrap_or_else(|| Arc::new(|_,_| {}));

            g.create_render_target(&mut data, callback);
        }
        if let Some(callback) = self.callback.lock().take() {
            g.update_render_target(&self.data.read(), callback);
        }

        self.as_image().draw(options, transform, g);
    }
}


#[derive(Clone, Default2)]
pub struct RenderTargetData {
    pub width: u32,
    pub height: u32,
    pub clear_color: Color,

    #[default(Matrix::identity())]
    pub projection: Matrix,
    pub tex: Arc<TextureReference>,
}
impl RenderTargetData {
    pub fn new(size: [u32; 2], clear_color: Color) -> Self {
        Self {
            width: size[0],
            height: size[1],
            clear_color,
            ..Self::default()
        }
    }
}

// #[cfg(feature = "graphics")]
// impl Drop for RenderTarget {
//     fn drop(&mut self) {
//         if self.image.reference_count() == 1 {
//             println!("dropping render target tex");
//             GameWindow::free_texture(*self.image.tex);
//         }
//     }
// }


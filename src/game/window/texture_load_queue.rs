use crate::prelude::*;
use image::RgbaImage;

use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};

pub static TEXTURE_LOAD_QUEUE: OnceCell<UnboundedSender<LoadImage>> = OnceCell::const_new();


pub async fn texture_load_loop() {
    let (texture_load_sender, mut texture_load_receiver) = unbounded_channel();
    TEXTURE_LOAD_QUEUE.set(texture_load_sender).ok().expect("bad");
    trace!("texture load queue set");

    // used to keep textures from dropping off the main thread
    let mut image_data = Vec::new();
    let mut render_targets = Vec::new();

    loop {
        if let Ok(method) = texture_load_receiver.try_recv() {
            let settings = opengl_graphics::TextureSettings::new();

            macro_rules! send_tex {
                ($tex:expr, $on_done:expr) => {{
                    let tex = Arc::new($tex);
                    image_data.push(tex.clone());
                    if let Err(_) = $on_done.send(Ok(tex)) {error!("uh oh")}
                }}
            }

            match method {
                LoadImage::GameClose => {
                    image_data.clear();
                    return;
                }
                LoadImage::Path(path, on_done) => {
                    match Texture::from_path(path, &settings) {
                        Ok(t) => send_tex!(t, on_done),
                        Err(e) => if let Err(_) = on_done.send(Err(TatakuError::String(e))) {error!("uh oh")},
                    };
                }
                LoadImage::Image(data, on_done) => {
                    send_tex!(Texture::from_image(&data, &settings), on_done)
                }
                LoadImage::Font(font, size, on_done) => {
                    let px = size.0;

                    // let mut textures = font.textures.write();
                    let mut characters = font.characters.write();

                    let count = font.font.chars().len();

                    println!("count: {count}");

                    if count < 1000 {

                        let count_width = count; //(count as f32 / 2.0).ceil() as usize; // keep as 1 row for now because this code sucks balls
                        
                        // generate all datas
                        #[derive(Default)]
                        struct RowData {
                            char_datas: Vec<(fontdue::Metrics, Vec<u8>, char)>,
                            max_height: usize,
                            total_width: usize
                        }
                        let mut rows = Vec::new();

                        for (n, (&char, _codepoint)) in font.font.chars().iter().enumerate() {
                            if n % count_width == 0 { rows.push(RowData::default()) }

                            // generate glyph data
                            let (metrics, bitmap) = font.font.rasterize(char, px);

                            let r = rows.last_mut().unwrap();
                            r.char_datas.push((metrics, bitmap, char));
                            r.max_height = r.max_height.max(metrics.height);
                            r.total_width += metrics.width;
                        }

                        let mut image_data:Vec<u8> = Vec::new();
                        let mut char_data = Vec::new(); // (pos, size)

                        let mut overall_width = 0;
                        let mut overall_height = 0;

                        for data in rows.into_iter() {
                            for y in 0..data.max_height {
                                let mut progressive_x = 0;

                                for (m, image, char) in data.char_datas.iter() {
                                    let y_start = y * m.width;
                                    let x_start = y_start + m.width;

                                    for i in y_start..x_start {
                                        image_data.push(255); // r
                                        image_data.push(255); // g
                                        image_data.push(255); // b
                                        image_data.push(*image.get(i).unwrap_or(&0)); // a
                                    }

                                    if y == 0 {
                                        char_data.push((
                                            Vector2::new(progressive_x as f64, overall_height as f64), 
                                            Vector2::new(m.width as f64, m.height as f64),
                                            *m,
                                            *char
                                        ));
                                    }

                                    progressive_x += m.width;
                                }
                            }

                            overall_width += data.total_width;
                            overall_height += data.max_height;
                        }

                        // pad with 0s until we get to the correct length
                        let required = overall_width * overall_height * 4;
                        for _ in 0..(required - image_data.len()) {
                            image_data.push(0);
                        }


                        let i = image::RgbaImage::from_vec(overall_width as u32, overall_height as u32, image_data).unwrap();
                        

                        let name = format!("{}.{}.png", font.font.file_hash(), size.0);
                        i.save_with_format(name, image::ImageFormat::Png).expect("pain");


                        let texture = Arc::new(Texture::from_image(&i, &settings));

                        for (pos, size2, metrics, char) in char_data {
                            
                            // setup data
                            let char_data = CharData {
                                texture: texture.clone(),
                                pos,
                                size: size2,
                                metrics,
                            };

                            // insert data
                            characters.insert((size, char), char_data);
                        }
                    } else {
                        for (&char, _codepoint) in font.font.chars() {
                            // TODO: load as one big image per-font
                            
                            // generate glyph data
                            let (metrics, bitmap) = font.font.rasterize(char, px);

                            // bitmap is a vec of grayscale pixels
                            // we need to turn that into rgba bytes
                            let mut data = Vec::new();
                            bitmap.into_iter().for_each(|gray| {
                                data.push(255); // r
                                data.push(255); // g
                                data.push(255); // b
                                data.push(gray); // a
                            });

                            // convert to image
                            let data = image::RgbaImage::from_vec(metrics.width as u32, metrics.height as u32, data).unwrap();

                            // load in opengl
                            let texture = Arc::new(Texture::from_image(&data, &settings));

                            // setup data
                            let char_data = CharData {
                                texture,
                                pos: Vector2::zero(),
                                size: Vector2::new(metrics.width as f64, metrics.height as f64),
                                metrics,
                            };

                            // insert data
                            characters.insert((size, char), char_data);
                        }
                    }


                    on_done.send(Err(TatakuError::String(String::new()))).ok().expect("uh oh");
                }

                LoadImage::RenderBuffer((w, h), on_done, callback) => {
                    match RenderTarget::new_main_thread(w, h) {
                        Ok(mut render_target) => {
                            callback(&mut render_target);
                            render_targets.push(render_target.render_target_data.clone());
                            
                            if let Err(_) = on_done.send(Ok(render_target)) { error!("uh oh") }
                        }
                        Err(e) => {
                            if let Err(_) = on_done.send(Err(e)) { error!("uh oh") }
                        }
                    }
                }
            }

            trace!("Done loading tex");
        }

        // drop textures that only have a reference here (ie, dropped everywhere else)
        image_data.retain(|i| Arc::strong_count(i) > 1);
        render_targets.retain(|i| Arc::strong_count(i) > 1);

        tokio::task::yield_now().await;
        // tokio::time::sleep(Duration::from_millis(10)).await;
    }
}



pub async fn load_texture<P: AsRef<Path>>(path: P) -> TatakuResult<Arc<Texture>> {
    let path = path.as_ref().to_string_lossy().to_string();
    trace!("loading tex {}", path);

    let (sender, mut receiver) = unbounded_channel();
    TEXTURE_LOAD_QUEUE.get().unwrap().send(LoadImage::Path(path, sender)).ok().expect("no?");

    if let Some(t) = receiver.recv().await {
        t
    } else {
        Err(TatakuError::String("idk".to_owned()))
    }
}

pub async fn load_texture_data(data: RgbaImage) -> TatakuResult<Arc<Texture>> {
    trace!("loading tex data");

    let (sender, mut receiver) = unbounded_channel();
    TEXTURE_LOAD_QUEUE.get().unwrap().send(LoadImage::Image(data, sender)).ok().expect("no?");

    if let Some(t) = receiver.recv().await {
        t
    } else {
        Err(TatakuError::String("idk".to_owned()))
    }
}

pub fn load_font_data(font: Font2, size:FontSize) -> TatakuResult<()> {
    // info!("loading font char ('{ch}',{size})");
    let (sender, mut receiver) = unbounded_channel();
    TEXTURE_LOAD_QUEUE.get().unwrap().send(LoadImage::Font(font, size, sender)).ok().expect("no?");

    loop {
        match receiver.try_recv() {
            Ok(_t) => {
                return Ok(())
            },
            Err(_) => {},
        }
    }
}


pub async fn create_render_target(size: (f64, f64), callback: fn(&mut RenderTarget)) -> TatakuResult<RenderTarget> {
    trace!("create render target");

    let (sender, mut receiver) = unbounded_channel();
    TEXTURE_LOAD_QUEUE.get().unwrap().send(LoadImage::RenderBuffer(size, sender, callback)).ok().expect("no?");

    if let Some(t) = receiver.recv().await {
        t
    } else {
        Err(TatakuError::String("idk".to_owned()))
    }
}

pub enum LoadImage {
    GameClose,
    Path(String, UnboundedSender<TatakuResult<Arc<Texture>>>),
    Image(RgbaImage, UnboundedSender<TatakuResult<Arc<Texture>>>),
    Font(Font2, FontSize, UnboundedSender<TatakuResult<()>>),

    RenderBuffer((f64, f64), UnboundedSender<TatakuResult<RenderTarget>>, fn(&mut RenderTarget))
}


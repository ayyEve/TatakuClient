use crate::prelude::*;
use image::RgbaImage;

use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};

pub type TextureLoadResult = (LoadImage, UnboundedSender<TatakuResult<Arc<Texture>>>);

pub static TEXTURE_LOAD_QUEUE: OnceCell<UnboundedSender<TextureLoadResult>> = OnceCell::const_new();


pub async fn texture_load_loop() {
    let (texture_load_sender, mut texture_load_receiver) = unbounded_channel();
    TEXTURE_LOAD_QUEUE.set(texture_load_sender).ok().expect("bad");
    trace!("texture load queue set");

    // used to keep textures from dropping off the main thread
    let mut image_data = Vec::new();

    loop {
        if let Ok((method, on_done)) = texture_load_receiver.try_recv() {
            let settings = opengl_graphics::TextureSettings::new();

            macro_rules! send_tex {
                ($tex:expr) => {{
                    let tex = Arc::new($tex);
                    image_data.push(tex.clone());
                    if let Err(_) = on_done.send(Ok(tex)) {error!("uh oh")}
                }}
            }

            match method {
                LoadImage::GameClose => {
                    image_data.clear();
                    return;
                }
                LoadImage::Path(path) => {
                    match Texture::from_path(path, &settings) {
                        Ok(t) => send_tex!(t),
                        Err(e) => if let Err(_) = on_done.send(Err(TatakuError::String(e))) {error!("uh oh")},
                    };
                },
                LoadImage::Image(data) => {
                    send_tex!(Texture::from_image(&data, &settings))
                },
                LoadImage::Font(font, size) => {
                    let px = size.0;

                    // let mut textures = font.textures.write();
                    let mut characters = font.characters.write();


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

                    on_done.send(Err(TatakuError::String(String::new()))).ok().expect("uh oh");
                }
            }

            trace!("Done loading tex");
        }

        // drop textures that only have a reference here (ie, dropped everywhere else)
        image_data.retain(|i| {
            Arc::strong_count(i) > 1
        });

        tokio::task::yield_now().await;
        // tokio::time::sleep(Duration::from_millis(10)).await;
    }
}



pub async fn load_texture<P: AsRef<Path>>(path: P) -> TatakuResult<Arc<Texture>> {
    let path = path.as_ref().to_string_lossy().to_string();
    trace!("loading tex {}", path);

    let (sender, mut receiver) = unbounded_channel();
    TEXTURE_LOAD_QUEUE.get().unwrap().send((LoadImage::Path(path), sender)).ok().expect("no?");

    if let Some(t) = receiver.recv().await {
        t
    } else {
        Err(TatakuError::String("idk".to_owned()))
    }
}

pub async fn load_texture_data(data: RgbaImage) -> TatakuResult<Arc<Texture>> {
    trace!("loading tex data");

    let (sender, mut receiver) = unbounded_channel();
    TEXTURE_LOAD_QUEUE.get().unwrap().send((LoadImage::Image(data), sender)).ok().expect("no?");

    if let Some(t) = receiver.recv().await {
        t
    } else {
        Err(TatakuError::String("idk".to_owned()))
    }
}

pub fn load_font_data(font: Font2, size:FontSize) -> TatakuResult<()> {
    // info!("loading font char ('{ch}',{size})");
    let (sender, mut receiver) = unbounded_channel();
    TEXTURE_LOAD_QUEUE.get().unwrap().send((LoadImage::Font(font, size), sender)).ok().expect("no?");

    loop {
        match receiver.try_recv() {
            Ok(t) => {
                return t.map(|_|());
            },
            Err(_) => {},
        }
    }

    // if let Some(t) = receiver.recv().await {
    //     
    // } else {
    //     Err(TatakuError::String("idk".to_owned()))
    // }
}


pub enum LoadImage {
    GameClose,
    Path(String),
    Image(RgbaImage),
    Font(Font2, FontSize),
}
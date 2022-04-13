use crate::prelude::*;
use image::RgbaImage;

use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};

pub type TextureLoadResult = (LoadImage, UnboundedSender<TatakuResult<Arc<Texture>>>);

pub static TEXTURE_LOAD_QUEUE: OnceCell<UnboundedSender<TextureLoadResult>> = OnceCell::const_new();


// pub async fn texture_load_loop(mut receiver: UnboundedReceiver<TextureLoadResult>) {
//     loop {
//         match receiver.recv().await {
//             Some(_) => todo!(),
//             None => todo!(),
//         }
//     }
// }



pub async fn load_texture<P: AsRef<Path>>(path: P) -> TatakuResult<Arc<Texture>> {
    let path = path.as_ref().to_string_lossy().to_string();
    info!("loading tex {}", path);

    let (sender, mut receiver) = unbounded_channel();
    TEXTURE_LOAD_QUEUE.get().unwrap().send((LoadImage::Path(path), sender)).ok().expect("no?");

    if let Some(t) = receiver.recv().await {
        t
    } else {
        Err(TatakuError::String("idk".to_owned()))
    }
}

pub async fn load_texture_data(data: RgbaImage) -> TatakuResult<Arc<Texture>> {
    info!("loading tex data");

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
    Path(String),
    Image(RgbaImage),
    Font(Font2, FontSize),
}
use crate::prelude::*;
use image::RgbaImage;
// use rectangle_pack::*;
// use std::{collections::BTreeMap, ffi::c_void};

use tokio::sync::mpsc::{ UnboundedSender, UnboundedReceiver, unbounded_channel };
const FONT_PADDING:u32 = 2;

pub static TEXTURE_LOAD_QUEUE: OnceCell<UnboundedSender<LoadImage>> = OnceCell::const_new();
static mut TEXTURE_LOAD_QUEUE_RECEIVER: OnceCell<UnboundedReceiver<LoadImage>> = OnceCell::const_new();

fn get_texture_load_queue<'a>() -> TatakuResult<&'a UnboundedSender<LoadImage>> {
    TEXTURE_LOAD_QUEUE.get().ok_or(TatakuError::String("texture load queue not set".to_owned()))
}

fn get_tex_load_receiver<'a>() -> TatakuResult<&'a mut UnboundedReceiver<LoadImage>> {
    unsafe{TEXTURE_LOAD_QUEUE_RECEIVER.get_mut()}.ok_or(TatakuError::String("texture load queue not set".to_owned()))
}

pub fn texture_load_init() {
    let (texture_load_sender, texture_load_receiver) = unbounded_channel();
    TEXTURE_LOAD_QUEUE.set(texture_load_sender).ok().expect("bad");
    unsafe{&mut TEXTURE_LOAD_QUEUE_RECEIVER}.set(texture_load_receiver).expect("bad2");
    trace!("texture load queue set");

    // used to keep textures from dropping off the main thread
    // let mut image_data = Vec::new();
    // let mut render_targets = Vec::new();

    // let mut texture_size = 0;
    // unsafe { gl::GetIntegerv(gl::MAX_TEXTURE_SIZE, &mut texture_size); }
    // let texture_size = texture_size.min(4096) as u32;

    // let mut font_bins = BTreeMap::new();
    // let mut font_textures = HashMap::new();

    // loop {
    //     if let Ok(method) = texture_load_receiver.try_recv() {
    //         // let settings = opengl_graphics::TextureSettings::new();

    //         macro_rules! send_tex {
    //             ($tex:expr, $on_done:expr) => {{
    //                 let tex = Arc::new($tex);
    //                 image_data.push(tex.clone());
    //                 if let Err(_) = $on_done.send(Ok(tex)) {error!("uh oh")}
    //             }}
    //         }

    //         match method {
    //             LoadImage::GameClose => {
    //                 image_data.clear();
    //                 font_textures.clear();
    //                 return;
    //             }
    //             LoadImage::Path(path, on_done) => {
    //                 match Texture::from_path(path, &settings) {
    //                     Ok(t) => send_tex!(t, on_done),
    //                     Err(e) => if let Err(_) = on_done.send(Err(TatakuError::String(e))) {error!("uh oh")},
    //                 };
    //             }
    //             LoadImage::Image(data, on_done) => {
    //                 send_tex!(Texture::from_image(&data, &settings), on_done)
    //             }
    //             LoadImage::Font(font, font_size, on_done) => {
    //                 let mut rects_to_place = GroupedRectsToPlace::<char>::new();
    //                 let mut char_data = HashMap::new();

    //                 for (&char, _codepoint) in font.font.chars() {
    //                     // generate glyph data
    //                     let (metrics, bitmap) = font.font.rasterize(char, font_size.0);

    //                     // bitmap is a vec of grayscale pixels
    //                     // we need to turn that into rgba bytes
    //                     // TODO: could reduce ram usage during font rasterization if this is moved to where the tex is actually loaded
    //                     let mut data = Vec::new();
    //                     bitmap.into_iter().for_each(|gray| {
    //                         data.push(255); // r
    //                         data.push(255); // g
    //                         data.push(255); // b
    //                         data.push(gray); // a
    //                     });

    //                     rects_to_place.push_rect(
    //                         char,
    //                         None,
    //                         RectToInsert::new(metrics.width as u32 + FONT_PADDING, metrics.height as u32 + FONT_PADDING, 1)
    //                     );

    //                     char_data.insert(char, (metrics, data));
    //                 }

    //                 let rect_info = loop {
    //                     let info = pack_rects(
    //                         &rects_to_place,
    //                         &mut font_bins,
    //                         &volume_heuristic,
    //                         &contains_smallest_box
    //                     );

    //                     match info {
    //                         Ok(info) => break info,
    //                         Err(_) => {
    //                             // insert new rect
    //                             let id = font_bins.len();
    //                             font_bins.insert(id, TargetBin::new(texture_size, texture_size, 1));

    //                             // make tex
    //                             let tex_data = image::RgbaImage::new(texture_size, texture_size);
    //                             let tex = Arc::new(Texture::from_image(&tex_data, &settings));
    //                             image_data.push(tex.clone());
    //                             font_textures.insert(id, tex);
    //                         }
    //                     }
    //                 };

    //                 let mut characters = font.characters.write();
    //                 for (char, (font_bin, data)) in rect_info.packed_locations().iter() {
    //                     let (metrics, char_data) = char_data.get(char).unwrap();
    //                     let texture = font_textures.get(font_bin).unwrap().clone();

    //                     let x = data.x();
    //                     let y = data.y();
    //                     let w = data.width() - FONT_PADDING;
    //                     let h = data.height() - FONT_PADDING;

    //                     let pos = Vector2::new(x as f64, y as f64);
    //                     let size = Vector2::new(w as f64, h as f64);

    //                     unsafe {
    //                         gl::TextureSubImage2D(
    //                             texture.get_id(),
    //                             0,
    //                             x as i32, y as i32,
    //                             w as i32, h as i32,
    //                             gl::BGRA,
    //                             gl::UNSIGNED_BYTE,
    //                             char_data.as_ptr() as *const c_void
    //                         );
    //                     }

                        
    //                     // setup data
    //                     let char_data = CharData {
    //                         texture,
    //                         pos,
    //                         size,
    //                         metrics: *metrics,
    //                     };

    //                     // insert data
    //                     characters.insert((font_size, *char), char_data);
    //                 }
                    
    //                 // make sure gl loads everything before the data vecs are dropped
    //                 unsafe {
    //                     gl::Flush();
    //                     gl::Finish();
    //                 }

    //                 trace!("done creating font");
    //                 font.loaded_sizes.write().insert(font_size.clone());
    //                 on_done.send(Ok(())).ok().expect("uh oh");
    //             }

    //             LoadImage::CreateRenderTarget((w, h), on_done, callback) => {
    //                 match RenderTarget::new_main_thread(w, h) {
    //                     Ok(mut render_target) => {
    //                         let graphics = graphics();
    //                         render_target.bind();
    //                         callback(&mut render_target, graphics);
    //                         render_target.unbind();

    //                         render_targets.push(render_target.render_target_data.clone());
    //                         image_data.push(render_target.image.tex.clone());

    //                         if let Err(_) = on_done.send(Ok(render_target)) { error!("uh oh") }
    //                     }
    //                     Err(e) => {
    //                         if let Err(_) = on_done.send(Err(e)) { error!("uh oh") }
    //                     }
    //                 }
    //             }

    //             LoadImage::UpdateRenderTarget(mut render_target, on_done, callback) => {
    //                 render_target.bind();
    //                 let graphics = graphics();
    //                 callback(&mut render_target, graphics);
    //                 render_target.unbind();

    //                 if let Err(_) = on_done.send(Ok(render_target)) { error!("uh oh") };
    //             }
    //         }

    //         trace!("Done loading tex");
    //     }

    //     // drop textures that only have a reference here (ie, dropped everywhere else)
    //     render_targets.retain(|i| Arc::strong_count(i) > 1);
    //     image_data.retain(|i| Arc::strong_count(i) > 1);

    //     tokio::task::yield_now().await;
    //     // tokio::time::sleep(Duration::from_millis(10)).await;
    // }
}

pub fn check_texture_load_loop(state: &mut GraphicsState) {
    if let Some(method) = get_tex_load_receiver().ok().and_then(|t|t.try_recv().ok())  {
        // let settings = opengl_graphics::TextureSettings::new();

        match method {
            LoadImage::GameClose => { return; }
            LoadImage::Path(path, on_done) => {
                on_done.send(state.load_texture_path(&path)).expect("poopy");
                // match Texture::from_path(path, &settings) {
                //     Ok(t) => send_tex!(t, on_done),
                //     Err(e) => if let Err(_) = on_done.send(Err(TatakuError::String(e))) {error!("uh oh")},
                // };
            }
            LoadImage::Image(data, on_done) => {
                on_done.send(state.load_texture_rgba(&data.to_vec(), data.width(), data.height())).expect("poopy");
                // send_tex!(Texture::from_image(&data, &settings), on_done)
            }
            LoadImage::Font(font, font_size, on_done) => {
                // let mut rects_to_place = GroupedRectsToPlace::<char>::new();
                // let mut char_data: HashMap<char, (fontdue::Metrics, Vec<u8>)> = HashMap::new();

                let font_size = FontSize::new(font_size);
                let mut char_data = Vec::new();
                
                for (&char, _codepoint) in font.font.chars() {
                    // generate glyph data
                    let (metrics, bitmap) = font.font.rasterize(char, font_size.f32());

                    // bitmap is a vec of grayscale pixels
                    // we need to turn that into rgba bytes
                    // TODO: could reduce ram usage during font rasterization if this is moved to where the tex is actually loaded
                    let mut data = Vec::with_capacity(bitmap.len() * 4);
                    bitmap.into_iter().for_each(|gray| {
                        data.push(255); // r
                        data.push(255); // g
                        data.push(255); // b
                        data.push(gray); // a
                    });
                    
                    char_data.push((char, data, metrics))
                    // let Ok(texture) = state.load_texture_rgba(&data, metrics.width as u32, metrics.height as u32) else {panic!("eve broke fonts")};
                    
                }

                if let Ok(datas) = state.load_texture_rgba_many(char_data.iter().map(|(_, data, m)|(data, m.width as u32, m.height as u32)).collect()) {
                    let mut characters = font.characters.write();
                    for (texture, (char, _, metrics)) in datas.into_iter().zip(char_data.into_iter()) {
                        // setup data
                        let char_data = CharData {
                            texture,
                            // atlas_data,
                            metrics,
                        };

                        // insert data
                        characters.insert((font_size.u32(), char), char_data);
                    }

                    font.loaded_sizes.write().insert(font_size.u32());
                } else {
                    panic!("no atlas space for font")
                }

                on_done.send(Ok(())).expect("uh oh");
            }

            // LoadImage::CreateRenderTarget((w, h), on_done, callback) => {
            //     // match RenderTarget::new_main_thread(w, h) {
            //     //     Ok(mut render_target) => {
            //     //         let graphics = graphics();
            //     //         render_target.bind();
            //     //         callback(&mut render_target, graphics);
            //     //         render_target.unbind();

            //     //         render_targets.push(render_target.render_target_data.clone());
            //     //         image_data.push(render_target.image.tex.clone());

            //     //         if let Err(_) = on_done.send(Ok(render_target)) { error!("uh oh") }
            //     //     }
            //     //     Err(e) => {
            //     //         if let Err(_) = on_done.send(Err(e)) { error!("uh oh") }
            //     //     }
            //     // }
            // }

            // LoadImage::UpdateRenderTarget(mut render_target, on_done, callback) => {
            //     // render_target.bind();
            //     // let graphics = graphics();
            //     // callback(&mut render_target, graphics);
            //     // render_target.unbind();

            //     // if let Err(_) = on_done.send(Ok(render_target)) { error!("uh oh") };
            // }
        }

        trace!("Done loading tex");
    }


}



pub async fn load_texture<P: AsRef<Path>>(path: P) -> TatakuResult<TextureReference> {
    let path = path.as_ref().to_string_lossy().to_string();
    trace!("loading tex {}", path);

    let (sender, mut receiver) = unbounded_channel();
    get_texture_load_queue().expect("no tex load queue").send(LoadImage::Path(path, sender)).ok().expect("no?");

    if let Some(t) = receiver.recv().await {
        t
    } else {
        Err(TatakuError::String("idk".to_owned()))
    }
}

pub async fn load_texture_data(data: RgbaImage) -> TatakuResult<TextureReference> {
    trace!("loading tex data");

    let (sender, mut receiver) = unbounded_channel();
    get_texture_load_queue().expect("no tex load queue").send(LoadImage::Image(data, sender)).ok().expect("no?");

    if let Some(t) = receiver.recv().await {
        t
    } else {
        Err(TatakuError::String("idk".to_owned()))
    }
}

pub fn load_font_data(font: Font, size:f32) -> TatakuResult<()> {
    // info!("loading font char ('{ch}',{size})");
    let (sender, mut receiver) = unbounded_channel();
    get_texture_load_queue().expect("no tex load queue").send(LoadImage::Font(font, size, sender)).ok().expect("no?");

    loop {
        match receiver.try_recv() {
            Ok(_t) => {
                return Ok(())
            },
            Err(_) => {},
        }
    }
}


// pub async fn create_render_target(size: (f64, f64), callback: impl FnOnce(&mut RenderTarget, &mut GlGraphics) + Send + 'static) -> TatakuResult<RenderTarget> {
//     trace!("create render target");

//     let (sender, mut receiver) = unbounded_channel();
//     get_texture_load_queue()?.send(LoadImage::CreateRenderTarget(size, sender, Box::new(callback))).ok().expect("no?");

//     if let Some(t) = receiver.recv().await {
//         t
//     } else {
//         Err(TatakuError::String("idk".to_owned()))
//     }
// }

// pub async fn update_render_target(rt:RenderTarget, callback: impl FnOnce(&mut RenderTarget, &mut GlGraphics) + Send + 'static) -> TatakuResult<RenderTarget> {
//     trace!("update render target");

//     let (sender, mut receiver) = unbounded_channel();
//     get_texture_load_queue()?.send(LoadImage::UpdateRenderTarget(rt, sender, Box::new(callback))).ok().expect("no?");

//     if let Some(t) = receiver.recv().await {
//         t
//     } else {
//         Err(TatakuError::String("idk".to_owned()))
//     }
// }



pub enum LoadImage {
    GameClose,
    Path(String, UnboundedSender<TatakuResult<TextureReference>>),
    Image(RgbaImage, UnboundedSender<TatakuResult<TextureReference>>),
    Font(Font, f32, UnboundedSender<TatakuResult<()>>),

    // CreateRenderTarget((f64, f64), UnboundedSender<TatakuResult<RenderTarget>>, Box<dyn FnOnce(&mut RenderTarget, &mut GlGraphics) + Send>),
    // UpdateRenderTarget(RenderTarget, UnboundedSender<TatakuResult<RenderTarget>>, Box<dyn FnOnce(&mut RenderTarget, &mut GlGraphics) + Send>),
}

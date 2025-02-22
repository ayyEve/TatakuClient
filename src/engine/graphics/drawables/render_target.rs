#![allow(unused)]
use graphics::Viewport;

use crate::prelude::*;

// yoinked form https://github.com/Furball-Engine/Furball.Vixie/blob/master/Furball.Vixie/Graphics/TextureRenderTarget.cs
use std::sync::atomic::{AtomicU32, Ordering};

lazy_static::lazy_static! {
    static ref CURRENT_BOUND:AtomicU32 = AtomicU32::new(0);
}

#[derive(Clone)]
pub struct RenderTarget {
    pub render_target_data: Arc<RenderTargetData>,

    /// When binding, it saves the old viewport here so it can reset it upon Unbinding
    old_view_port: [i32; 4],

    pub width: f64,
    pub height: f64,

    pub image: Image,
}
impl RenderTarget {
    pub async fn new(width: f64, height: f64, callback: impl FnOnce(&mut RenderTarget, &mut GlGraphics) + Send + 'static) -> TatakuResult<Self> {
        create_render_target((width, height), callback).await
    }

    pub fn new_main_thread(width: f64, height: f64) -> TatakuResult<Self> {

        // Generate and bind a FrameBuffer
        let framebuffer_id = unsafe {
            let mut ids = [0; 1];
            gl::GenFramebuffers(1, ids.as_mut_ptr());
            let id = ids[0];

            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, id);
            id
        };
        trace!("got framebuffer id: {}", framebuffer_id);

        // Generate a Texture
        let texture_id = unsafe {
            let mut ids = [0; 1];
            gl::GenTextures(1, ids.as_mut_ptr());
            let id = ids[0];
            
            gl::BindTexture(gl::TEXTURE_2D, id);
            // Set it to Empty and set parameters
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                width as i32,
                height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                std::ptr::null()
            );
            //Set The Filtering to nearest (apperantly necessary, idk)
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            id
        };
        trace!("got tex id: {}", texture_id);

        //Generate the Depth buffer
        let depth_renderbuffer_id = unsafe {
            let mut ids = [0; 1];
            gl::GenRenderbuffers(1, ids.as_mut_ptr());
            let id = ids[0];

            gl::BindRenderbuffer(gl::RENDERBUFFER, id);
            gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH_COMPONENT, width as i32, height as i32);
            gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::RENDERBUFFER, id);
            //Connect the bound texture to the FrameBuffer object
            gl::FramebufferTexture(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, texture_id, 0);

            let draw_buffers = [gl::COLOR_ATTACHMENT0];
            gl::DrawBuffers(1, draw_buffers.as_ptr());

            id
        };
        trace!("got depth_renderbuffer_id: {}", depth_renderbuffer_id);

        //Check if FrameBuffer created successfully, and unbind it
        unsafe {
            if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
                let err = gl::GetError();
                error!("error creating render target: {err}");

                gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
                
                return Err(TatakuError::GlError(GlError::RenderBuffer))
            }
            
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }


        let old_view_port = [0; 4];

        let image = Image::new(
            Vector2::ZERO,
            -99999999999999990.0, 
            Arc::new(Texture::new(texture_id, width as u32, height as u32)),
            Vector2::ONE
        );

        let data = Arc::new(RenderTargetData {
            framebuffer_id,
            texture_id,
            depth_renderbuffer_id,
        });

        Ok(Self {
            height,
            width,
            render_target_data: data,
            old_view_port,

            image,
        })
    }

    pub fn bind(&mut self) {
        trace!("binding");
        CURRENT_BOUND.store(self.render_target_data.framebuffer_id, Ordering::SeqCst);
        trace!("{}", self.render_target_data.framebuffer_id);

        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.render_target_data.framebuffer_id);
            gl::GetIntegerv(gl::VIEWPORT, self.old_view_port.as_mut_ptr());
            gl::Viewport(0, 0, self.width as i32, self.height as i32);
        }
    }

    pub fn unbind(&mut self) {
        trace!("unbinding");
        CURRENT_BOUND.store(0, Ordering::SeqCst);

        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            let [x, y, width, height] = self.old_view_port;
            gl::Viewport(x, y, width, height);
        }
    }

    pub async fn update(&mut self, callback: impl FnOnce(&mut RenderTarget, &mut GlGraphics) + Send + 'static) {
        let t = self.clone();
        if let Ok(t) = update_render_target(t, callback).await {
            *self = t;
        } else {
            error!("error updating render target")
        }
    }

    pub fn viewport(&self) -> Viewport {
        Viewport {
            rect: [0, 0, self.width as i32, self.height as i32],
            draw_size: [self.width as u32, self.height as u32],
            window_size: [self.width, self.height],
        }
    }
}


#[derive(Clone)]
pub struct RenderTargetData {
    /// Unique ID of this FrameBuffer
    framebuffer_id: u32,

    /// Texture ID of the Texture that this RenderTarget draws to
    texture_id: u32,

    /// Depth Buffer of this RenderTarget
    depth_renderbuffer_id: u32,
}

impl Drop for RenderTargetData {
    fn drop(&mut self) {
        let current_bound = CURRENT_BOUND.load(Ordering::SeqCst);
        if self.framebuffer_id == current_bound {
            warn!("dropping render target while active");

            CURRENT_BOUND.store(0, Ordering::SeqCst);

            unsafe {
                gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
                // rip viewport
            }
        }

        unsafe {
            gl::DeleteFramebuffers(1, [self.framebuffer_id].as_ptr());
            // gl::DeleteTextures(1, [self.texture_id].as_ptr()); // let the image drop do this
            gl::DeleteRenderbuffers(1, [self.depth_renderbuffer_id].as_ptr());
        }
    }
}

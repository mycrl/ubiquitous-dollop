use std::sync::{atomic::AtomicU32, Arc, Mutex};

use anyhow::Result;
use pixels::{wgpu::TextureFormat, Pixels, PixelsBuilder, SurfaceTexture};
use winit::{
    event::{Event, WindowEvent},
    window::Window,
};

use crate::{utils::EasyAtomic, CustomEvent};

#[derive(Clone, Copy)]
pub struct TextureSize {
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy)]
pub struct TexturePosition {
    pub x: u32,
    pub y: u32,
}

#[derive(Default)]
struct AtomicTextureSize {
    width: AtomicU32,
    height: AtomicU32,
}

pub struct Render {
    pixels: Mutex<Pixels>,
    size: AtomicTextureSize,
}

impl Render {
    pub fn new(window: &Window) -> Result<Arc<Self>> {
        let size = window.inner_size();
        let pixels = PixelsBuilder::new(
            size.width,
            size.height,
            SurfaceTexture::new(size.width, size.height, window),
        )
        .texture_format(TextureFormat::Bgra8UnormSrgb)
        .build()?;

        Ok(Arc::new(Self {
            pixels: Mutex::new(pixels),
            size: AtomicTextureSize {
                width: AtomicU32::new(size.width),
                height: AtomicU32::new(size.height),
            },
        }))
    }

    pub fn resize(&self, width: u32, height: u32) -> Result<()> {
        self.pixels.lock().unwrap().resize_surface(width, height)?;
        Ok(())
    }

    pub fn input_texture(&self, texture: &[u8], size: TextureSize, position: TexturePosition) {
        if position.x >= size.width || position.y >= size.height {
            return;
        }

        let mut pixels = self.pixels.lock().unwrap();
        if size.width != self.size.width.get() || size.height != self.size.height.get() {
            if pixels.resize_buffer(size.width, size.height).is_err() {
                return;
            }

            self.size.width.set(size.width);
            self.size.height.set(size.height);
        }

        let frame = pixels.frame_mut();
        frame.copy_from_slice(texture);
    }

    pub fn redraw(&self) -> Result<()> {
        self.pixels.lock().unwrap().render()?;
        Ok(())
    }

    pub fn input(&self, events: &Event<CustomEvent>) {
        match events {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(size) => {
                    let _ = self.resize(size.width, size.height);
                }
                _ => (),
            },
            _ => (),
        }
    }
}

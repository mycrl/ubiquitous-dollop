use std::sync::{Arc, Mutex};

use anyhow::Result;
use pixels::{wgpu::TextureFormat, Pixels, PixelsBuilder, SurfaceTexture};
use winit::window::Window;

pub struct Render {
    pixels: Mutex<Pixels>,
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
        }))
    }

    pub fn resize(&self, width: u32, height: u32) -> Result<()> {
        self.pixels.lock().unwrap().resize_surface(width, height)?;
        Ok(())
    }

    pub fn input_webview_texture(&self, texture: &[u8], _width: u32, _height: u32) {
        self.pixels
            .lock()
            .unwrap()
            .frame_mut()
            .copy_from_slice(texture);

        self.redraw().unwrap();
    }

    pub fn redraw(&self) -> Result<()> {
        self.pixels.lock().unwrap().render()?;
        Ok(())
    }
}

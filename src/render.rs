use anyhow::Result;
use pixels::{Pixels, SurfaceTexture};
use winit::window::Window;

pub struct Render {
    pixels: Pixels,
}

impl Render {
    pub fn new(window: &Window) -> Result<Self> {
        let size = window.inner_size();
        Ok(Self {
            pixels: Pixels::new(
                size.width,
                size.height,
                SurfaceTexture::new(size.width, size.height, window),
            )?,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        self.pixels.resize_surface(width, height)?;
        Ok(())
    }

    pub fn redraw(&mut self, r: u8, g: u8, b: u8) -> Result<()> {
        let frame = self.pixels.frame_mut();
        for pixel in frame.chunks_exact_mut(4) {
            pixel[0] = r;
            pixel[1] = g;
            pixel[2] = b;
            pixel[3] = 255;
        }

        self.pixels.render()?;
        Ok(())
    }
}

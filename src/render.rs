use std::sync::Mutex;

use anyhow::{anyhow, Result};
use wgpu::{
    CompositeAlphaMode, Device, Extent3d, Instance, PresentMode, Queue, Surface,
    SurfaceConfiguration, Texture, TextureDescriptor, TextureDimension, TextureFormat,
    TextureUsages,
};
use winit::window::Window;

pub struct Render {
    surface: Surface,
    queue: Queue,
    device: Device,
    config: Mutex<SurfaceConfiguration>,
}

impl Render {
    pub async fn new(window: &Window) -> Result<Self> {
        let size = window.inner_size();
        let instance = Instance::default();
        let surface = unsafe { instance.create_surface(window) }?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow!("crate adapter failed!"))?;
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await?;
        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: TextureFormat::Rgba8Snorm,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Opaque,
            view_formats: vec![TextureFormat::Rgba8Snorm],
        };

        surface.configure(&device, &config);
        Ok(Self {
            config: Mutex::new(config),
            surface,
            queue,
            device,
        })
    }

    pub fn resize(&self, width: u32, height: u32) {
        let mut config = self.config.lock().unwrap();
        config.width = width;
        config.height = height;
        self.surface.configure(&self.device, &config);
    }

    pub fn redraw(&mut self) -> Result<()> {
        Ok(())
    }
}

pub fn create_texture(label: &str, device: &Device, config: &SurfaceConfiguration) -> Texture {
    device.create_texture(&TextureDescriptor {
        label: Some(label),
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::R8Unorm,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        view_formats: &config.view_formats,
        size: Extent3d {
            depth_or_array_layers: 1,
            width: config.width,
            height: config.height,
        },
    })
}

use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Result};
use winit::window::Window;
use wgpu::*;

pub struct Render {
    surface: Surface,
    queue: Queue,
    device: Device,
    config: Mutex<SurfaceConfiguration>,
    textures: [Texture; 4],
}

impl Render {
    pub async fn new(window: &Window) -> Result<Arc<Self>> {
        let size = window.inner_size();
        let instance = Instance::default();
        let surface = unsafe { instance.create_surface(window) }?;
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow!("crate adapter failed!"))?;
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    features: Features::empty(),
                    limits: Limits::default(),
                    label: None,
                },
                None,
            )
            .await?;
        let caps = surface.get_capabilities(&adapter);
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: caps.formats[0],
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Opaque,
            view_formats: vec![],
        };

        let textures = [
            create_texture("R", &device, &config),
            create_texture("G", &device, &config),
            create_texture("B", &device, &config),
            create_texture("A", &device, &config),
        ];

        surface.configure(&device, &config);
        Ok(Arc::new(Self {
            config: Mutex::new(config),
            queue,
            device,
            surface,
            textures,
        }))
    }

    pub fn resize(&self, width: u32, height: u32) {
        let mut config = self.config.lock().unwrap();
        config.width = width;
        config.height = height;
        self.surface.configure(&self.device, &config);
    }

    pub fn input_webview_texture(&self, texture: &[u8], width: u32, height: u32) {
        self.queue.write_texture(
            ImageCopyTexture {
                texture: &self.textures[0],
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            texture,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
    }

    pub fn redraw(&self) -> Result<()> {
        let frame = self.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: None,
            });
        
        {
            let _ = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        store: true,
                        load: LoadOp::Clear(Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });
        }
            
        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
        Ok(())
    }
}

pub fn create_texture(label: &str, device: &Device, config: &SurfaceConfiguration) -> Texture {
    device.create_texture(&TextureDescriptor {
        label: Some(label),
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Bgra8UnormSrgb,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        view_formats: &vec![],
        size: Extent3d {
            depth_or_array_layers: 1,
            width: config.width,
            height: config.height,
        },
    })
}

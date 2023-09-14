mod render;
mod view;

use std::sync::Arc;

use render::Render;
use tokio::runtime::Runtime;
use view::{Webview, WebviewInfo};
use webview::{execute_subprocess, is_subprocess};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() -> anyhow::Result<()> {
    if is_subprocess() {
        execute_subprocess();
    }

    let runtime = Runtime::new()?;
    let event_loop = EventLoop::new();
    let window = Arc::new(
        WindowBuilder::new()
            .with_min_inner_size(PhysicalSize {
                width: 1280,
                height: 720,
            })
            .build(&event_loop)?,
    );

    let size = window.inner_size();
    let render = Render::new(&window)?;
    let webview = runtime.block_on(async {
        Webview::new(
            WebviewInfo {
                url: "https://google.com",
                width: size.width,
                height: size.height,
            },
            render.clone(),
        )
        .await
    })?;

    window.set_ime_allowed(true);
    event_loop.run(move |event, _| match event {
        Event::WindowEvent { event, .. } => {
            webview.input(event, &window);
            match event {
                WindowEvent::Resized(size) => {
                    render.resize(size.width, size.height).unwrap();
                }
                _ => (),
            }
        }
        Event::RedrawRequested(_) => {
            render.redraw().unwrap();
        }
        Event::MainEventsCleared => {
            window.request_redraw();
        }
        _ => (),
    });
}

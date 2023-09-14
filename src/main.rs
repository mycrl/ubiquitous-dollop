mod render;
mod view;

use std::{sync::Arc, thread, time::Duration};

use render::Render;
use tokio::runtime::Runtime;
use view::{Webview, WebviewInfo};
use webview::{execute_subprocess, is_subprocess};
use winit::{
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
    let window = Arc::new(WindowBuilder::new().build(&event_loop)?);
    let render = runtime.block_on(async { Render::new(&window).await })?;

    let size = window.inner_size();
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

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                render.resize(size.width, size.height);
                webview.resize(size.width, size.height);
            }
            Event::RedrawRequested(_) => {
                render.redraw().unwrap();
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => (),
        }
    });
}

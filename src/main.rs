mod render;
mod view;

use std::{sync::Arc, thread, time::Duration};

use render::Render;
use tokio::runtime::Runtime;
use view::{CustomEvent, Webview};
use webview::{execute_subprocess, is_subprocess};
use winit::{
    dpi::{LogicalPosition, LogicalSize, PhysicalSize},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::{Fullscreen, WindowBuilder},
};

fn main() -> anyhow::Result<()> {
    if is_subprocess() {
        execute_subprocess();
    }

    let runtime = Runtime::new()?;
    let event_loop = EventLoopBuilder::<CustomEvent>::with_user_event().build()?;
    let window = Arc::new(
        WindowBuilder::new()
            .with_min_inner_size(PhysicalSize {
                width: 1280,
                height: 720,
            })
            .build(&event_loop)?,
    );

    let render = Render::new(&window)?;
    let webview = runtime.block_on(async {
        Webview::new(
            "https://baidu.com",
            render.clone(),
            window.clone(),
            event_loop.create_proxy(),
        )
        .await
    })?;

    let window_ = window.clone();
    thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(1000 / 60));
        window_.request_redraw();
    });

    window.set_ime_allowed(true);
    window.set_resizable(true);

    event_loop.run(move |event, _, control_flow| match event {
        Event::UserEvent(event) => match event {
            CustomEvent::ImeRect(rect) => {
                window.set_ime_cursor_area(
                    LogicalPosition::new(rect.x + rect.width, rect.y + rect.height),
                    LogicalSize::new(rect.width, rect.height),
                );
            }
            CustomEvent::FullscreenChange(fullscreen) => {
                window.set_fullscreen(if fullscreen {
                    Some(Fullscreen::Borderless(None))
                } else {
                    None
                });
            }
            CustomEvent::TitleChange(title) => {
                window.set_title(&title);
            }
        },
        Event::WindowEvent { event, .. } => {
            webview.input(event.clone(), &window);
            match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::Resized(size) => {
                    render.resize(size.width, size.height).unwrap();
                }
                _ => (),
            }
        }
        Event::RedrawRequested(_) => {
            render.redraw().unwrap();
        }
        _ => (),
    })?;

    Ok(())
}

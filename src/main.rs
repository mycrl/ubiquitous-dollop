mod config;
mod render;
mod rtc;
mod view;
mod utils;

use std::sync::Arc;

use config::Config;
use dotenv::dotenv;
use librtc::RTCConfiguration;
use render::Render;
use rtc::Rtc;
use tokio::runtime::Runtime;
use view::Webview;
use webview::{execute_subprocess, is_subprocess, Rect};
use winit::{
    dpi::{LogicalPosition, LogicalSize, PhysicalSize},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::{Fullscreen, WindowBuilder},
};

pub enum CustomEvent {
    ImeRect(Rect),
    TitleChange(String),
    FullscreenChange(bool),
    Closed,
}

fn main() -> anyhow::Result<()> {
    let _ = dotenv();
    if is_subprocess() {
        execute_subprocess();
    }

    let config = Config::new();
    let runtime = Runtime::new()?;
    let event_loop = EventLoopBuilder::<CustomEvent>::with_user_event().build()?;
    let window = Arc::new(
        WindowBuilder::new()
            .with_min_inner_size(PhysicalSize {
                width: 800,
                height: 600,
            })
            .build(&event_loop)?,
    );

    window.set_ime_allowed(true);
    window.set_resizable(true);

    let render = Render::new(&window)?;
    let webview = runtime.block_on(async {
        Webview::new(
            config.clone(),
            render.clone(),
            window.clone(),
            event_loop.create_proxy(),
        )
        .await
    })?;

    // let rtc = Rtc::new(&RTCConfiguration {
    //     ..Default::default()
    // })?;

    let webview_ = webview.clone();
    let event_proxy = event_loop.create_proxy();
    runtime.spawn(async move {
        webview_.closed().await;
        let _ = event_proxy.send_event(CustomEvent::Closed);
    });

    event_loop.run(move |event, _, control_flow| {
        webview.input(&event, &window);
        render.input(&event);

        match event {
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
                CustomEvent::Closed => {
                    *control_flow = ControlFlow::Exit;
                }
            },
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => (),
            },
            Event::RedrawRequested(_) => {
                window.pre_present_notify();
                render.redraw().unwrap();
            }
            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => (),
        }
    })?;

    Ok(())
}

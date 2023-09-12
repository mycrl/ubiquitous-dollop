mod render;

use std::{thread, time::Duration};

use render::Render;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() -> anyhow::Result<()> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop)?;
    // let mut render = Render::new(&window)?;

    let mut rgb = [0u8; 3];
    thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(1000 / 60));
        window.request_redraw();
    });

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
                // if render.resize(size.width, size.height).is_err() {
                //     *control_flow = ControlFlow::Exit;
                // }
            }
            Event::RedrawRequested(_) => {
                for item in &mut rgb {
                    if item == &255 {
                        *item = 0;
                    } else {
                        *item += 1;
                    }
                }

                // if render.redraw().is_err() {
                //     *control_flow = ControlFlow::Exit;
                // }
            }
            _ => (),
        }
    });
}

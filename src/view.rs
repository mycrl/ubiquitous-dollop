use std::{
    cell::Cell,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use anyhow::Result;
use webview::{
    ActionState, App, AppSettings, Browser, BrowserSettings, ImeAction, Modifiers, MouseAction,
    MouseButtons, Observer, Position,
};
use winit::{
    event::{ElementState, Ime, MouseButton, MouseScrollDelta, WindowEvent, VirtualKeyCode, ModifiersState},
    window::Window,
};

use crate::render::Render;

pub struct WebviewInfo<'a> {
    pub url: &'a str,
    pub width: u32,
    pub height: u32,
}

struct WebviewObserver {
    render: Arc<Render>,
}

impl Observer for WebviewObserver {
    fn on_frame(&self, texture: &[u8], width: u32, height: u32) {
        self.render.input_webview_texture(texture, width, height);
    }
}

pub struct Webview {
    #[allow(unused)]
    app: Arc<App>,
    browser: Arc<Browser>,
    ime_enabled: AtomicBool,
    modifiers: Cell<Modifiers>,
}

impl Webview {
    pub async fn new(info: WebviewInfo<'_>, render: Arc<Render>) -> Result<Self> {
        let app = App::new(&AppSettings {
            cache_path: None,
            browser_subprocess_path: None,
            scheme_path: None,
        })
        .await?;

        let browser = app
            .create_browser(
                &BrowserSettings {
                    url: info.url,
                    window_handle: webview::HWND::default(),
                    frame_rate: 60,
                    width: info.width,
                    height: info.height,
                    device_scale_factor: 1.0,
                    is_offscreen: true,
                },
                WebviewObserver { render },
            )
            .await?;

        Ok(Self {
            modifiers: Cell::new(Modifiers::None),
            ime_enabled: AtomicBool::new(false),
            browser,
            app,
        })
    }

    pub fn input(&self, events: WindowEvent, window: &Window) {
        match events {
            WindowEvent::Resized(size) => self.browser.resize(size.width, size.height),
            WindowEvent::Ime(ime) => {
                match ime {
                    Ime::Enabled => {
                        self.ime_enabled.set(true);

                        // Because the keyboard input event is triggered before the ime event, a
                        // character entered because the keyboard input event is triggered is deleted first.
                        //
                        // scan_code: 14 == delete
                        self.browser
                            .on_keyboard(14, ActionState::Down, Modifiers::None);
                        self.browser
                            .on_keyboard(14, ActionState::Up, Modifiers::None);
                    }
                    Ime::Disabled => {
                        self.ime_enabled.set(false);
                    }
                    Ime::Commit(input) => {
                        self.browser.on_ime(ImeAction::Composition(input.as_str()));
                    }
                    Ime::Preedit(input, pos) => {
                        if let Some((x, y)) = pos {
                            self.browser
                                .on_ime(ImeAction::Pre(input.as_str(), x as i32, y as i32));
                        }
                    }
                }
            }
            WindowEvent::KeyboardInput { input, .. } => {
                if let Some(key) = input.virtual_keycode {
                    if !self.ime_enabled.get() {
                        let allow = if input.repeat {
                            !(key == VirtualKeyCode::Shift || key == VirtualKeyCode::Control || key == VirtualKeyCode::At)
                        } else {
                            true
                        };

                        if allow {
                            let modifiers = self.modifiers.get();
                            self.browser.on_keyboard(
                                input.scancode,
                                match input.state {
                                    winit::event::ElementState::Pressed => ActionState::Down,
                                    winit::event::ElementState::Released => ActionState::Up,
                                },
                                modifiers,
                            );
                        }
                    }
                }
            }
            WindowEvent::ModifiersChanged(state) => {
                self.modifiers.set(if state.is_empty() {
                    Modifiers::None
                } else {
                    match state {
                        ModifiersState::ALT => Modifiers::Alt,
                        ModifiersState::CTRL => Modifiers::Ctrl,
                        ModifiersState::SHIFT => Modifiers::Shift,
                        ModifiersState::LOGO => Modifiers::Win,
                        _ => Modifiers::None,
                    }
                });
            }
            WindowEvent::MouseWheel {
                delta, phase: _, ..
            } => match delta {
                MouseScrollDelta::LineDelta(delta_h, delta_v) => {
                    self.browser.on_mouse(MouseAction::Wheel(Position {
                        x: delta_h as i32,
                        y: (delta_v * 24.) as i32,
                    }));
                }
                _ => (),
            },
            WindowEvent::MouseInput { state, button, .. } => {
                self.browser.on_mouse(MouseAction::Click(
                    match button {
                        MouseButton::Left => MouseButtons::Left,
                        MouseButton::Right => MouseButtons::Right,
                        MouseButton::Middle => MouseButtons::Middle,
                        _ => MouseButtons::Left,
                    },
                    match state {
                        ElementState::Pressed => ActionState::Down,
                        ElementState::Released => ActionState::Up,
                    },
                    None,
                ));
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.browser.on_mouse(MouseAction::Move(Position {
                    x: position.x as i32,
                    y: position.y as i32,
                }));
            }
            _ => (),
        }
    }
}

trait EasyAtomic {
    fn get(&self) -> bool;
    fn set(&self, value: bool);
}

impl EasyAtomic for AtomicBool {
    fn get(&self) -> bool {
        self.load(Ordering::Relaxed)
    }

    fn set(&self, value: bool) {
        self.store(value, Ordering::Relaxed)
    }
}

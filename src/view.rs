use crate::render::Render;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, RwLock,
};

use anyhow::Result;
use webview::{
    ActionState, App, AppSettings, Browser, BrowserSettings, ImeAction, Modifiers, MouseAction,
    MouseButtons, Observer, Position, Rect, HWND,
};

use winit::{
    event::{ElementState, Ime, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::EventLoopProxy,
    keyboard::{Key, KeyCode, ModifiersState},
    platform::scancode::KeyCodeExtScancode,
    window::{
        raw_window_handle::{HasRawWindowHandle, RawWindowHandle},
        Window,
    },
};

pub enum CustomEvent {
    ImeRect(Rect),
    TitleChange(String),
    FullscreenChange(bool),
    Closed,
}

struct WebviewObserver {
    render: Arc<Render>,
    event_proxy: EventLoopProxy<CustomEvent>,
}

impl Observer for WebviewObserver {
    fn on_frame(&self, texture: &[u8], width: u32, height: u32) {
        self.render.input_webview_texture(texture, width, height);
    }

    fn on_ime_rect(&self, rect: Rect) {
        let _ = self.event_proxy.send_event(CustomEvent::ImeRect(rect));
    }

    fn on_fullscreen_change(&self, fullscreen: bool) {
        let _ = self
            .event_proxy
            .send_event(CustomEvent::FullscreenChange(fullscreen));
    }

    fn on_title_change(&self, title: String) {
        let _ = self.event_proxy.send_event(CustomEvent::TitleChange(title));
    }
}

pub struct Webview {
    app: Arc<App>,
    browser: Arc<Browser>,
    ime_enabled: AtomicBool,
    modifiers: RwLock<Modifiers>,
}

impl Webview {
    pub async fn new(
        url: &str,
        render: Arc<Render>,
        window: Arc<Window>,
        event_proxy: EventLoopProxy<CustomEvent>,
    ) -> Result<Arc<Self>> {
        let app = App::new(&AppSettings {
            cache_path: None,
            browser_subprocess_path: None,
            scheme_path: None,
        })
        .await?;

        let window_handle = match window.raw_window_handle() {
            RawWindowHandle::Win32(handle) => HWND(handle.hwnd),
            RawWindowHandle::WinRt(handle) => HWND(handle.core_window),
            _ => HWND::default(),
        };

        let size = window.inner_size();
        let browser = app
            .create_browser(
                &BrowserSettings {
                    device_scale_factor: 1.0,
                    is_offscreen: true,
                    height: size.height,
                    width: size.width,
                    frame_rate: 60,
                    window_handle,
                    url,
                },
                WebviewObserver {
                    render,
                    event_proxy,
                },
            )
            .await?;

        Ok(Arc::new(Self {
            modifiers: RwLock::new(Modifiers::None),
            ime_enabled: AtomicBool::new(false),
            browser,
            app,
        }))
    }

    pub fn input(&self, events: WindowEvent, _window: &Window) {
        match events {
            WindowEvent::Resized(size) => self.browser.resize(size.width, size.height),
            WindowEvent::Ime(ime) => {
                match ime {
                    Ime::Enabled => {
                        self.ime_enabled.set(true);

                        // Because the keyboard input event is triggered before the ime event, a
                        // character entered because the keyboard input event is triggered is deleted first.
                        for state in [ActionState::Down, ActionState::Up] {
                            self.browser.on_keyboard(
                                KeyCode::Backspace.to_scancode().unwrap(),
                                state,
                                Modifiers::None,
                            );
                        }
                    }
                    Ime::Disabled => {
                        self.ime_enabled.set(false);
                    }
                    Ime::Commit(input) => {
                        self.browser.on_ime(ImeAction::Composition(&input));
                    }
                    Ime::Preedit(input, pos) => {
                        if let Some((x, y)) = pos {
                            self.browser
                                .on_ime(ImeAction::Pre(&input, x as i32, y as i32));
                        }
                    }
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if !self.ime_enabled.get() {
                    let allow = if event.repeat {
                        !(event.logical_key == Key::Shift
                            || event.logical_key == Key::Control
                            || event.logical_key == Key::Alt)
                    } else {
                        true
                    };

                    if allow {
                        if let Some(code) = event.physical_key.to_scancode() {
                            let modifiers = *self.modifiers.read().unwrap();
                            self.browser.on_keyboard(
                                code,
                                match event.state {
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
                *self.modifiers.write().unwrap() = match state.state() {
                    ModifiersState::ALT => Modifiers::Alt,
                    ModifiersState::CONTROL => Modifiers::Ctrl,
                    ModifiersState::SHIFT => Modifiers::Shift,
                    ModifiersState::SUPER => Modifiers::Win,
                    _ => Modifiers::None,
                };
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

    pub async fn closed(&self) {
        self.app.closed().await
    }
}

trait EasyAtomic {
    type Item;

    fn get(&self) -> Self::Item;
    fn set(&self, value: Self::Item);
}

impl EasyAtomic for AtomicBool {
    type Item = bool;

    fn get(&self) -> Self::Item {
        self.load(Ordering::Relaxed)
    }

    fn set(&self, value: Self::Item) {
        self.store(value, Ordering::Relaxed)
    }
}

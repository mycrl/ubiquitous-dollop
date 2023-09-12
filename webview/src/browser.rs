use std::{ffi::{c_char, c_float, c_int, c_void}, sync::Arc};

use anyhow::Result;

use crate::{app::RawApp, ptr::AsCStr};

#[repr(C)]
pub enum BrowserState {
    Load = 1,
    LoadError = 2,
    BeforeLoad = 3,
    BeforeClose = 4,
    Close = 5,
}

#[repr(C)]
pub struct Rect {
    x: c_int,
    y: c_int,
    width: c_int,
    height: c_int,
}

#[repr(C)]
struct RawResult {
    success: *const c_char,
    failure: *const c_char,
    destroy: extern "C" fn(str: *const c_char),
}

type BridgeOnCallback = extern "C" fn(callback_ctx: *mut c_void, ret: RawResult);

#[repr(C)]
struct RawBrowserObserver {
    on_state_change: extern "C" fn(state: BrowserState, ctx: *mut c_void),
    on_ime_rect: extern "C" fn(rect: Rect, ctx: *mut c_void),
    on_frame: extern "C" fn(buf: *const c_void, width: c_int, height: c_int, ctx: *mut c_void),
    on_title_change: extern "C" fn(title: *const c_char, ctx: *mut c_void),
    on_fullscreen_change: extern "C" fn(fullscreen: bool, ctx: *mut c_void),
    on_bridge: extern "C" fn(
        req: *const c_char,
        ctx: *mut c_void,
        callback_ctx: *const c_void,
        callback: BridgeOnCallback,
    ),
}

#[repr(C)]
struct RawBrowserSettings {
    url: *const c_char,
    window_handle: *const c_void,
    frame_rate: u32,
    width: u32,
    height: u32,
    device_scale_factor: c_float,
    is_offscreen: bool,
}

#[repr(C)]
struct RawBrowser {
    r#ref: *const c_void,
}

extern "C" {
    fn create_browser(
        app: *const RawApp,
        settings: *const RawBrowserSettings,
        observer: RawBrowserObserver,
        ctx: *mut c_void,
    ) -> *const RawBrowser;
}

pub struct HWND(pub *const c_void);

unsafe impl Send for HWND {}
unsafe impl Sync for HWND {}

pub struct BrowserSettings<'a> {
    url: Option<&'a str>,
    window_handle: HWND,
    frame_rate: u32,
    width: u32,
    height: u32,
    device_scale_factor: f32,
    is_offscreen: bool,
}

impl Into<RawBrowserSettings> for BrowserSettings<'_> {
    fn into(self) -> RawBrowserSettings {
        RawBrowserSettings {
            url: self.url.as_c_str().0,
            window_handle: self.window_handle.0,
            frame_rate: self.frame_rate,
            width: self.width,
            height: self.height,
            device_scale_factor: self.device_scale_factor,
            is_offscreen: self.is_offscreen,
        }
    }
}

pub struct Browser {
    settings: *mut RawBrowserSettings,
    ptr: *const RawBrowser,
}

unsafe impl Send for Browser {}
unsafe impl Sync for Browser {}

impl Browser {
    pub(crate) async fn new(app: *const RawApp, settings: BrowserSettings<'_>) -> Result<Arc<Self>> {
        
        Ok(())
    }
}

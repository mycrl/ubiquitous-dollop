mod bridge;

use std::{
    ffi::{c_char, c_float, c_int, c_void},
    slice::from_raw_parts,
    sync::Arc,
};

use anyhow::{anyhow, Result};
use serde::{de::DeserializeOwned, Serialize};
use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    oneshot,
};

use crate::{
    app::RawApp,
    ptr::{from_c_str, AsCStr},
};

use self::bridge::Bridge;

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BrowserState {
    Load = 1,
    LoadError = 2,
    BeforeLoad = 3,
    BeforeClose = 4,
    Close = 5,
}

#[repr(C)]
pub struct Rect {
    pub x: c_int,
    pub y: c_int,
    pub width: c_int,
    pub height: c_int,
}

#[repr(C)]
struct RawResult {
    success: *const c_char,
    failure: *const c_char,
    destroy: extern "C" fn(str: *const c_char),
}

type BridgeOnCallback = extern "C" fn(callback_ctx: *mut c_void, ret: RawResult);

#[repr(C)]
#[derive(Clone, Copy)]
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
    fn browser_exit(browser: *const RawBrowser);
}

pub struct HWND(pub *const c_void);

unsafe impl Send for HWND {}
unsafe impl Sync for HWND {}

pub struct BrowserSettings<'a> {
    pub url: Option<&'a str>,
    pub window_handle: HWND,
    pub frame_rate: u32,
    pub width: u32,
    pub height: u32,
    pub device_scale_factor: f32,
    pub is_offscreen: bool,
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

#[allow(unused)]
pub trait Observer: Send + Sync {
    fn on_state_change(&self, state: BrowserState) {}
    fn on_ime_rect(&self, rect: Rect) {}
    fn on_frame(&self, buf: &[u8], width: u32, height: u32) {}
    fn on_title_change(&self, title: String) {}
    fn on_fullscreen_change(&self, fullscreen: bool) {}
}

enum ObserverMsg {
    StateChange(BrowserState),
}

#[derive(Clone)]
struct Delegation {
    observer: Arc<dyn Observer>,
    tx: Arc<UnboundedSender<ObserverMsg>>,
}

impl Delegation {
    fn new<T>(observer: T) -> (Self, UnboundedReceiver<ObserverMsg>)
    where
        T: Observer + 'static,
    {
        let (tx, rx) = unbounded_channel();
        (
            Self {
                observer: Arc::new(observer),
                tx: Arc::new(tx),
            },
            rx,
        )
    }
}

pub struct Browser {
    #[allow(unused)]
    delegation: Delegation,
    settings: *mut RawBrowserSettings,
    ptr: *const RawBrowser,
}

unsafe impl Send for Browser {}
unsafe impl Sync for Browser {}

impl Browser {
    pub(crate) async fn new<T>(
        app: *const RawApp,
        settings: BrowserSettings<'_>,
        observer: T,
    ) -> Result<Arc<Self>>
    where
        T: Observer + 'static,
    {
        let (delegation, mut receiver) = Delegation::new(observer);
        let settings = Box::into_raw(Box::new(settings.into()));
        let ptr = unsafe {
            create_browser(
                app,
                settings,
                BROWSER_OBSERVER,
                &delegation as *const _ as *mut _,
            )
        };

        let (created_tx, created_rx) = oneshot::channel::<bool>();
        let delegation_ = delegation.clone();
        tokio::spawn(async move {
            let mut tx = Some(created_tx);

            while let Some(msg) = receiver.recv().await {
                match msg {
                    ObserverMsg::StateChange(state) => {
                        delegation_.observer.on_state_change(state);
                        match state {
                            BrowserState::LoadError => {
                                tx.take().map(|tx| tx.send(true));
                            }
                            BrowserState::Load => {
                                tx.take().map(|tx| tx.send(false));
                            }
                            _ => (),
                        }
                    }
                }
            }
        });

        if !created_rx.await? {
            return Err(anyhow!("create browser failed, maybe is load failed!"));
        }

        Ok(Arc::new(Self {
            delegation,
            settings,
            ptr,
        }))
    }

    pub async fn call_bridge<Q, S>(&self, req: &Q) -> Result<Option<S>>
    where
        Q: Serialize,
        S: DeserializeOwned,
    {
        Bridge::call(self.ptr, req).await
    }

    pub fn on_bridge<Q, S, H>(&self, handler: H) 
    where
        H: Fn(Q) -> S
    {

    }
}

impl Drop for Browser {
    fn drop(&mut self) {
        drop(unsafe { Box::from_raw(self.settings) });
        unsafe { browser_exit(self.ptr) }
    }
}

static BROWSER_OBSERVER: RawBrowserObserver = RawBrowserObserver {
    on_state_change,
    on_ime_rect,
    on_frame,
    on_title_change,
    on_fullscreen_change,
    on_bridge,
};

extern "C" fn on_state_change(state: BrowserState, ctx: *mut c_void) {
    let ctx = unsafe { &*(ctx as *mut Delegation) };
    ctx.tx
        .send(ObserverMsg::StateChange(state))
        .expect("channel is closed, message send failed!");
}

extern "C" fn on_ime_rect(rect: Rect, ctx: *mut c_void) {
    let ctx = unsafe { &*(ctx as *mut Delegation) };
    ctx.observer.on_ime_rect(rect);
}

extern "C" fn on_frame(buf: *const c_void, width: c_int, height: c_int, ctx: *mut c_void) {
    let ctx = unsafe { &*(ctx as *mut Delegation) };
    ctx.observer.on_frame(
        unsafe { from_raw_parts(buf as *const _, width as usize * height as usize * 4) },
        width as u32,
        height as u32,
    );
}

extern "C" fn on_title_change(title: *const c_char, ctx: *mut c_void) {
    if let Some(title) = from_c_str(title) {
        let ctx = unsafe { &*(ctx as *mut Delegation) };
        ctx.observer.on_title_change(title);
    }
}

extern "C" fn on_fullscreen_change(fullscreen: bool, ctx: *mut c_void) {
    let ctx = unsafe { &*(ctx as *mut Delegation) };
    ctx.observer.on_fullscreen_change(fullscreen);
}

extern "C" fn on_bridge(
    req: *const c_char,
    ctx: *mut c_void,
    callback_ctx: *const c_void,
    callback: BridgeOnCallback,
) {
}

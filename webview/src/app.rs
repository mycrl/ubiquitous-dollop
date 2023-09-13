use std::{
    ffi::{c_char, c_int, c_void},
    sync::Arc,
};

use crate::{ptr::AsCStr, Browser, BrowserSettings, Observer};

use anyhow::{anyhow, Result};
use tokio::{
    runtime::Handle,
    sync::oneshot::{self, Sender},
};

#[repr(C)]
struct RawAppSettings {
    cache_path: *const c_char,
    browser_subprocess_path: *const c_char,
    scheme_path: *const c_char,
}

#[repr(C)]
pub(crate) struct RawApp {
    settings: *const RawAppSettings,
    r#ref: *const c_void,
}

type CreateAppCallback = extern "C" fn(ctx: *mut c_void);

extern "C" {
    fn create_app(
        settings: *const RawAppSettings,
        callback: CreateAppCallback,
        ctx: *mut c_void,
    ) -> *const RawApp;
    fn app_run(app: *const RawApp, argc: c_int, args: *const *const c_char) -> c_int;
    fn app_exit(app: *const RawApp);
}

pub struct AppSettings<'a> {
    pub cache_path: Option<&'a str>,
    pub browser_subprocess_path: Option<&'a str>,
    pub scheme_path: Option<&'a str>,
}

impl Into<RawAppSettings> for AppSettings<'_> {
    fn into(self) -> RawAppSettings {
        RawAppSettings {
            cache_path: self.cache_path.as_c_str().0,
            browser_subprocess_path: self.browser_subprocess_path.as_c_str().0,
            scheme_path: self.scheme_path.as_c_str().0,
        }
    }
}

pub struct App {
    settings: *mut RawAppSettings,
    ptr: *const RawApp,
}

unsafe impl Send for App {}
unsafe impl Sync for App {}

impl App {
    pub async fn new(settings: AppSettings<'_>) -> Result<Arc<Self>> {
        let settings = Box::into_raw(Box::new(settings.into()));
        let (tx, rx) = oneshot::channel::<()>();
        let ptr = unsafe {
            create_app(
                settings,
                create_app_callback,
                Box::into_raw(Box::new(tx)) as *mut _,
            )
        };

        if ptr.is_null() {
            return Err(anyhow!("create app failed!"));
        }

        rx.await?;
        Ok(Arc::new(Self { settings, ptr }))
    }

    pub async fn create_browser<T>(
        &self,
        settings: BrowserSettings<'_>,
        observer: T,
    ) -> Result<Arc<Browser>>
    where
        T: Observer + 'static,
    {
        Browser::new(self.ptr, settings, observer).await
    }

    pub async fn run(self: Arc<Self>) -> Result<()> {
        let args = std::env::args()
            .map(|arg| arg.as_c_str())
            .collect::<Vec<_>>();
        let ret = Handle::current()
            .spawn_blocking(move || unsafe {
                let args = args.iter().map(|arg| arg.0).collect::<Vec<*const c_char>>();
                app_run(self.ptr, args.len() as c_int, args.as_ptr())
            })
            .await?;
        if ret != 0 {
            return Err(anyhow!("app runing failed!, code: {}", ret));
        }

        Ok(())
    }
}

impl Drop for App {
    fn drop(&mut self) {
        drop(unsafe { Box::from_raw(self.settings) });
        unsafe { app_exit(self.ptr) }
    }
}

extern "C" fn create_app_callback(ctx: *mut c_void) {
    let tx = unsafe { Box::from_raw(ctx as *mut Sender<()>) };
    tx.send(()).unwrap();
}

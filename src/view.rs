use std::sync::Arc;

use anyhow::Result;
use webview::{App, AppSettings, Browser, BrowserSettings, Observer};

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
    app: Arc<App>,
    browser: Arc<Browser>,
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

        Ok(Self { browser, app })
    }

    pub fn resize(&self, width: u32, height: u32) {
        self.browser.resize(width, height)
    }
}

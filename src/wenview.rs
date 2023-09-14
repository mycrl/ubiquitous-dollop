use std::sync::Arc;

use anyhow::Result;
use webview::{App, AppSettings};

pub struct Webview {
    app: Arc<App>,
}

impl Webview {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            app: App::new(AppSettings {
                cache_path: None,
                browser_subprocess_path: None,
                scheme_path: None,
            })
            .await?,
        })
    }
}

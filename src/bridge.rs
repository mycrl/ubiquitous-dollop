use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use webview::BridgeObserver;

#[derive(Debug, Deserialize)]
pub struct IceServer {
    pub credential: Option<String>,
    pub username: Option<String>,
    pub urls: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub enum Request {
    Connect {
        code: String,
        ice_server: Option<IceServer>,
    },
}

#[derive(Debug, Serialize)]
pub enum Response {}

pub struct Bridger {}

#[async_trait]
impl BridgeObserver for Bridger {
    type Req = Request;
    type Res = Option<Response>;

    async fn on(&self, req: Self::Req) -> Result<Self::Res> {
        Ok(None)
    }
}

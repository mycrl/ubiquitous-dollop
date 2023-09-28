use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use webview::BridgeObserver;

use crate::{
    rtc::Rtc,
    signaling::Signaling, settings::SignalingSettings,
};

#[derive(Debug, Deserialize)]
pub struct IceServer {
    pub credential: Option<String>,
    pub username: Option<String>,
    pub urls: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub enum Request {
    ConnectSignaling {
        server: String,
        id: String,
        secret: String,
    },
    Start {
        id: String,
    },
}

#[derive(Debug, Serialize)]
pub enum Response {}

pub struct Bridger {
    pub signaling: Arc<Signaling>,
    pub rtc: Arc<Rtc>,
}

#[async_trait]
impl BridgeObserver for Bridger {
    type Req = Request;
    type Res = Option<Response>;
    type Err = anyhow::Error;

    async fn on(&self, req: Self::Req) -> Result<Self::Res, Self::Err> {
        match req {
            Request::ConnectSignaling { server, id, secret } => {
                self.signaling
                    .connect(
                        SignalingSettings { server, id, secret },
                        self.rtc.get_signaler(),
                    )
                    .await?;
            }
            Request::Start { id } => {
                self.rtc.offer(id).await?;
            }
        }

        Ok(None)
    }
}

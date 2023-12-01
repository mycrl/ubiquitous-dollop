use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use librtc::{RTCConfiguration, RTCIceServer};
use serde::{Deserialize, Serialize};
use webview::BridgeObserver;

use crate::{
    rtc::{MaybeUninitRtc, Rtc, SignalingObserver},
    settings::{PartialSettings, Settings, SettingsManager},
    signaling::Signaling,
};

#[derive(Debug, Deserialize)]
pub struct IceServer {
    pub credential: Option<String>,
    pub username: Option<String>,
    pub urls: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", content = "req")]
pub enum Request {
    SetSettings(PartialSettings),
    GetSettings,
    ConnectSignaling,
    Start { id: String },
}

#[derive(Debug, Serialize)]
#[serde(tag = "kind", content = "res")]
pub enum Response {
    GetSettings(Settings),
}

pub struct Bridger {
    pub signaling: Arc<Signaling>,
    pub maybe_uinit_rtc: MaybeUninitRtc,
    pub settings: Arc<SettingsManager>,
}

#[async_trait]
impl BridgeObserver for Bridger {
    type Req = Request;
    type Res = Option<Response>;
    type Err = anyhow::Error;

    async fn on(&self, req: Self::Req) -> Result<Self::Res, Self::Err> {
        match req {
            Request::SetSettings(config) => {
                self.settings.set(config);
            }
            Request::GetSettings => {
                return Ok(Some(Response::GetSettings(self.settings.get().clone())))
            }
            Request::ConnectSignaling => {
                self.signaling.connect(SignalingObserver {
                    maybe_uinit_rtc: self.maybe_uinit_rtc.clone(),
                    signaling: self.signaling.clone(),
                }).await?;
            }
            Request::Start { id } => {
                let settings = self.settings.get().rtc.clone();
                let mut cfg = RTCConfiguration::default();
                cfg.ice_servers = Some(vec![RTCIceServer {
                    credential: settings.credential,
                    username: settings.username,
                    urls: settings.urls,
                }]);

                let rtc = Rtc::new(&cfg, self.signaling.clone())?;
                let _ = self.maybe_uinit_rtc.write().await.insert(rtc.clone());
                rtc.offer(id).await?;
            }
        }

        Ok(None)
    }
}

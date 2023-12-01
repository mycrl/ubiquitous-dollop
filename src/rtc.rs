use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use librtc::{
    Observer, RTCConfiguration, RTCIceCandidate, RTCPeerConnection, RTCSessionDescription,
};

use tokio::sync::RwLock;

use crate::signaling::{Signaler, Signaling};

pub struct SignalingObserver {
    pub maybe_uinit_rtc: MaybeUninitRtc,
    pub signaling: Arc<Signaling>,
}

#[async_trait]
impl Signaler for SignalingObserver {
    async fn on_offer(&self, id: String, offer: RTCSessionDescription) {
        let handle = || async {
            if let Some(rtc) = self.maybe_uinit_rtc.read().await.as_ref() {
                rtc.peerconnection.set_remote_description(&offer).await?;
                let answer = rtc.peerconnection.create_answer().await?;
                rtc.peerconnection.set_local_description(&answer).await?;
                self.signaling.send_answer(id, answer)?;
            }

            Ok::<(), anyhow::Error>(())
        };

        if let Err(e) = handle().await {
            log::error!("failed to on remote offer!, error={:?}", e);
        }
    }

    async fn on_answer(&self, _: String, answer: RTCSessionDescription) {
        if let Some(rtc) = self.maybe_uinit_rtc.read().await.as_ref() {
            if let Err(e) = rtc.peerconnection.set_remote_description(&answer).await {
                log::error!("failed to set remote answer!, error={:?}", e);
            }
        }
    }

    async fn on_ice_candidate(&self, _: String, candidate: RTCIceCandidate) {
        if let Some(rtc) = self.maybe_uinit_rtc.read().await.as_ref() {
            if let Err(e) = rtc.peerconnection.add_ice_candidate(&candidate) {
                log::error!("failed to add remote candidate!, error={:?}", e);
            }
        }
    }
}

struct RtcObserver {
    signaling: Arc<Signaling>,
}

impl Observer for RtcObserver {
    fn on_ice_candidate(&self, candidate: RTCIceCandidate) {
        if let Some(to) = self.signaling.get_last_to() {
            if let Err(e) = self.signaling.send_ice_candidate(to, candidate) {
                log::error!("failed to send ice candidate signaling!, error={:?}", e);
            }
        }
    }
}

pub type MaybeUninitRtc = Arc<RwLock<Option<Arc<Rtc>>>>;

pub struct Rtc {
    peerconnection: Arc<RTCPeerConnection>,
    signaling: Arc<Signaling>,
}

impl Rtc {
    pub fn new(config: &RTCConfiguration, signaling: Arc<Signaling>) -> Result<Arc<Self>> {
        Ok(Arc::new(Self {
            signaling: signaling.clone(),
            peerconnection: RTCPeerConnection::new(config, RtcObserver { signaling })?,
        }))
    }

    pub async fn offer(&self, id: String) -> Result<()> {
        let offer = self.peerconnection.create_offer().await?;
        self.peerconnection.set_local_description(&offer).await?;
        self.signaling.send_offer(id, offer)?;

        Ok(())
    }
}

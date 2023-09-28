use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use librtc::{
    Observer, RTCConfiguration, RTCIceCandidate, RTCPeerConnection, RTCSessionDescription,
};

use crate::signaling::{Signaler, Signaling};

pub struct SignalingObserver {
    conn: Arc<RTCPeerConnection>,
    signaling: Arc<Signaling>,
}

#[async_trait]
impl Signaler for SignalingObserver {
    async fn on_offer(&self, id: String, offer: RTCSessionDescription) {
        let handle = || async {
            self.conn.set_remote_description(&offer).await?;
            let answer = self.conn.create_answer().await?;
            self.conn.set_local_description(&answer).await?;
            self.signaling.send_answer(id, answer)?;

            Ok::<(), anyhow::Error>(())
        };

        if let Err(e) = handle().await {
            log::error!("failed to on remote offer!, error={:?}", e);
        }
    }

    async fn on_answer(&self, _: String, answer: RTCSessionDescription) {
        if let Err(e) = self.conn.set_remote_description(&answer).await {
            log::error!("failed to set remote answer!, error={:?}", e);
        }
    }

    async fn on_ice_candidate(&self, _: String, candidate: RTCIceCandidate) {
        if let Err(e) = self.conn.add_ice_candidate(&candidate) {
            log::error!("failed to add remote candidate!, error={:?}", e);
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

pub struct Rtc {
    conn: Arc<RTCPeerConnection>,
    signaling: Arc<Signaling>,
}

impl Rtc {
    pub fn new(config: &RTCConfiguration, signaling: Arc<Signaling>) -> Result<Arc<Self>> {
        Ok(Arc::new(Self {
            signaling: signaling.clone(),
            conn: RTCPeerConnection::new(config, RtcObserver { signaling })?,
        }))
    }

    pub fn get_signaler(&self) -> SignalingObserver {
        SignalingObserver {
            conn: self.conn.clone(),
            signaling: self.signaling.clone(),
        }
    }

    pub async fn offer(&self, id: String) -> Result<()> {
        let offer = self.conn.create_offer().await?;
        self.conn.set_local_description(&offer).await?;
        self.signaling.send_offer(id, offer)?;

        Ok(())
    }
}

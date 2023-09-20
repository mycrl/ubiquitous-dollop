use std::sync::Arc;

use anyhow::Result;
use librtc::{Observer, RTCConfiguration, RTCIceCandidate, RTCPeerConnection};

struct RtcObserver {}

impl Observer for RtcObserver {
    fn on_ice_candidate(&self, candidate: RTCIceCandidate) {}
}

pub struct Rtc {
    conn: Arc<RTCPeerConnection>,
}

impl Rtc {
    pub fn new(config: &RTCConfiguration) -> Result<Self> {
        Ok(Self {
            conn: RTCPeerConnection::new(config, RtcObserver {})?,
        })
    }
}

use std::sync::Arc;

use anyhow::Result;
use librtc::{RTCPeerConnection, Observer, RTCConfiguration};

struct RtcObserver;

impl Observer for RtcObserver {
    fn on_track(&self, track: librtc::MediaStreamTrack) {
        
    }
}

pub struct Rtc {
    conn: Arc<RTCPeerConnection>,
}

impl Rtc {
    pub fn new(config: &RTCConfiguration) -> Result<Self> {
        Ok(Self {
            conn: RTCPeerConnection::new(config, RtcObserver)?,
        })
    }
}

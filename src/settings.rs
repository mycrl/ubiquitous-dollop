use std::sync::{RwLock, RwLockReadGuard};

use macros::Partial;
use serde::{Deserialize, Serialize};

#[derive(Partial, Debug, Default, Clone, Deserialize, Serialize)]
#[partial(
    alias = "PartialSignalingSettings", 
    derives = [Debug + Default + Clone + Deserialize + Serialize]
)]
pub struct SignalingSettings {
    pub server: String,
    pub id: String,
    pub secret: String,
}

#[derive(Partial, Debug, Default, Clone, Deserialize, Serialize)]
#[partial(
    alias = "PartialRtcSettings", 
    derives = [Debug + Default + Clone + Deserialize + Serialize]
)]
pub struct RtcSettings {
    pub credential: Option<String>,
    pub username: Option<String>,
    pub urls: Option<Vec<String>>,
}

#[derive(Partial, Debug, Default, Clone, Deserialize, Serialize)]
#[partial(
    alias = "PartialSettings", 
    derives = [Debug + Default + Clone + Deserialize + Serialize]
)]
pub struct Settings {
    #[partial(from = "PartialSignalingSettings")]
    pub signaling: SignalingSettings,
    #[partial(from = "PartialRtcSettings")]
    pub rtc: RtcSettings,
}

#[derive(Default)]
pub struct SettingsManager(RwLock<Settings>);

impl SettingsManager {
    pub fn get(&self) -> RwLockReadGuard<Settings> {
        self.0.read().unwrap()
    }

    pub fn set(&self, settings: PartialSettings) {
        self.0.write().unwrap().from_partial(settings)
    }
}

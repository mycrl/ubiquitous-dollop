use std::sync::RwLock;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

pub static GLOBAL_SETTINGS: Lazy<RwLock<Settings>> = Lazy::new(|| {
    RwLock::new(Settings::default())
});

pub struct SettingsManager;

impl SettingsManager {
    #[inline]
    pub fn get_ref() -> Settings {
        GLOBAL_SETTINGS.read().unwrap().clone()
    }
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct SignalingSettings {
    #[serde(default = "SignalingSettings::server")]
    pub server: String,
    #[serde(default = "SignalingSettings::id")]
    pub id: String,
    #[serde(default = "SignalingSettings::secret")]
    pub secret: String,
}

impl SignalingSettings {
    fn server() -> String {
        SettingsManager::get_ref().signaling.server.clone()
    }

    fn id() -> String {
        SettingsManager::get_ref().signaling.id.clone()
    }
    
    fn secret() -> String {
        SettingsManager::get_ref().signaling.secret.clone()
    }
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct Settings {
    pub signaling: SignalingSettings,
}

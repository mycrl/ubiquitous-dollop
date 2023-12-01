use std::sync::{Arc, Mutex, RwLock};

use anyhow::Result;
use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use librtc::{RTCIceCandidate, RTCSessionDescription};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::settings::SettingsManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Signal {
    Offer(RTCSessionDescription),
    Answer(RTCSessionDescription),
    Candidate(RTCIceCandidate),
}

#[derive(Debug, Serialize, Deserialize)]
struct Payload {
    to: String,
    from: String,
    data: Signal,
}

#[allow(unused)]
#[async_trait]
pub trait Signaler: Send + Sync {
    async fn on_offer(&self, id: String, offer: RTCSessionDescription) {}
    async fn on_answer(&self, id: String, answer: RTCSessionDescription) {}
    async fn on_ice_candidate(&self, id: String, candidate: RTCIceCandidate) {}
}

#[derive(Clone)]
pub struct Signaling {
    sender: Arc<RwLock<Option<UnboundedSender<(String, Signal)>>>>,
    last_to: Arc<Mutex<Option<String>>>,
    settings: Arc<SettingsManager>,
}

impl Signaling {
    pub fn new(settings: Arc<SettingsManager>) -> Self {
        Self {
            settings,
            sender: Default::default(),
            last_to: Default::default(),
        }
    }

    pub async fn connect<T>(&self, signaler: T) -> Result<()>
    where
        T: Signaler + 'static,
    {
        let settings = self.settings.get().signaling.clone();
        let (mut stream, _) = connect_async(format!(
            "{server}?id={id}&secret={secret}",
            server = settings.server,
            id = settings.id,
            secret = settings.secret
        ))
        .await?;

        let sender = self.sender.clone();
        let last_to = self.last_to.clone();

        tokio::spawn(async move {
            let (tx, mut rx) = unbounded_channel();

            {
                let _ = sender.write().unwrap().insert(tx);
            }

            loop {
                tokio::select! {
                    Some(ret) = stream.next() => {
                        if let Ok(msg) = ret {
                            if let Message::Text(msg) = msg {
                                if let Ok(payload) = serde_json::from_str::<Payload>(&msg) {
                                    let _ = last_to.lock().unwrap().insert(payload.from.clone());

                                    match payload.data {
                                        Signal::Offer(offer) => signaler.on_offer(payload.from, offer).await,
                                        Signal::Answer(answer) => signaler.on_answer(payload.from, answer).await,
                                        Signal::Candidate(candidate) => signaler.on_ice_candidate(payload.from, candidate).await,
                                    }
                                }
                            }
                        } else {
                            break;
                        }
                    }
                    Some((to, data)) = rx.recv() => {
                        let _ = last_to.lock().unwrap().insert(to.clone());

                        if let Ok(payload) = serde_json::to_string(&Payload {
                            from: settings.id.clone(),
                            data,
                            to,
                        }) {
                            if stream.send(Message::Text(payload)).await.is_err() {
                                break;
                            }
                        }
                    }
                }
            }

            sender.write().unwrap().take();
        });

        Ok(())
    }

    pub fn send_offer(&self, to: String, offer: RTCSessionDescription) -> Result<()> {
        if let Some(sender) = self.sender.read().unwrap().as_ref() {
            sender.send((to, Signal::Offer(offer)))?;
        }

        Ok(())
    }

    pub fn send_answer(&self, to: String, answer: RTCSessionDescription) -> Result<()> {
        if let Some(sender) = self.sender.read().unwrap().as_ref() {
            sender.send((to, Signal::Answer(answer)))?;
        }

        Ok(())
    }

    pub fn send_ice_candidate(&self, to: String, candidate: RTCIceCandidate) -> Result<()> {
        if let Some(sender) = self.sender.read().unwrap().as_ref() {
            sender.send((to, Signal::Candidate(candidate)))?;
        }

        Ok(())
    }

    pub fn get_last_to(&self) -> Option<String> {
        self.last_to.lock().unwrap().as_ref().cloned()
    }
}

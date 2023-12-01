use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
};

use anyhow::{anyhow, Result};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio::{
    net::TcpListener,
    sync::mpsc::{unbounded_channel, UnboundedSender},
};

use tokio_tungstenite::{
    accept_hdr_async,
    tungstenite::{
        handshake::server::{Callback, ErrorResponse, Request, Response},
        http::{response, StatusCode},
        Message,
    },
};

#[derive(Debug, Deserialize)]
struct Payload {
    to: String,
}

struct AuthParams<'a> {
    id: &'a str,
    secret: &'a str,
}

impl<'a> TryFrom<&'a str> for AuthParams<'a> {
    type Error = anyhow::Error;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut kvs = HashMap::new();
        for item in value.split('&') {
            let mut it = item.split('=').take(2);
            if let (Some(k), Some(v)) = (it.next(), it.next()) {
                kvs.insert(k, v);
            }
        }

        let id = kvs.get("id").ok_or_else(|| anyhow!("not found id!"))?;
        let secret = kvs
            .get("secret")
            .ok_or_else(|| anyhow!("not found secret!"))?;
        Ok(Self { id, secret })
    }
}

struct Guarder {
    router: Arc<Router>,
    sender: UnboundedSender<Signal>,
    id: Arc<Mutex<Option<String>>>,
    secret: String,
}

impl Callback for Guarder {
    fn on_request(self, request: &Request, response: Response) -> Result<Response, ErrorResponse> {
        let handle = || {
            let auth = AuthParams::try_from(request.uri().query().unwrap_or(""))?;
            if auth.secret != self.secret {
                return Err(anyhow!("auth failed!"));
            }

            let _ = self.id.lock().unwrap().insert(auth.id.to_string());
            self.router.register(auth.id.to_string(), self.sender);
            Ok::<(), anyhow::Error>(())
        };

        match handle() {
            Ok(_) => Ok(response),
            Err(e) => Err(response::Builder::new()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Some(e.to_string()))
                .unwrap()),
        }
    }
}

enum Signal {
    Message(String),
    Closed,
}

#[derive(Default)]
struct Router(RwLock<HashMap<String, UnboundedSender<Signal>>>);

impl Router {
    fn register(&self, id: String, sender: UnboundedSender<Signal>) {
        if let Some(sender) = self.0.write().unwrap().insert(id, sender) {
            let _ = sender.send(Signal::Closed);
        }
    }

    fn unregister(&self, id: &str) {
        self.0.write().unwrap().remove(id);
    }

    fn send(&self, id: &str, payload: String) -> Result<()> {
        if let Some(sender) = self.0.read().unwrap().get(id) {
            sender.send(Signal::Message(payload))?;
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    simple_logger::init()?;

    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    log::info!("signaling server listening: addr={}", "127.0.0.1:8080");

    let router = Arc::new(Router::default());
    while let Ok((socket, addr)) = listener.accept().await {
        log::info!("tcp socket accept: addr={:?}", addr);

        let router = router.clone();
        tokio::spawn(async move {
            let (sender, mut receiver) = unbounded_channel();
            let id = Arc::new(Mutex::new(None));

            if let Ok(mut stream) = accept_hdr_async(
                socket,
                Guarder {
                    secret: "test".to_string(),
                    router: router.clone(),
                    id: id.clone(),
                    sender,
                },
            )
            .await
            {
                let id = match id.lock().unwrap().as_ref() {
                    Some(id) => id.clone(),
                    None => return,
                };

                log::info!("websocket accept: addr={:?}, id={}", addr, id);
                loop {
                    tokio::select! {
                        Some(ret) = stream.next() => {
                            if let Ok(msg) = ret {
                                if let Message::Text(payload) = msg {
                                    if let Ok(json) = serde_json::from_str::<Payload>(&payload) {
                                        if router.send(&json.to, payload).is_err() {
                                            break;
                                        }
                                    }
                                }
                            } else {
                                break;
                            }
                        }
                        Some(signal) = receiver.recv() => {
                            match signal {
                                Signal::Message(msg) => {
                                    if stream.send(Message::Text(msg)).await.is_err() {
                                        break;
                                    }
                                }
                                Signal::Closed => {
                                    break;
                                }
                            }
                        }
                        else => ()
                    }
                }

                log::info!("websocket disconnect: addr={:?}, id={}", addr, id);
                router.unregister(&id);
            }
        });
    }

    Ok(())
}

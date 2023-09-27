use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use anyhow::{anyhow, Result};
use futures_util::stream::StreamExt;
use tokio::{net::TcpListener, sync::mpsc::UnboundedSender};
use tokio_tungstenite::{
    accept_async,
    tungstenite::{
        handshake::server::{Callback, ErrorResponse, Request, Response},
        http::{response, StatusCode},
        Message,
    },
};

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
    sender: UnboundedSender<String>,
    secret: String,
}

impl Callback for Guarder {
    fn on_request(self, request: &Request, response: Response) -> Result<Response, ErrorResponse> {
        let ret = (|| {
            let auth = AuthParams::try_from(request.uri().query().unwrap_or(""))?;
            if auth.secret != self.secret {
                return Err(anyhow!("auth failed!"));
            }

            self.router.register(auth.id.to_string(), self.sender);
            Ok::<(), anyhow::Error>(())
        })();

        match ret {
            Ok(_) => Ok(response),
            Err(e) => Err(response::Builder::new()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Some(e.to_string()))
                .unwrap()),
        }
    }
}

#[derive(Default)]
struct Router(RwLock<HashMap<String, UnboundedSender<String>>>);

impl Router {
    fn register(&self, id: String, sender: UnboundedSender<String>) {
        self.0.write().unwrap().insert(id, sender);
    }

    fn send(&self, id: &str, payload: String) -> Result<()> {
        if let Some(sender) = self.0.read().unwrap().get(id) {
            sender.send(payload)?;
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    let router = Arc::new(Router::default());

    while let Ok((socket, addr)) = listener.accept().await {
        tokio::spawn(async move {
            if let Ok(mut stream) = accept_async(socket).await {
                loop {
                    tokio::select! {
                        Some(ret) = stream.next() => {
                            let msg = match ret {
                                Ok(ret) => ret,
                                Err(_) => {
                                    break;
                                }
                            };

                            if let Message::Text(payload) = msg {

                            }
                        }
                    }
                }
            }
        });
    }

    Ok(())
}

use std::{
    ffi::{c_char, c_void},
    time::Duration,
};

use anyhow::Result;
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use tokio::{
    sync::oneshot::{channel, Sender},
    time::timeout,
};

use super::RawBrowser;
use crate::ptr::{from_c_str, AsCStr};

type BridgeCallCallback = extern "C" fn(res: *const c_char, ctx: *mut c_void);

extern "C" {
    fn browser_bridge_call(
        browser: *const RawBrowser,
        req: *const c_char,
        callback: BridgeCallCallback,
        ctx: *mut c_void,
    );
}

#[async_trait]
pub trait BridgeOnExt: Send + Sync {
    type Req;
    type Res;

    async fn on(&self, req: Self::Req) -> Result<Self::Res>;
}

pub struct BridgeOnHandler<Q, S> {
    pub(crate) processor: Box<dyn BridgeOnExt<Req = Q, Res = S>>,
}

impl<Q, S> BridgeOnHandler<Q, S>
where
    Q: DeserializeOwned + Send,
    S: Serialize + 'static,
{
    pub fn new<T: BridgeOnExt<Req = Q, Res = S> + 'static>(processer: T) -> Self {
        Self {
            processor: Box::new(processer),
        }
    }

    pub(crate) async fn handle(&self, req: &str) -> Result<String, String> {
        serde_json::to_string(
            &self
                .processor
                .on(serde_json::from_str(unsafe { std::mem::transmute(req) })
                    .map_err(|s| s.to_string())?)
                .await
                .map_err(|s| s.to_string())?,
        )
        .map_err(|s| s.to_string())
    }
}

pub(crate) struct BridgeOnContext(
    pub Box<dyn Fn(String, Box<dyn FnOnce(Result<String, String>) + Send + Sync>)>,
);

pub struct Bridge;

impl Bridge {
    pub async fn call<Q, S>(
        ptr: *const RawBrowser,
        req: &Q,
    ) -> Result<Option<S>>
    where
        Q: Serialize,
        S: DeserializeOwned,
    {
        let (tx, rx) = channel::<Option<String>>();
        let req = serde_json::to_string(req)?.as_c_str();

        unsafe {
            browser_bridge_call(
                ptr,
                req.0,
                bridge_call_callback,
                Box::into_raw(Box::new(tx)) as *mut c_void,
            );
        }

        Ok(
            if let Some(ret) = timeout(Duration::from_secs(10), rx).await?? {
                Some(serde_json::from_str(&ret)?)
            } else {
                None
            },
        )
    }
}

extern "C" fn bridge_call_callback(res: *const c_char, ctx: *mut c_void) {
    let tx = unsafe { Box::from_raw(ctx as *mut Sender<Option<String>>) };
    tx.send(from_c_str(res))
        .expect("channel is closed, message send failed!");
}

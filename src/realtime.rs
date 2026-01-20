use crate::httpc::HttpClient;
use anyhow::{anyhow, Result};
use eventsource_stream::Eventsource;
use futures_lite::stream::StreamExt;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::task::AbortHandle;

type SubscriptionCallback = Box<dyn Fn(EventResponse) + Send + Sync + 'static>;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecordOperations {
    Create,
    Update,
    Delete,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EventResponse {
    pub action: RecordOperations,
    pub record: Value,
}

#[derive(Clone)]
pub struct RealtimeClient {
    inner: Arc<RealtimeClientInner>,
}

impl std::fmt::Debug for RealtimeClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RealtimeClient")
            .field("inner", &"...")
            .finish()
    }
}

struct RealtimeClientInner {
    http_client: HttpClient,
    client_id: RwLock<Option<String>>,
    subscriptions: RwLock<HashMap<String, Vec<SubscriptionCallback>>>,
    connected: RwLock<bool>,
    abort_handle: RwLock<Option<AbortHandle>>,
}

impl RealtimeClient {
    pub fn new(http_client: HttpClient) -> Self {
        Self {
            inner: Arc::new(RealtimeClientInner {
                http_client,
                client_id: RwLock::new(None),
                subscriptions: RwLock::new(HashMap::new()),
                connected: RwLock::new(false),
                abort_handle: RwLock::new(None),
            }),
        }
    }

    pub fn client_id(&self) -> Option<String> {
        self.inner.client_id.read().unwrap().clone()
    }

    async fn connect(&self) -> Result<()> {
        let response = self
            .inner
            .http_client
            .get("/api/realtime", None)
            .send()
            .await?
            .error_for_status()?;

        let mut stream = Box::pin(response.bytes_stream().eventsource());

        let first_event = stream
            .next()
            .await
            .ok_or_else(|| anyhow!("Stream closed immediately"))??;

        if first_event.event == "PB_CONNECT" {
            *self.inner.client_id.write().unwrap() = Some(first_event.id);
            *self.inner.connected.write().unwrap() = true;

            if !self.inner.subscriptions.read().unwrap().is_empty() {
                self.submit_subscriptions().await?;
            }
        } else {
            return Err(anyhow!("Expected PB_CONNECT, got {}", first_event.event));
        }

        let inner = self.inner.clone();

        let handle = tokio::spawn(async move {
            while let Some(item) = stream.next().await {
                match item {
                    Ok(event) => {
                        if let Some(callbacks) =
                            inner.subscriptions.read().unwrap().get(&event.event)
                        {
                            let data = serde_json::from_str::<EventResponse>(&event.data);

                            if let Ok(data) = data {
                                for cb in callbacks {
                                    cb(data.clone());
                                }
                            }
                        }
                    }
                    Err(_) => break,
                }
            }

            *inner.connected.write().unwrap() = false;
            *inner.client_id.write().unwrap() = None;
        });

        *self.inner.abort_handle.write().unwrap() = Some(handle.abort_handle());

        Ok(())
    }

    pub async fn subscribe<F>(&self, topic: &str, callback: F) -> Result<()>
    where
        F: Fn(EventResponse) + Send + Sync + 'static,
    {
        self.inner
            .subscriptions
            .write()
            .unwrap()
            .entry(topic.to_string())
            .or_default()
            .push(Box::new(callback));

        if !*self.inner.connected.read().unwrap() {
            self.connect().await?;
        } else {
            self.submit_subscriptions().await?;
        }

        Ok(())
    }

    pub async fn unsubscribe(&self, topic: &str) -> Result<()> {
        self.inner.subscriptions.write().unwrap().remove(topic);

        if *self.inner.connected.read().unwrap() {
            self.submit_subscriptions().await?;
        }

        if self.inner.subscriptions.read().unwrap().is_empty() {
            self.disconnect();
        }

        Ok(())
    }

    async fn submit_subscriptions(&self) -> Result<()> {
        let client_id = self.client_id().ok_or_else(|| anyhow!("Not connected"))?;
        let subs: Vec<String> = self
            .inner
            .subscriptions
            .read()
            .unwrap()
            .keys()
            .cloned()
            .collect();

        let body = serde_json::json!({
            "clientId": client_id,
            "subscriptions": subs
        });

        let _res = self
            .inner
            .http_client
            .post("/api/realtime", body.to_string())
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    fn disconnect(&self) {
        if let Some(abort_handle) = self.inner.abort_handle.read().unwrap().clone() {
            abort_handle.abort();
        }
        *self.inner.connected.write().unwrap() = false;
        *self.inner.client_id.write().unwrap() = None;
    }
}

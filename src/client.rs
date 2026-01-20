use crate::auth::{Auth, AuthState, AuthStore, AuthenticatedRequest, Authorized, Unauthorized};
use crate::realtime::RealtimeClient;
use crate::{collections::CollectionsManager, httpc::HttpClient};
use crate::{logs::LogsManager, records::RecordsManager};
use anyhow::{anyhow, Result};
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct Client<State: AuthState> {
    http_client: HttpClient,
    realtime_client: RealtimeClient,
    auth: Auth<State>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HealthCheckResponse {
    pub code: i32,
    pub message: String,
}

impl<State: AuthState> Client<State> {
    pub fn auth(&self) -> &Auth<State> {
        &self.auth
    }

    pub fn base_url(&self) -> &str {
        self.http_client.base_url()
    }

    pub fn collection(&self, collection_name: &'static str) -> RecordsManager<'_> {
        RecordsManager {
            http_client: &self.http_client,
            realtime_client: &self.realtime_client,
            token: self.auth.token(),
            collection_name,
        }
    }

    pub async fn auth_with_password(
        &self,
        collection: &str,
        identifier: &str,
        secret: &str,
    ) -> Result<Client<Authorized>> {
        let auth = self
            .auth
            .auth_with_password(collection, identifier, secret)
            .await?;

        Ok(Client {
            auth,
            http_client: self.http_client.clone(),
            realtime_client: self.realtime_client.clone(),
        })
    }
}
impl Client<Unauthorized> {
    pub fn new(base_url: &str) -> Client<Unauthorized> {
        let http_client = HttpClient::new(base_url);

        Client {
            auth: Auth::<Unauthorized>::new(http_client.clone()),
            realtime_client: RealtimeClient::new(http_client.clone()),
            http_client,
        }
    }

    pub async fn health_check(&self) -> Result<HealthCheckResponse> {
        match self.http_client.get("/api/health", None).send().await {
            Ok(response) => Ok(response.json::<HealthCheckResponse>().await?),
            Err(e) => Err(anyhow!("{}", e)),
        }
    }
}

impl Client<Authorized> {
    pub fn new_with_auth(base_url: &str, auth_store: AuthStore) -> Result<Client<Authorized>> {
        let http_client = HttpClient::new(base_url);

        Ok(Client {
            auth: Auth::<Authorized>::new(auth_store, http_client.clone())?,
            realtime_client: RealtimeClient::new(http_client.clone()),
            http_client,
        })
    }

    pub fn auth_store(&self) -> &AuthStore {
        self.auth.auth_store()
    }

    pub fn auth_token(&self) -> &str {
        self.auth
            .token()
            .expect("Authorized client must have a token")
    }

    pub fn collections(&self) -> CollectionsManager<'_> {
        CollectionsManager {
            client: &self.http_client,
            token: self.auth_token(),
        }
    }

    pub fn logs(&self) -> LogsManager<'_> {
        LogsManager {
            client: &self.http_client,
            token: self.auth_token(),
        }
    }

    pub async fn health_check(&self) -> Result<HealthCheckResponse> {
        match self
            .http_client
            .get("/api/health", None)
            .attach_auth_info(self.auth_token())
            .send()
            .await
        {
            Ok(response) => Ok(response.json::<HealthCheckResponse>().await?),
            Err(e) => Err(anyhow!("{}", e)),
        }
    }
}

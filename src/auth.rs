use std::fmt::Debug;
use crate::httpc::HttpClient;
use crate::records::RecordId;
use anyhow::{anyhow, Result};
use reqwest::header::AUTHORIZATION;
use reqwest::RequestBuilder;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::marker::PhantomData;

#[derive(Debug, Clone, Deserialize)]
pub struct AuthSuccessResponse {
    #[serde(rename = "record")]
    pub record_value: serde_json::Value,
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthBaseFields {
    pub id: RecordId,
    #[serde(rename = "collectionName")]
    pub collection_name: String,
    #[serde(rename = "collectionId")]
    pub collection_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthRecord {
    #[serde(flatten)]
    pub base_fields: AuthBaseFields,
    #[serde(flatten)]
    pub fields: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthStore {
    record: AuthRecord,
    token: String,
}

pub trait AuthState: Clone {
    type Storage: Debug + Clone;

    fn token(storage: &Self::Storage) -> Option<&str>;
}

#[derive(Clone)]
pub struct Authorized;

impl AuthState for Authorized {
    type Storage = AuthStore;

    fn token(storage: &Self::Storage) -> Option<&str> {
        Some(&storage.token)
    }
}

#[derive(Clone)]
pub struct Unauthorized;

impl AuthState for Unauthorized {
    type Storage = ();

    fn token(_storage: &Self::Storage) -> Option<&str> {
        None
    }
}

#[derive(Debug, Clone)]
pub struct Auth<State: AuthState> {
    _state: PhantomData<State>,
    auth_store: State::Storage,
    record_value: serde_json::Value,
    http_client: HttpClient,
}

impl Auth<Authorized> {
    pub fn new(auth_store: AuthStore, http_client: HttpClient) -> Result<Self> {
        Ok(Self {
            _state: PhantomData,
            record_value: serde_json::to_value(&auth_store.record)?,
            auth_store,
            http_client,
        })
    }

    pub fn auth_store(&self) -> &AuthStore {
        &self.auth_store
    }

    pub fn record<T: Serialize + DeserializeOwned>(&self) -> Result<(AuthBaseFields, T)> {
        Ok((
            self.auth_store.record.base_fields.clone(),
            serde_json::from_value(self.record_value.clone())?,
        ))
    }

    pub async fn refresh(&mut self) -> Result<()> {
        let url = format!(
            "/api/collections/{}/auth-refresh",
            self.auth_store.record.base_fields.collection_name
        );

        match self
            .http_client
            .post(&url, "".to_string())
            .try_attach_auth_info(Some(self.auth_store.token.as_str()))
            .send()
            .await
        {
            Ok(response) => {
                let response = response.json::<AuthSuccessResponse>().await?;

                self.auth_store.record = serde_json::from_value(response.record_value.clone())?;
                self.record_value = response.record_value.clone();
                self.auth_store.token = response.token.clone();

                Ok(())
            }
            Err(e) => Err(anyhow!("{}", e)),
        }
    }
}

impl Auth<Unauthorized> {
    pub fn new(http_client: HttpClient) -> Self {
        Self {
            _state: PhantomData,
            auth_store: (),
            record_value: serde_json::Value::Null,
            http_client: http_client.clone(),
        }
    }
}

impl<State: AuthState> Auth<State> {
    pub fn token(&self) -> Option<&str> {
        State::token(&self.auth_store)
    }

    pub async fn auth_with_password(
        &self,
        collection: &str,
        identifier: &str,
        secret: &str,
    ) -> Result<Auth<Authorized>> {
        let url = format!("/api/collections/{}/auth-with-password", collection);

        let auth_payload = json!({
            "identity": identifier,
            "password": secret
        });

        match self
            .http_client
            .post(&url, auth_payload.to_string())
            .send()
            .await
        {
            Ok(response) => {
                let response = response.json::<AuthSuccessResponse>().await?;

                Ok(Auth::<Authorized>::new(
                    AuthStore {
                        record: serde_json::from_value::<AuthRecord>(response.record_value)?,
                        token: response.token,
                    },
                    self.http_client.to_owned(),
                )?)
            }
            Err(e) => Err(anyhow!("{}", e)),
        }
    }
}

pub trait AuthenticatedRequest {
    fn attach_auth_info(self, token: &str) -> Self;
    fn try_attach_auth_info(self, token: Option<&str>) -> Self;
}

impl AuthenticatedRequest for RequestBuilder {
    fn attach_auth_info(self, token: &str) -> Self {
        self.header(AUTHORIZATION, token.to_owned())
    }

    fn try_attach_auth_info(self, token: Option<&str>) -> Self {
        if let Some(token) = token {
            return self.header(AUTHORIZATION, token.to_owned());
        }
        self
    }
}

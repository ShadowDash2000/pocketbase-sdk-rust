use crate::httpc::MAX_BODY_SIZE;
use crate::{collections::CollectionsManager, httpc::Httpc};
use crate::{logs::LogsManager, records::RecordsManager};
use anyhow::{anyhow, Result};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Clone, Deserialize)]
pub struct AuthSuccessResponse<T> {
    pub record: AuthRecord<T>,
    pub token: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthBaseFields {
    #[serde(rename = "collectionName")]
    pub collection_name: String,
    #[serde(rename = "collectionId")]
    pub collection_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthRecord<T> {
    #[serde(flatten)]
    pub base_fields: AuthBaseFields,
    #[serde(flatten)]
    pub fields: T,
}

#[derive(Debug, Clone)]
pub struct AuthStore {
    base_url: String,
    record: AuthRecord<serde_json::Value>,
    pub(crate) token: String,
}

impl AuthStore {
    pub fn record<T: DeserializeOwned>(&self) -> Result<AuthRecord<T>> {
        Ok(AuthRecord {
            base_fields: self.record.base_fields.clone(),
            fields: serde_json::from_value(self.record.fields.clone())?,
        })
    }

    pub fn refresh(&mut self) -> Result<()> {
        let url = format!(
            "{}/api/collections/{}/auth-refresh",
            self.base_url, self.record.base_fields.collection_name
        );

        match Httpc::post(Some(self), &url, "".to_string()) {
            Ok(mut response) => {
                let response = response
                    .body_mut()
                    .with_config()
                    .limit(MAX_BODY_SIZE)
                    .read_json::<AuthSuccessResponse<serde_json::Value>>()?;

                self.record = response.record.clone();
                self.token = response.token.clone();

                Ok(())
            }
            Err(e) => Err(anyhow!("{}", e)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Client {
    base_url: String,
    auth: Option<AuthStore>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HealthCheckResponse {
    pub code: i32,
    pub message: String,
}

impl Client {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            auth: None,
        }
    }

    pub fn new_with_auth(base_url: &str, auth: AuthStore) -> Self {
        Self {
            base_url: base_url.to_string(),
            auth: Some(auth),
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn auth_store(&self) -> Option<&AuthStore> {
        self.auth.as_ref()
    }

    pub fn auth_token(&self) -> Option<&str> {
        self.auth.as_ref().map(|auth| auth.token.as_str())
    }

    pub fn collections(&self) -> CollectionsManager<'_> {
        CollectionsManager { client: self }
    }

    pub fn logs(&self) -> LogsManager<'_> {
        LogsManager { client: self }
    }

    pub fn records(&self, record_name: &'static str) -> RecordsManager<'_> {
        RecordsManager {
            client: self,
            collection_name: record_name,
        }
    }

    pub fn health_check(&self) -> Result<HealthCheckResponse> {
        let url = format!("{}/api/health", self.base_url);
        match Httpc::get(self.auth_store(), &url, None) {
            Ok(mut response) => Ok(response
                .body_mut()
                .with_config()
                .limit(MAX_BODY_SIZE)
                .read_json::<HealthCheckResponse>()?),
            Err(e) => Err(anyhow!("{}", e)),
        }
    }

    pub fn auth_with_password(
        &self,
        collection: &str,
        identifier: &str,
        secret: &str,
    ) -> Result<Self> {
        let url = format!(
            "{}/api/collections/{}/auth-with-password",
            self.base_url, collection
        );

        let auth_payload = json!({
            "identity": identifier,
            "password": secret
        });

        match Httpc::post(self.auth_store(), &url, auth_payload.to_string()) {
            Ok(mut response) => {
                let response = response
                    .body_mut()
                    .with_config()
                    .limit(MAX_BODY_SIZE)
                    .read_json::<AuthSuccessResponse<serde_json::Value>>()?;

                Ok(Self {
                    base_url: self.base_url.clone(),
                    auth: Some(AuthStore {
                        base_url: self.base_url.clone(),
                        record: response.record.clone(),
                        token: response.token.clone(),
                    }),
                })
            }
            Err(e) => Err(anyhow!("{}", e)),
        }
    }
}

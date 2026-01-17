use crate::httpc::MAX_BODY_SIZE;
use crate::records::RecordId;
use crate::{collections::CollectionsManager, httpc::Httpc};
use crate::{logs::LogsManager, records::RecordsManager};
use anyhow::{anyhow, Result};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;

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

#[derive(Debug, Clone)]
pub struct Auth {
    base_url: String,
    auth_store: AuthStore,
    record_value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthStore {
    record: AuthRecord,
    pub(crate) token: String,
}

impl Auth {
    pub fn record<T: Serialize + DeserializeOwned>(&self) -> Result<(AuthBaseFields, T)> {
        Ok((
            self.auth_store.record.base_fields.clone(),
            serde_json::from_value(self.record_value.clone())?,
        ))
    }

    pub fn refresh(&mut self) -> Result<()> {
        let url = format!(
            "{}/api/collections/{}/auth-refresh",
            self.base_url, self.auth_store.record.base_fields.collection_name
        );

        match Httpc::post(Some(&self.auth_store), &url, "".to_string()) {
            Ok(mut response) => {
                let response = response
                    .body_mut()
                    .with_config()
                    .limit(MAX_BODY_SIZE)
                    .read_json::<AuthSuccessResponse>()?;

                self.auth_store.record = serde_json::from_value(response.record_value.clone())?;
                self.record_value = response.record_value.clone();
                self.auth_store.token = response.token.clone();

                Ok(())
            }
            Err(e) => Err(anyhow!("{}", e)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Client {
    base_url: String,
    auth: Option<Auth>,
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

    pub fn new_with_auth(base_url: &str, auth: AuthStore) -> Result<Self> {
        Ok(Self {
            base_url: base_url.to_string(),
            auth: Some(Auth {
                base_url: base_url.to_string(),
                record_value: serde_json::to_value(&auth.record)?,
                auth_store: auth,
            }),
        })
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn auth(&self) -> Option<&Auth> {
        self.auth.as_ref()
    }

    pub fn auth_store(&self) -> Option<&AuthStore> {
        self.auth.as_ref().map(|auth| &auth.auth_store)
    }

    pub fn auth_token(&self) -> Option<&str> {
        self.auth
            .as_ref()
            .map(|auth| auth.auth_store.token.as_str())
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
                    .read_json::<AuthSuccessResponse>()?;

                Ok(Self {
                    base_url: self.base_url.clone(),
                    auth: Some(Auth {
                        base_url: self.base_url.clone(),
                        auth_store: AuthStore {
                            record: serde_json::from_value::<AuthRecord>(
                                response.clone().record_value,
                            )?,
                            token: response.token.clone(),
                        },
                        record_value: response.record_value.clone(),
                    }),
                })
            }
            Err(e) => Err(anyhow!("{}", e)),
        }
    }
}

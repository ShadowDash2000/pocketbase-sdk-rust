use crate::auth::AuthenticatedRequest;
use crate::httpc::HttpClient;
use anyhow::{anyhow, Result};
use serde::Serialize;
use serde::{de::DeserializeOwned, Deserialize};

pub type RecordId = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordBaseFields {
    #[serde(default)]
    pub id: RecordId,
    #[serde(default, rename = "collectionName")]
    pub collection_name: String,
    #[serde(default, rename = "collectionId")]
    pub collection_id: String,
}

#[derive(Debug, Clone)]
pub struct RecordsManager<'a> {
    pub(crate) client: &'a HttpClient,
    pub(crate) token: Option<&'a str>,
    pub collection_name: &'a str,
}

#[derive(Debug, Clone)]
pub struct RecordsListRequestBuilder<'a> {
    pub(crate) client: &'a HttpClient,
    pub(crate) token: Option<&'a str>,
    pub collection_name: &'a str,
    pub filter: Option<String>,
    pub sort: Option<String>,
    pub expand: Option<String>,
    pub fields: Option<String>,
    pub page: i32,
    pub per_page: i32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordList<T> {
    pub page: i32,
    pub per_page: i32,
    pub total_items: i32,
    pub items: Vec<T>,
}

impl<'a> RecordsListRequestBuilder<'a> {
    pub async fn call<T: Default + DeserializeOwned>(&self) -> Result<RecordList<T>> {
        let url = format!("/api/collections/{}/records", self.collection_name);

        let mut build_opts: Vec<(&str, &str)> = vec![];
        if let Some(filter_opts) = &self.filter {
            build_opts.push(("filter", filter_opts))
        }
        if let Some(sort_opts) = &self.sort {
            build_opts.push(("sort", sort_opts))
        }
        if let Some(expand_opts) = &self.expand {
            build_opts.push(("expand", expand_opts))
        }
        if let Some(fields_opts) = &self.fields {
            build_opts.push(("fields", fields_opts))
        }
        let per_page_opts = self.per_page.to_string();
        let page_opts = self.page.to_string();
        build_opts.push(("perPage", per_page_opts.as_str()));
        build_opts.push(("page", page_opts.as_str()));

        match self
            .client
            .get(&url, Some(build_opts))
            .try_attach_auth_info(self.token)
            .send()
            .await
        {
            Ok(result) => Ok(result.json::<RecordList<T>>().await?),
            Err(e) => Err(e.into()),
        }
    }

    pub fn filter(&self, filter_opts: &str) -> Self {
        Self {
            filter: Some(filter_opts.to_string()),
            ..self.clone()
        }
    }

    pub fn sort(&self, sort_opts: &str) -> Self {
        Self {
            sort: Some(sort_opts.to_string()),
            ..self.clone()
        }
    }

    pub fn expand(&self, expand_opts: &str) -> Self {
        Self {
            expand: Some(expand_opts.to_string()),
            ..self.clone()
        }
    }

    pub fn fields(&self, fields_opts: &str) -> Self {
        Self {
            fields: Some(fields_opts.to_string()),
            ..self.clone()
        }
    }

    pub fn page(&self, page: i32) -> Self {
        Self {
            page,
            ..self.clone()
        }
    }

    pub fn per_page(&self, per_page: i32) -> Self {
        Self {
            per_page,
            ..self.clone()
        }
    }

    pub async fn full_list<T: Default + DeserializeOwned>(&self) -> Result<Vec<T>> {
        let mut result: Vec<T> = vec![];
        let mut page = 1;
        let per_page = self.per_page;

        loop {
            let list = self.page(page).call::<T>().await?;
            let items_len = list.items.len();
            let total_items = list.total_items as usize;
            result.extend(list.items);

            if items_len < per_page as usize || result.len() >= total_items {
                break;
            }
            page += 1;
        }

        Ok(result)
    }
}

#[derive(Debug, Clone)]
pub struct RecordViewRequestBuilder<'a> {
    pub(crate) client: &'a HttpClient,
    pub(crate) token: Option<&'a str>,
    pub collection_name: &'a str,
    pub identifier: &'a str,
    pub expand: Option<String>,
    pub fields: Option<String>,
}

impl<'a> RecordViewRequestBuilder<'a> {
    pub async fn call<T: Default + DeserializeOwned>(&self) -> Result<T> {
        let url = format!(
            "/api/collections/{}/records/{}",
            self.collection_name, self.identifier
        );

        let mut build_opts: Vec<(&str, &str)> = vec![];
        if let Some(expand_opts) = &self.expand {
            build_opts.push(("expand", expand_opts))
        }
        if let Some(fields_opts) = &self.fields {
            build_opts.push(("fields", fields_opts))
        }

        match self
            .client
            .get(&url, Some(build_opts))
            .try_attach_auth_info(self.token)
            .send()
            .await
        {
            Ok(result) => Ok(result.json::<T>().await?),
            Err(e) => Err(anyhow!("error: {}", e)),
        }
    }

    pub fn expand(&self, expand_opts: &str) -> Self {
        Self {
            expand: Some(expand_opts.to_string()),
            ..self.clone()
        }
    }

    pub fn fields(&self, fields_opts: &str) -> Self {
        Self {
            fields: Some(fields_opts.to_string()),
            ..self.clone()
        }
    }
}

#[derive(Clone, Debug)]
pub struct RecordDestroyRequestBuilder<'a> {
    pub(crate) client: &'a HttpClient,
    pub(crate) token: Option<&'a str>,
    pub collection_name: &'a str,
    pub identifier: &'a str,
}

impl<'a> RecordDestroyRequestBuilder<'a> {
    pub async fn call(&self) -> Result<()> {
        let url = format!(
            "/api/collections/{}/records/{}",
            self.collection_name, self.identifier
        );
        match self
            .client
            .delete(&url)
            .try_attach_auth_info(self.token)
            .send()
            .await
        {
            Ok(result) => {
                if result.status() == 204 {
                    Ok(())
                } else {
                    Err(anyhow!("Failed to delete"))
                }
            }
            Err(e) => Err(anyhow!("error: {}", e)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RecordDeleteAllRequestBuilder<'a> {
    pub(crate) client: &'a HttpClient,
    pub(crate) token: Option<&'a str>,
    pub collection_name: &'a str,
    pub filter: Option<&'a str>,
}

#[derive(Debug, Clone)]
pub struct RecordCreateRequestBuilder<'a, T: Serialize + Clone> {
    pub(crate) client: &'a HttpClient,
    pub(crate) token: Option<&'a str>,
    pub collection_name: &'a str,
    pub record: T,
}

#[derive(Deserialize, Clone, Debug)]
pub struct CreateResponse {
    #[serde(rename = "collectionName")]
    pub collection_name: Option<String>,
    #[serde(rename = "collectionId")]
    pub collection_id: Option<String>,
    pub id: String,
    pub updated: String,
    pub created: String,
}

impl<'a, T: Serialize + Clone> RecordCreateRequestBuilder<'a, T> {
    pub async fn call(&self) -> Result<CreateResponse> {
        let url = format!("/api/collections/{}/records", self.collection_name);
        let payload = serde_json::to_string(&self.record).map_err(anyhow::Error::from)?;
        match self
            .client
            .post(&url, payload)
            .try_attach_auth_info(self.token)
            .send()
            .await
        {
            Ok(result) => Ok(result.json::<CreateResponse>().await?),
            Err(e) => Err(anyhow!("error: {}", e)),
        }
    }
}

pub struct RecordUpdateRequestBuilder<'a, K: Serialize + Clone> {
    pub(crate) client: &'a HttpClient,
    pub(crate) token: Option<&'a str>,
    pub collection_name: &'a str,
    pub id: &'a str,
    pub data: K,
}

impl<'a, K: Serialize + Clone> RecordUpdateRequestBuilder<'a, K> {
    pub async fn call<T: Default + DeserializeOwned>(&self) -> Result<T> {
        let url = format!(
            "/api/collections/{}/records/{}",
            self.collection_name, self.id
        );
        let payload = serde_json::to_string(&self.data).map_err(anyhow::Error::from)?;
        match self
            .client
            .patch(&url, payload)
            .try_attach_auth_info(self.token)
            .send()
            .await
        {
            Ok(result) => Ok(result.json::<T>().await?),
            Err(e) => Err(anyhow!("error: {}", e)),
        }
    }

    pub async fn send(&self) -> Result<()> {
        let url = format!(
            "/api/collections/{}/records/{}",
            self.collection_name, self.id
        );
        let payload = serde_json::to_string(&self.data).map_err(anyhow::Error::from)?;
        match self
            .client
            .patch(&url, payload)
            .try_attach_auth_info(self.token)
            .send()
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow!("error: {}", e)),
        }
    }
}

impl<'a> RecordsManager<'a> {
    pub fn view(&self, identifier: &'a str) -> RecordViewRequestBuilder<'a> {
        RecordViewRequestBuilder {
            client: self.client,
            token: self.token,
            collection_name: self.collection_name,
            identifier,
            expand: None,
            fields: None,
        }
    }

    pub fn destroy(&self, identifier: &'a str) -> RecordDestroyRequestBuilder<'a> {
        RecordDestroyRequestBuilder {
            client: self.client,
            token: self.token,
            collection_name: self.collection_name,
            identifier,
        }
    }

    pub fn update<K: Serialize + Clone>(
        &self,
        identifier: &'a str,
        data: K,
    ) -> RecordUpdateRequestBuilder<'a, K> {
        RecordUpdateRequestBuilder {
            client: self.client,
            token: self.token,
            collection_name: self.collection_name,
            id: identifier,
            data,
        }
    }

    pub fn create<T: Serialize + Clone>(&self, record: T) -> RecordCreateRequestBuilder<'a, T> {
        RecordCreateRequestBuilder {
            client: self.client,
            token: self.token,
            collection_name: self.collection_name,
            record,
        }
    }

    pub fn list(&self) -> RecordsListRequestBuilder<'a> {
        RecordsListRequestBuilder {
            client: self.client,
            token: self.token,
            collection_name: self.collection_name,
            filter: None,
            sort: None,
            expand: None,
            fields: None,
            page: 1,
            per_page: 100,
        }
    }

    pub async fn full_list<T: Default + DeserializeOwned>(&self) -> Result<Vec<T>> {
        self.list().full_list::<T>().await
    }
}

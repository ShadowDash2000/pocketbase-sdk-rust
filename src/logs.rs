use crate::auth::AuthenticatedRequest;
use crate::httpc::HttpClient;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::collections::HashMap;

pub struct LogsManager<'a> {
    pub(crate) client: &'a HttpClient,
    pub(crate) token: &'a str,
}

#[derive(Debug, Clone)]
pub struct LogListRequestBuilder<'a> {
    pub(crate) client: &'a HttpClient,
    pub(crate) token: &'a str,
    pub page: i32,
    pub per_page: i32,
    pub sort: Option<&'a str>,
    pub filter: Option<&'a str>,
}

#[derive(Debug, Clone)]
pub struct LogViewRequestBuilder<'a> {
    pub(crate) client: &'a HttpClient,
    pub(crate) token: &'a str,
    pub id: &'a str,
}

#[derive(Debug, Clone)]
pub struct LogStatisticsRequestBuilder<'a> {
    pub(crate) client: &'a HttpClient,
    pub(crate) token: &'a str,
    pub filter: Option<&'a str>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogListItem {
    pub id: String,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub url: String,
    pub method: String,
    pub status: i32,
    pub ip: Option<String>,
    pub referer: String,
    pub user_agent: String,
    pub meta: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogList {
    pub page: i32,
    pub per_page: i32,
    pub total_items: i32,
    pub items: Vec<LogListItem>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LogStatDataPoint {
    pub total: i32,
    pub date: String,
}

impl<'a> LogStatisticsRequestBuilder<'a> {
    pub fn filter(&self, filter_query: &'a str) -> Self {
        Self {
            filter: Some(filter_query),
            ..self.clone()
        }
    }

    pub async fn call(&self) -> Result<Vec<LogStatDataPoint>> {
        let mut build_opts = Vec::new();
        if let Some(filter_opts) = &self.filter {
            build_opts.push(("filter", filter_opts.to_owned()));
        }

        match self
            .client
            .get("/api/logs/requests/stats", Some(build_opts))
            .attach_auth_info(self.token)
            .send()
            .await
        {
            Ok(result) => Ok(result.json::<Vec<LogStatDataPoint>>().await?),
            Err(e) => Err(e.into()),
        }
    }
}

impl<'a> LogViewRequestBuilder<'a> {
    pub async fn call(&self) -> Result<LogListItem> {
        let url = format!("/api/logs/requests/{}", self.id);
        match self
            .client
            .get(&url, None)
            .attach_auth_info(self.token)
            .send()
            .await
        {
            Ok(result) => Ok(result.json::<LogListItem>().await?),
            Err(e) => Err(e.into()),
        }
    }
}

impl<'a> LogListRequestBuilder<'a> {
    pub fn page(&self, page_count: i32) -> Self {
        LogListRequestBuilder {
            page: page_count,
            ..self.clone()
        }
    }

    pub fn per_page(&self, per_page_count: i32) -> Self {
        LogListRequestBuilder {
            per_page: per_page_count,
            ..self.clone()
        }
    }

    pub fn filter(&self, filter_opts: &'a str) -> Self {
        LogListRequestBuilder {
            filter: Some(filter_opts),
            ..self.clone()
        }
    }

    pub fn sort(&self, sort_opts: &'a str) -> Self {
        LogListRequestBuilder {
            sort: Some(sort_opts),
            ..self.clone()
        }
    }

    pub async fn call(&self) -> Result<LogList> {
        let mut build_opts = Vec::new();

        if let Some(sort_opts) = &self.sort {
            build_opts.push(("sort", sort_opts.to_owned()))
        }
        if let Some(filter_opts) = &self.filter {
            build_opts.push(("filter", filter_opts.to_owned()))
        }
        let per_page_opts = self.per_page.to_string();
        let page_opts = self.page.to_string();
        build_opts.push(("perPage", per_page_opts.as_str()));
        build_opts.push(("page", page_opts.as_str()));

        match self
            .client
            .get("/api/logs/requests", Some(build_opts))
            .attach_auth_info(self.token)
            .send()
            .await
        {
            Ok(result) => Ok(result.json::<LogList>().await?),
            Err(e) => Err(e.into()),
        }
    }
}

impl<'a> LogsManager<'a> {
    pub fn list(&self) -> LogListRequestBuilder<'a> {
        LogListRequestBuilder {
            client: self.client,
            token: self.token,
            page: 1,
            per_page: 100,
            sort: None,
            filter: None,
        }
    }

    pub fn view(&self, id: &'a str) -> LogViewRequestBuilder<'a> {
        LogViewRequestBuilder {
            client: self.client,
            token: self.token,
            id,
        }
    }

    pub fn statistics(&self) -> LogStatisticsRequestBuilder<'a> {
        LogStatisticsRequestBuilder {
            client: self.client,
            token: self.token,
            filter: None,
        }
    }
}

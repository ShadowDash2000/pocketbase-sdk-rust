use reqwest::RequestBuilder;

#[derive(Debug, Clone)]
pub struct HttpClient {
    base_url: String,
    client: reqwest::Client,
}

impl HttpClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    fn build_url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    pub fn get(&self, url: &str, query_params: Option<Vec<(&str, &str)>>) -> RequestBuilder {
        let mut builder = self.client.get(self.build_url(url));

        if let Some(pairs) = query_params {
            builder = builder.query(pairs.as_slice());
        }

        builder
    }

    pub fn post(&self, url: &str, body_content: String) -> RequestBuilder {
        self.client
            .post(self.build_url(url))
            .header("Content-Type", "application/json")
            .body(body_content)
    }

    pub fn delete(&self, url: &str) -> RequestBuilder {
        self.client.delete(self.build_url(url))
    }

    pub fn patch(&self, url: &str, body_content: String) -> RequestBuilder {
        self.client
            .patch(self.build_url(url))
            .header("Content-Type", "application/json")
            .body(body_content)
    }
}

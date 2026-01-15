use crate::client::Client;
use anyhow::Result;
use ureq::http::header::AUTHORIZATION;
use ureq::http::Response;
use ureq::{Body, Error, RequestBuilder};

pub struct Httpc;

pub const MAX_BODY_SIZE: u64 = 10 << 20;

pub trait AuthenticatedRequest {
    fn attach_auth_info<T>(self, client: &Client<T>) -> Self;
}

impl<B> AuthenticatedRequest for RequestBuilder<B> {
    fn attach_auth_info<T>(self, client: &Client<T>) -> Self {
        if let Some(token) = client.auth_token() {
            return self.header(AUTHORIZATION, token);
        }
        self
    }
}

impl Httpc {
    pub fn get<T>(
        client: &Client<T>,
        url: &str,
        query_params: Option<Vec<(&str, &str)>>,
    ) -> Result<Response<Body>, Error> {
        let mut builder = ureq::get(url);

        if let Some(pairs) = query_params {
            for (k, v) in pairs {
                builder = builder.query(k, v);
            }
        }

        builder.attach_auth_info(client).call()
    }

    pub fn post<T>(
        client: &Client<T>,
        url: &str,
        body_content: String,
    ) -> Result<Response<Body>, Error> {
        ureq::post(url)
            .header("Content-Type", "application/json")
            .attach_auth_info(client)
            .send(body_content.as_str())
    }

    pub fn delete<T>(client: &Client<T>, url: &str) -> Result<Response<Body>, Error> {
        ureq::delete(url).attach_auth_info(client).call()
    }

    pub fn patch<T>(
        client: &Client<T>,
        url: &str,
        body_content: String,
    ) -> Result<Response<Body>, Error> {
        ureq::patch(url)
            .header("Content-Type", "application/json")
            .attach_auth_info(client)
            .send(body_content.as_str())
    }
}

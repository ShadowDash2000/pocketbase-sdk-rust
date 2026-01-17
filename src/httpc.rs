use crate::client::AuthStore;
use anyhow::Result;
use ureq::http::header::AUTHORIZATION;
use ureq::http::Response;
use ureq::{Body, Error, RequestBuilder};

pub struct Httpc;

pub const MAX_BODY_SIZE: u64 = 10 << 20; // 10 mb

pub trait AuthenticatedRequest {
    fn attach_auth_info(self, auth_store: Option<&AuthStore>) -> Self;
}

impl<B> AuthenticatedRequest for RequestBuilder<B> {
    fn attach_auth_info(self, auth_store: Option<&AuthStore>) -> Self {
        if let Some(auth_store) = auth_store {
            return self.header(AUTHORIZATION, auth_store.token.clone());
        }
        self
    }
}

impl Httpc {
    pub fn get(
        auth_store: Option<&AuthStore>,
        url: &str,
        query_params: Option<Vec<(&str, &str)>>,
    ) -> Result<Response<Body>, Error> {
        let mut builder = ureq::get(url);

        if let Some(pairs) = query_params {
            for (k, v) in pairs {
                builder = builder.query(k, v);
            }
        }

        builder.attach_auth_info(auth_store).call()
    }

    pub fn post(
        auth_store: Option<&AuthStore>,
        url: &str,
        body_content: String,
    ) -> Result<Response<Body>, Error> {
        ureq::post(url)
            .header("Content-Type", "application/json")
            .attach_auth_info(auth_store)
            .send(body_content.as_str())
    }

    pub fn delete(auth_store: Option<&AuthStore>, url: &str) -> Result<Response<Body>, Error> {
        ureq::delete(url).attach_auth_info(auth_store).call()
    }

    pub fn patch(
        auth_store: Option<&AuthStore>,
        url: &str,
        body_content: String,
    ) -> Result<Response<Body>, Error> {
        ureq::patch(url)
            .header("Content-Type", "application/json")
            .attach_auth_info(auth_store)
            .send(body_content.as_str())
    }
}

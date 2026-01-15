use crate::client::Auth;
use crate::client::Client;
use crate::httpc::{Httpc, MAX_BODY_SIZE};
use anyhow::{anyhow, Result};
use serde::Deserialize;
use serde_json::json;

pub struct Admin<'a> {
    base_url: &'a str,
}

#[derive(Debug, Clone, Deserialize)]
struct AuthSuccessResponse {
    token: String,
}

impl<'a> Admin<'a> {
    pub fn auth_with_password(&self, identifier: &str, secret: &str) -> Result<Client<Auth>> {
        let url = format!("{}/api/admins/auth-with-password", self.base_url);
        let credentials = json!({
            "identity": identifier,
            "password": secret,
        });
        let client = Client::new(self.base_url);
        match Httpc::post(&client, &url, credentials.to_string()) {
            Ok(mut response) => {
                let raw_response = response
                    .body_mut()
                    .with_config()
                    .limit(MAX_BODY_SIZE)
                    .read_json::<AuthSuccessResponse>();
                match raw_response {
                    Ok(AuthSuccessResponse { token }) => {
                        Ok(Client::new_auth(self.base_url, &token))
                    }
                    Err(e) => Err(anyhow!("{}", e)),
                }
            }
            Err(e) => Err(anyhow!("{}", e)),
        }
    }

    pub fn new(base_url: &'a str) -> Admin<'a> {
        Admin { base_url }
    }
}

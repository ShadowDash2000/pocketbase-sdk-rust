use httpmock::prelude::*;
use pocketbase_sdk::client::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;

#[derive(Clone, Debug, Serialize, Default, Deserialize)]
pub struct Record {
    pub id: String,
    pub title: String,
}

#[test]
fn list_records_success() {
    let mockserver = mock_records_server();
    let client = Client::new(mockserver.base_url().as_str())
        .auth_with_password("users", "sreedev@icloud.com", "Sreedev123")
        .unwrap();

    let records = client
        .records("posts")
        .list()
        .per_page(1000)
        .call::<Record>();
    assert!(records.is_ok());
}

#[test]
fn full_list_records_success() {
    let mockserver = mock_records_server();
    let client = Client::new(mockserver.base_url().as_str())
        .auth_with_password("users", "sreedev@icloud.com", "Sreedev123")
        .unwrap();

    let records = client
        .records("posts")
        .list()
        .per_page(1000)
        .full_list::<Record>()
        .unwrap();
    assert_eq!(records.len(), 3);
    assert_eq!(records[0].id, "1");
    assert_eq!(records[1].id, "2");
    assert_eq!(records[2].id, "3");
}

fn mock_records_server() -> MockServer {
    let server = MockServer::start();

    let items_data =
        fs::read_to_string("tests/items.json").expect("Unable to read tests/items.json");
    let items: Vec<Value> = serde_json::from_str(&items_data).expect("Unable to parse items.json");
    let total_items = items.len();

    server.mock(|when, then| {
        when.method(GET)
            .path("/api/collections/posts/records")
            .query_param("page", "1")
            .query_param("perPage", "1000")
            .header("Authorization", "eyJhbGciOiJIUzI1NiJ9.eyJpZCI6IjRxMXhsY2xtZmxva3UzMyIsInR5cGUiOiJhdXRoUmVjb3JkIiwiY29sbGVjdGlvbklkIjoiX3BiX3VzZXJzX2F1dGhfIiwiZXhwIjoyMjA4OTg1MjYxfQ.UwD8JvkbQtXpymT09d7J6fdA0aP9g4FJ1GPh_ggEkzc");
        then.header("Content-Type", "application/json")
            .json_body(json!({
                "page": 1,
                "perPage": 1000,
                "totalItems": total_items,
                "items": items
            }));
    });

    server.mock(|when, then| {
        when
            .method(POST)
            .json_body(json!({
                "identity": "sreedev@icloud.com",
                "password": "Sreedev123"
            }))
            .path("/api/collections/users/auth-with-password");

        then
            .status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                    "token": "eyJhbGciOiJIUzI1NiJ9.eyJpZCI6IjRxMXhsY2xtZmxva3UzMyIsInR5cGUiOiJhdXRoUmVjb3JkIiwiY29sbGVjdGlvbklkIjoiX3BiX3VzZXJzX2F1dGhfIiwiZXhwIjoyMjA4OTg1MjYxfQ.UwD8JvkbQtXpymT09d7J6fdA0aP9g4FJ1GPh_ggEkzc",
                    "record": {
                    "id": "8171022dc95a4ed",
                    "collectionId": "d2972397d45614e",
                    "collectionName": "users",
                    "created": "2022-06-24 06:24:18.434Z",
                    "updated": "2022-06-24 06:24:18.889Z",
                    "username": "test@example.com",
                    "email": "test@example.com",
                    "verified": false,
                    "emailVisibility": true,
                    "someCustomField": "example 123"
                }
            }));
    });
    server
}

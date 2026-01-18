use anyhow::Result;
use pocketbase_sdk::client::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub count: i32,
    pub created: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NewProduct {
    pub name: String,
    pub count: i32,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    /* Authenticate Client */
    let authenticated_client = Client::new("http://localhost:8090")
        .auth_with_password("users", "sreedev@icloud.com", "Sreedev123")
        .await?;

    /* List Products */
    let products = authenticated_client
        .records("products")
        .list()
        .call::<Product>()
        .await?;
    dbg!(products);

    /* List Products with filter */
    let filtered_products = authenticated_client
        .records("products")
        .list()
        .filter("count < 6000")
        .call::<Product>()
        .await?;
    dbg!(filtered_products);

    /* View Product */
    let product = authenticated_client
        .records("products")
        .view("jme4ixxqie2f9ho")
        .call::<Product>()
        .await?;
    dbg!(product);

    /* Create Product */
    let new_product = NewProduct {
        name: String::from("bingo"),
        count: 69420,
    };
    let create_response = authenticated_client
        .records("products")
        .create(new_product)
        .call()
        .await?;
    dbg!(&create_response);

    /* Update Product */
    let updated_product = NewProduct {
        name: String::from("bango"),
        count: 69420,
    };
    let update_response = authenticated_client
        .records("products")
        .update(create_response.id.as_str(), updated_product)
        .call::<Product>()
        .await?;

    dbg!(update_response);

    /* Delete Product */
    authenticated_client
        .records("products")
        .destroy(create_response.id.as_str())
        .call()
        .await?;

    Ok(())
}

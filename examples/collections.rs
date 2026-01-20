use anyhow::Result;
use pocketbase_sdk::client::Client;

#[tokio::main]
async fn main() -> Result<()> {
    // admin authentication
    let client = Client::new("http://localhost:8090")
        .auth_with_password("_superusers", "sreedev@icloud.com", "Sreedev123")
        .await?;

    // collections list + Filter
    let collections = client
        .collections()
        .list()
        .page(1)
        .filter("name = 'employees'".to_string())
        .per_page(100)
        .call()
        .await?;

    dbg!(collections);

    // view collection
    let user_collection = client.collections().view("users").call().await?;

    dbg!(user_collection);

    Ok(())
}

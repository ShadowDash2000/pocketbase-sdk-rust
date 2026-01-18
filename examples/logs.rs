use anyhow::Result;
use pocketbase_sdk::client::Client;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    // admin authentication
    let client = Client::new("http://localhost:8090")
        .auth_with_password("_superusers", "sreedev@icloud.com", "Sreedev123")
        .await?;

    // list logs
    let logs = client.logs().list().page(1).per_page(10).call().await?;
    dbg!(&logs);

    // view log
    let somelogid = &logs.items[0].id;
    let logitem = client.logs().view(somelogid).call().await?;
    dbg!(logitem);

    // view log statistics data points
    let logstats = client.logs().statistics().call().await?;
    dbg!(logstats);

    Ok(())
}

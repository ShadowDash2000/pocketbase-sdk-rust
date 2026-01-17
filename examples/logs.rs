use anyhow::Result;
use pocketbase_sdk::client::Client;

fn main() -> Result<()> {
    env_logger::init();

    // admin authentication
    let client = Client::new("http://localhost:8090").auth_with_password(
        "_superusers",
        "sreedev@icloud.com",
        "Sreedev123",
    )?;

    // list logs
    let logs = client.logs().list().page(1).per_page(10).call()?;
    dbg!(&logs);

    // view log
    let somelogid = &logs.items[0].id;
    let logitem = client.logs().view(somelogid).call()?;
    dbg!(logitem);

    // view log statistics data points
    let logstats = client.logs().statistics().call()?;
    dbg!(logstats);

    Ok(())
}

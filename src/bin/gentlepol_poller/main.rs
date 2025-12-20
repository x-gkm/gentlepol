use std::env;
use futures::StreamExt;

use gentlepol::db::Db;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    _ = dotenvy::dotenv();

    let db_url = env::var("DATABASE_URL")?;

    let db = Db::build(&db_url).await?;

    let mut stream = db.get_all_web_news();

    while let Some(fetched) = stream.next().await {
        let web_news = fetched?;
        println!("{web_news:?}");
    }

    Ok(())
}

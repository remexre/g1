use anyhow::Result;
use defer::defer;
use futures::prelude::*;
use g1::{Connection, SqliteConnection};

#[tokio::main]
async fn main() -> Result<()> {
    let mut db_dir = std::env::temp_dir();
    db_dir.push("g1-blobs-example");
    std::fs::create_dir_all(&db_dir)?;
    let _guard = defer(|| {
        std::fs::remove_dir_all(&db_dir).unwrap();
    });

    let data = b"example".to_vec();

    let conn = SqliteConnection::open(db_dir.clone()).await?;
    let hash = conn
        .store_blob(stream::once(future::ok(data.into())).boxed())
        .await?;
    let data = conn
        .fetch_blob(hash)
        .await?
        .map_ok(|bs| bs.into_iter().collect::<Vec<_>>())
        .try_concat()
        .await?;

    assert_eq!(data, b"example");

    Ok(())
}

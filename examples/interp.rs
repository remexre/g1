use anyhow::Result;
use defer::defer;
use g1::{query, Connection, SqliteConnection};
use std::sync::Arc;
use tokio;

#[tokio::main]
async fn main() -> Result<()> {
    let mut db_dir = std::env::temp_dir();
    db_dir.push("g1-macro-example");
    std::fs::create_dir_all(&db_dir)?;
    let _guard = defer(|| {
        std::fs::remove_dir_all(&db_dir).unwrap();
    });

    let conn = SqliteConnection::open(db_dir.clone()).await?;

    let lang = "English";
    let solns = conn
        .query_all(query! {
            hello("English", "Hello").
            hello("Spanish", "Hola").
            hello("French", "Bonjour").

            ?- hello($lang, X).
        })
        .await?;

    assert_eq!(solns, vec![vec![Arc::from("English"), Arc::from("Hello")],]);

    Ok(())
}

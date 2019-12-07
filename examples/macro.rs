use anyhow::Result;
use defer::defer;
use g1::{query, Connection, G1SqliteConnection};
use std::sync::Arc;
use tokio;

#[tokio::main]
async fn main() -> Result<()> {
    let mut db_dir = std::env::temp_dir();
    db_dir.push("g1-macro-example");
    std::fs::create_dir_all(&db_dir)?;
    defer(|| {
        std::fs::remove_dir_all(&db_dir).unwrap();
    });

    let conn = G1SqliteConnection::open(db_dir.clone()).await?;
    let mut solns = conn
        .query_all(query! {
            edge("A", "B").
            edge("B", "C").
            edge("C", "D").
            edge("D", "B").

            path(X, Y) :- edge(X, Y).
            path(X, Z) :- edge(X, Y), path(Y, Z).

            ?- path("D", X).
        })
        .await?;

    solns.sort();
    assert_eq!(
        solns,
        vec![
            vec![Arc::from("D"), Arc::from("B")],
            vec![Arc::from("D"), Arc::from("C")],
            vec![Arc::from("D"), Arc::from("D")],
        ]
    );

    Ok(())
}

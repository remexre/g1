//! Keep this example synced with the `README.md` and `lib.rs` doc-comment.

use anyhow::{anyhow, Result};
use defer::defer;
use futures::prelude::*;
use g1::{query, Atom, Connection, SqliteConnection};

#[tokio::main]
async fn main() -> Result<()> {
    let mut db_dir = std::env::temp_dir();
    db_dir.push("g1-readme-example");
    std::fs::create_dir_all(&db_dir)?;
    let _guard = defer(|| {
        std::fs::remove_dir_all(&db_dir).unwrap();
    });

    let conn = SqliteConnection::open(db_dir.clone()).await?;

    conn.create_name(conn.create_atom().await?, "example/readme", "foo", false)
        .await?;
    conn.create_name(conn.create_atom().await?, "example/readme", "bar", false)
        .await?;

    let foo = conn
        .query_first(query! { ?- name(Atom, "example/readme", "foo"). })
        .await?
        .ok_or_else(|| anyhow!("couldn't find foo"))?[0]
        .parse()?;
    let bar = conn
        .query_first(query! { ?- name(Atom, "example/readme", "bar"). })
        .await?
        .ok_or_else(|| anyhow!("couldn't find bar"))?[0]
        .parse()?;

    conn.create_name(bar, "other namespace", "bar", false)
        .await?;

    assert_eq!(
        conn.query_first(query! { ?- name(Atom, "other namespace", "bar"). })
            .await?
            .ok_or_else(|| anyhow!("couldn't find bar"))?[0]
            .parse::<Atom>()?,
        bar
    );

    conn.create_edge(foo, bar, "next").await?;
    conn.create_edge(bar, foo, "prev").await?;

    let edges = conn.query_all(query! { ?- edge(From, To, Label). }).await?;
    assert!(edges.contains(&vec![
        foo.to_string().into(),
        bar.to_string().into(),
        "next".to_string().into()
    ]));
    assert!(edges.contains(&vec![
        bar.to_string().into(),
        foo.to_string().into(),
        "prev".to_string().into()
    ]));

    conn.create_tag(foo, "letters", "3", false).await?;
    let hash = conn
        .store_blob(stream::once(future::ok((b"bar" as &[_]).into())).boxed())
        .await?;
    conn.create_blob(
        bar,
        "name again",
        "text/plain".parse().unwrap(),
        hash,
        false,
    )
    .await?;

    Ok(())
}

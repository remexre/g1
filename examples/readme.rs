//! Keep this example synced with the `README.md`.

use anyhow::{anyhow, Result};
use g1::Connection;

#[tokio::main]
async fn main() -> Result<()> {
    let conn = Connection::open("http://localhost:61616/")?;

    conn.create_name(conn.create_atom().await?, "example/readme", "foo")
        .await?;
    conn.create_name(conn.create_atom().await?, "example/readme", "bar")
        .await?;

    let foo = conn
        .find_atom_by_name("example/readme", "foo")
        .await?
        .ok_or_else(|| anyhow!("couldn't find bar"))?;
    let bar = conn
        .find_atom_by_name("example/readme", "bar")
        .await?
        .ok_or_else(|| anyhow!("couldn't find bar"))?;

    conn.create_name(bar, "other namespace", "bar").await?;

    assert_eq!(
        conn.find_atom_by_name("other namespace", "bar").await?,
        Some(bar)
    );

    conn.create_edge(foo, bar, "next").await?;
    conn.create_edge(bar, foo, "prev").await?;

    let edges = conn.find_edges(None, None, None).await?;
    assert!(edges.contains(&(foo, bar, "next".to_string())));
    assert!(edges.contains(&(bar, foo, "prev".to_string())));

    unimplemented!("{:?}", conn)
}

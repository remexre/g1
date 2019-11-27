g1
==

A simple graph store.

Model
-----

**Atom**: Atoms are the nodes of the graph. Each is represented as a UUID.

**Name**: Names uniquely identify an atom. They have a namespace and a title, both of which are strings.

**Edge**: Edges are directed, with an atom at both endpoints. Edges have a string key associated with them. At most one edge between two atoms with a given key may exist.

**Tag**: Tags are attached to atoms. They have a kind and a value, both of which are strings.

**Blob**: Blobs are attached to atoms. They have a type, which is a MIME type, and contents, which is an arbitrarily large binary string.

Strings are UTF-8 strings, which should be no longer than 256 bytes.

Rust API
--------

For now, only simple operations are implemented:

```rust
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

    conn.create_tag(foo, "letters", "3").await?;
    conn.create_blob(bar, "text/plain".parse().unwrap(), b"bar")
        .await?;

    Ok(())
}
```

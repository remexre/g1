//! A simple graph store.
//!
//! Model
//! -----
//!
//! **Atom**: Atoms are the nodes of the graph. Each is represented as a UUID.
//!
//! **Name**: Names uniquely identify an atom. They have a namespace and a title, both of which are strings.
//!
//! **Edge**: Edges are directed, with an atom at both endpoints. Edges have a string label associated with them. At most one edge between two atoms with a given label may exist.
//!
//! **Tag**: Tags are attached to atoms. They have a key and a value, both of which are strings.
//!
//! **Blob**: Blobs are attached to atoms. They have a kind, which is a string; a type, which is a MIME type; and contents, which are an arbitrarily large binary string. Blobs are referred to by a SHA256 hash.
//!
//! Strings are UTF-8 strings, which should be no longer than 256 bytes.
//!
//! Rust API
//! --------
//!
//! For now, only simple operations are implemented:
//!
//! ```rust
//! use anyhow::{anyhow, Result};
//! use defer::defer;
//! use futures::prelude::*;
//! use g1::{query, Atom, Connection, SqliteConnection};
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let mut db_dir = std::env::temp_dir();
//!     db_dir.push("g1-readme-example");
//!     std::fs::create_dir_all(&db_dir)?;
//!     let _guard = defer(|| {
//!         std::fs::remove_dir_all(&db_dir).unwrap();
//!     });
//!
//!     let conn = SqliteConnection::open(db_dir.clone()).await?;
//!
//!     conn.create_name(conn.create_atom().await?, "example/readme", "foo", false)
//!         .await?;
//!     conn.create_name(conn.create_atom().await?, "example/readme", "bar", false)
//!         .await?;
//!
//!     let foo = conn
//!         .query_first(query! { ?- name(Atom, "example/readme", "foo"). })
//!         .await?
//!         .ok_or_else(|| anyhow!("couldn't find foo"))?[0]
//!         .parse()?;
//!     let bar = conn
//!         .query_first(query! { ?- name(Atom, "example/readme", "bar"). })
//!         .await?
//!         .ok_or_else(|| anyhow!("couldn't find bar"))?[0]
//!         .parse()?;
//!
//!     conn.create_name(bar, "other namespace", "bar", false)
//!         .await?;
//!
//!     assert_eq!(
//!         conn.query_first(query! { ?- name(Atom, "other namespace", "bar"). })
//!             .await?
//!             .ok_or_else(|| anyhow!("couldn't find bar"))?[0]
//!             .parse::<Atom>()?,
//!         bar
//!     );
//!
//!     conn.create_edge(foo, bar, "next").await?;
//!     conn.create_edge(bar, foo, "prev").await?;
//!
//!     let edges = conn.query_all(query! { ?- edge(From, To, Label). }).await?;
//!     assert!(edges.contains(&vec![
//!         foo.to_string().into(),
//!         bar.to_string().into(),
//!         "next".to_string().into()
//!     ]));
//!     assert!(edges.contains(&vec![
//!         bar.to_string().into(),
//!         foo.to_string().into(),
//!         "prev".to_string().into()
//!     ]));
//!
//!     conn.create_tag(foo, "letters", "3", false).await?;
//!     let hash = conn
//!         .store_blob(stream::once(future::ok((b"bar" as &[_]).into())).boxed())
//!         .await?;
//!     conn.create_blob(
//!         bar,
//!         "name again",
//!         "text/plain".parse().unwrap(),
//!         hash,
//!         false,
//!     )
//!     .await?;
//!
//!     Ok(())
//! }
//! ```
#![deny(
    bad_style,
    bare_trait_objects,
    const_err,
    dead_code,
    improper_ctypes,
    legacy_directory_ownership,
    missing_debug_implementations,
    missing_docs,
    no_mangle_generic_items,
    non_shorthand_field_patterns,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    plugin_as_library,
    private_in_public,
    safe_extern_statics,
    trivial_numeric_casts,
    unconditional_recursion,
    unions_with_drop_fields,
    unsafe_code,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_extern_crates,
    unused_import_braces,
    unused_parens,
    unused_qualifications,
    unused_results,
    while_true
)]

pub use g1_common::{
    nameless::{NamelessClause, NamelessPredicate, NamelessQuery, NamelessValue},
    Atom, Bytes, Connection, Hash, Mime,
};

/// Parses a query into a `NamelessQuery` literal.
#[proc_macro_hack::proc_macro_hack]
pub use g1_macros::query;
#[doc(hidden)]
pub use lazy_static::lazy_static;

#[cfg(feature = "g1-sqlite-connection")]
pub use g1_sqlite_connection::{SqliteConnection, SqliteError};

/// Useful utilities.
pub mod utils {
    /// Reads a file as a stream of chunks. Useful with `Connection::store_blob`.
    pub use g1_common::utils::file_to_stream;
}

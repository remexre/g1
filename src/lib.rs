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
//! use g1::Connection;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // TODO: Reenable me
//!     /*
//!     let conn = Connection::open("http://localhost:61616/")?;
//!
//!     conn.create_name(conn.create_atom().await?, "example/readme", "foo")
//!         .await?;
//!     conn.create_name(conn.create_atom().await?, "example/readme", "bar")
//!         .await?;
//!
//!     let foo = conn
//!         .find_atom_by_name("example/readme", "foo")
//!         .await?
//!         .ok_or_else(|| anyhow!("couldn't find bar"))?;
//!     let bar = conn
//!         .find_atom_by_name("example/readme", "bar")
//!         .await?
//!         .ok_or_else(|| anyhow!("couldn't find bar"))?;
//!
//!     conn.create_name(bar, "other namespace", "bar").await?;
//!
//!     assert_eq!(
//!         conn.find_atom_by_name("other namespace", "bar").await?,
//!         Some(bar)
//!     );
//!
//!     conn.create_edge(foo, bar, "next").await?;
//!     conn.create_edge(bar, foo, "prev").await?;
//!
//!     let edges = conn.find_edges(None, None, None).await?;
//!     assert!(edges.contains(&(foo, bar, "next".to_string())));
//!     assert!(edges.contains(&(bar, foo, "prev".to_string())));
//!
//!     conn.create_tag(foo, "letters", "3").await?;
//!     conn.create_blob(bar, "name again", "text/plain".parse().unwrap(), b"bar")
//!         .await?;
//!     */
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
pub use g1_sqlite_connection::{G1SqliteConnection, G1SqliteError};

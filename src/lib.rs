//! A simple graph store.
//!
//! Model
//! -----
//!
//! ### Atom
//!
//! Atoms are the nodes of the graph. Each is represented as a UUID.
//!
//! ### Names
//!
//! Names uniquely identify an atom. They have a namespace and a title, both of which are strings.
//!
//! ### Edge
//!
//! Edges are directed, with an atom at both endpoints. Edges have a string key associated with them. At most one edge between two atoms with a given key may exist.
//!
//! ### Tag
//!
//! Tags are attached to atoms. They have a kind and a value, both of which are strings.
//!
//! ### Blob
//!
//! Blobs are attached to atoms. They have a type, which is a MIME type, and contents, which is an arbitrarily large binary string.
//!
//! ### Misc.
//!
//! strings are UTF-8 strings, which should be no longer than 256 characters.
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
//!     conn.create_blob(bar, "text/plain", b"bar").await?;
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
    trivial_casts,
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

mod utils;

use bytes::{Bytes, BytesMut};
use derive_more::{Display, FromStr};
use futures_util::try_stream::TryStreamExt;
use hyper::{client::HttpConnector, Client, Request, StatusCode};
pub use mime::Mime;
use serde::{de::DeserializeOwned, Serialize};
use serde_derive::{Deserialize, Serialize};
use std::error::Error;
use tokio::prelude::*;
use url::Url;
use uuid::Uuid;

/// Atoms are the nodes of the graph. Each is represented as a UUID.
#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Eq,
    Display,
    FromStr,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
)]
#[serde(transparent)]
pub struct Atom(#[serde(with = "utils::string")] Uuid);

/// A connection to a G1 database.
#[derive(Debug)]
pub struct Connection {
    base_url: Url,
    client: Client<HttpConnector>,
}

impl Connection {
    /// Opens a connection to the database at the given URL.
    pub fn open(base_url: &str) -> Result<Connection, OpenError> {
        let base_url = Url::parse(base_url)?;
        let conn = Connection::open_url(base_url);
        Ok(conn)
    }

    /// Opens a connection to the database at the given URL.
    pub fn open_url(base_url: Url) -> Connection {
        Connection {
            base_url,
            client: Client::new(),
        }
    }

    /// Makes a query.
    async fn query<T: Serialize, U: DeserializeOwned>(
        &self,
        relative_url: &str,
        body: &T,
    ) -> Result<U, QueryError> {
        let url = self.base_url.join(relative_url).unwrap_or_else(|e| {
            panic!("Invalid relative url in query ({:?}): {}", relative_url, e)
        });
        let body = serde_json::to_string(body).expect("Failed to serialize body");
        let req = Request::post(url.as_ref())
            .body(body.into())
            .expect("Failed to build request");
        let res = self.client.request(req).await.map_err(QueryError::Hyper)?;
        if res.status() != StatusCode::OK {
            return Err(QueryError::BadStatus(res.status()));
        }

        // In theory this invocation should prevent chunks from being copied until they end up in
        // the final BytesMut.
        let body = res
            .into_body()
            .map(|r| r.map(Bytes::from).map(BytesMut::from))
            .try_concat()
            .await
            .map_err(QueryError::Hyper)?;

        let out = serde_json::from_slice(&body).map_err(QueryError::BadResponse)?;
        Ok(out)
    }
}

// The simple/CRUD API.
impl Connection {
    /// Creates a new `Atom`.
    pub async fn create_atom(&self) -> Result<Atom, QueryError> {
        unimplemented!()
    }

    /// Creates a new name for an `Atom`.
    pub async fn create_name(&self, _atom: Atom, _ns: &str, _name: &str) -> Result<(), QueryError> {
        unimplemented!()
    }

    /// Deletes a name, returning whether it existed.
    pub async fn delete_name(
        &self,
        _atom: Atom,
        _ns: &str,
        _name: &str,
    ) -> Result<bool, QueryError> {
        unimplemented!()
    }

    /// Finds the `Atom` corresponding to the given name, if any.
    pub async fn find_atom_by_name(
        &self,
        _ns: &str,
        _name: &str,
    ) -> Result<Option<Atom>, QueryError> {
        let () = self.query("./v0/find-atom", &()).await?;
        unimplemented!()
    }

    /// Creates an edge between two `Atom`s.
    pub async fn create_edge(&self, _from: Atom, _to: Atom, _key: &str) -> Result<(), QueryError> {
        unimplemented!()
    }

    /// Deletes an edge, returning whether it existed.
    pub async fn delete_edge(
        &self,
        _from: Atom,
        _to: Atom,
        _key: &str,
    ) -> Result<bool, QueryError> {
        unimplemented!()
    }

    /// Returns the edges that meet the given criteria as `(from, to, key)` tuples.
    ///
    /// `None` means "don't care," the query is otherwise a conjunction (an `AND`).
    pub async fn find_edges(
        &self,
        _from: Option<Atom>,
        _to: Option<Atom>,
        _key: Option<&str>,
    ) -> Result<Vec<(Atom, Atom, String)>, QueryError> {
        unimplemented!()
    }

    /// Adds a tag to an `Atom` with the given kind and value.
    pub async fn create_tag(
        &self,
        _atom: Atom,
        _kind: &str,
        _value: &str,
    ) -> Result<(), QueryError> {
        unimplemented!()
    }

    /// Find the tag with the given kind on the `Atom`.
    pub async fn find_tag(&self, _atom: Atom, _kind: &str) -> Result<Option<String>, QueryError> {
        unimplemented!()
    }

    /// Deletes the tag with the given kind on the `Atom`, returning whether it was found.
    pub async fn delete_tag(&self, _atom: Atom, _kind: &str) -> Result<bool, QueryError> {
        unimplemented!()
    }

    /// Adds a blob to an `Atom` with the given MIME type and value.
    pub async fn create_blob(
        &self,
        _atom: Atom,
        _mime: Mime,
        _contents: &[u8],
    ) -> Result<(), QueryError> {
        unimplemented!()
    }

    /// Find the blob with the given MIME type on the `Atom`.
    pub async fn find_blob(&self, _atom: Atom, _mime: Mime) -> Result<Option<String>, QueryError> {
        unimplemented!()
    }

    /// Deletes the blob with the given MIME type on the `Atom`, returning whether it was found.
    pub async fn delete_blob(&self, _atom: Atom, _mime: Mime) -> Result<bool, QueryError> {
        unimplemented!()
    }
}

/// An error opening a connection to the database.
#[derive(Debug, Display)]
pub enum OpenError {
    /// An error parsing the URL given to the `Connection::open` function.
    UrlParseError(url::ParseError),
}

impl Error for OpenError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            OpenError::UrlParseError(err) => Some(err),
        }
    }
}

impl From<url::ParseError> for OpenError {
    fn from(err: url::ParseError) -> OpenError {
        OpenError::UrlParseError(err)
    }
}

/// An error making a query to the database.
#[derive(Debug, Display)]
pub enum QueryError {
    /// An unexpected status code was received.
    BadStatus(StatusCode),

    /// A response couldn't be deserialized.
    BadResponse(serde_json::Error),

    /// An error making a request to the server.
    Hyper(hyper::Error),
}

impl Error for QueryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            QueryError::BadStatus(_) => None,
            QueryError::BadResponse(err) => Some(err),
            QueryError::Hyper(err) => Some(err),
        }
    }
}

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
//! **Blob**: Blobs are attached to atoms. They have a kind, which is a string; a type, which is a MIME type; and contents, which are an arbitrarily large binary string. Blobs are referred to by the SHA256 hash of their contents.
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

/*
mod utils;

use bytes::{Bytes, BytesMut};
use derive_more::{Display, FromStr};
use futures_util::try_stream::TryStreamExt;
use hyper::{client::HttpConnector, Client, Request, StatusCode};
pub use mime::Mime;
use serde::{de::DeserializeOwned, Serialize};
use serde_derive::{Deserialize, Serialize};
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
    str::FromStr,
};
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

/// Hashes are identifiers for blobs. Each is the SHA256 hash of the blob.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct Hash([u8; 32]);

impl Display for Hash {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        for i in 0..32 {
            write!(fmt, "{:02x}", self.0[i])?;
        }
        Ok(())
    }
}

impl FromStr for Hash {
    type Err = HashParseError;

    fn from_str(s: &str) -> Result<Hash, HashParseError> {
        fn hex(ch: char, i: usize) -> Result<u8, HashParseError> {
            Ok(match ch {
                '0' => 0,
                '1' => 1,
                '2' => 2,
                '3' => 3,
                '4' => 4,
                '5' => 5,
                '6' => 6,
                '7' => 7,
                '8' => 8,
                '9' => 9,
                'a' | 'A' => 10,
                'b' | 'B' => 11,
                'c' | 'C' => 12,
                'd' | 'D' => 13,
                'e' | 'E' => 14,
                'f' | 'F' => 15,
                ch => return Err(HashParseError::BadChar(i, ch)),
            })
        }

        if s.len() != 64 {
            return Err(HashParseError::BadLength(s.len()));
        }

        let mut hash = [0; 32];
        for (i, ch) in s.chars().enumerate() {
            let x = hex(ch, i)?;
            let j = i / 2;
            let off = if (i % 2) == 0 { 4 } else { 0 };
            hash[j] |= x << off;
        }
        Ok(Hash(hash))
    }
}

/// An error parsing a `Hash`.
#[derive(Clone, Copy, Debug)]
pub enum HashParseError {
    /// The character at the given index wasn't a hex character.
    BadChar(usize, char),

    /// The hash had an unexpected length.
    BadLength(usize),
}

impl Display for HashParseError {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        match self {
            HashParseError::BadChar(i, c) => write!(
                fmt,
                "the character {:?} at index {} wasn't a hex character",
                c, i
            ),
            HashParseError::BadLength(l) => {
                write!(fmt, "the string should be of length 64, not {}", l)
            }
        }
    }
}

impl Error for HashParseError {}

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

    /// Makes a query, returning the body without deserializing (or finishing reading) it.
    async fn query_body<T: Serialize>(
        &self,
        relative_url: &str,
        req_body: &T,
    ) -> Result<impl Stream<Item = Result<Bytes, QueryError>>, QueryError> {
        let url = self.base_url.join(relative_url).unwrap_or_else(|e| {
            panic!("Invalid relative url in query ({:?}): {}", relative_url, e)
        });
        let req_body = serde_json::to_string(req_body).expect("Failed to serialize body");
        let req = Request::post(url.as_ref())
            .body(req_body.into())
            .expect("Failed to build request");
        let res = self.client.request(req).await.map_err(QueryError::Hyper)?;
        if res.status() != StatusCode::OK {
            return Err(QueryError::BadStatus(res.status()));
        }

        Ok(res
            .into_body()
            .map(|r| r.map(Bytes::from).map_err(QueryError::Hyper)))
    }

    /// Makes a query.
    async fn query<T: Serialize, U: DeserializeOwned>(
        &self,
        relative_url: &str,
        req_body: &T,
    ) -> Result<U, QueryError> {
        // In theory this invocation should prevent chunks from being copied until they end up in
        // the final BytesMut.
        let body = self
            .query_body(relative_url, req_body)
            .await?
            .map(|r| r.map(BytesMut::from))
            .try_concat()
            .await?;

        let out = serde_json::from_slice(&body).map_err(QueryError::BadResponse)?;
        Ok(out)
    }
}

// The simple/CRUD API.
impl Connection {
    /// Creates a new `Atom`.
    pub async fn create_atom(&self) -> Result<Atom, QueryError> {
        #[derive(Serialize)]
        struct Body;
        self.query("./v0/create-atom", &Body).await
    }

    /// Creates a new name for an `Atom`.
    pub async fn create_name(&self, atom: Atom, ns: &str, title: &str) -> Result<(), QueryError> {
        #[derive(Serialize)]
        struct Body<'ns, 'title> {
            atom: Atom,
            ns: &'ns str,
            title: &'title str,
        }

        self.query("./v0/create-name", &Body { atom, ns, title })
            .await
    }

    /// Deletes a name, returning whether it existed.
    pub async fn delete_name(&self, ns: &str, title: &str) -> Result<bool, QueryError> {
        #[derive(Serialize)]
        struct Body<'ns, 'title> {
            ns: &'ns str,
            title: &'title str,
        }

        self.query("./v0/delete-name", &Body { ns, title }).await
    }

    /// Finds the `Atom` corresponding to the given name, if any.
    pub async fn find_atom_by_name(
        &self,
        ns: &str,
        title: &str,
    ) -> Result<Option<Atom>, QueryError> {
        #[derive(Serialize)]
        struct Body<'ns, 'title> {
            ns: &'ns str,
            title: &'title str,
        }

        self.query("./v0/find-atom-by-name", &Body { ns, title })
            .await
    }

    /// Creates an edge between two `Atom`s.
    pub async fn create_edge(&self, from: Atom, to: Atom, label: &str) -> Result<(), QueryError> {
        #[derive(Serialize)]
        struct Body<'label> {
            from: Atom,
            to: Atom,
            label: &'label str,
        }

        self.query("./v0/create-edge", &Body { from, to, label })
            .await
    }

    /// Deletes an edge, returning whether it existed.
    pub async fn delete_edge(&self, from: Atom, to: Atom, label: &str) -> Result<bool, QueryError> {
        #[derive(Serialize)]
        struct Body<'label> {
            from: Atom,
            to: Atom,
            label: &'label str,
        }

        self.query("./v0/delete-edge", &Body { from, to, label })
            .await
    }

    /// Returns the edges that meet the given criteria as `(from, to, label)` tuples.
    ///
    /// `None` means "don't care," the query is otherwise a conjunction (an `AND`).
    pub async fn find_edges(
        &self,
        from: Option<Atom>,
        to: Option<Atom>,
        label: Option<&str>,
    ) -> Result<Vec<(Atom, Atom, String)>, QueryError> {
        #[derive(Serialize)]
        struct Body<'label> {
            from: Option<Atom>,
            to: Option<Atom>,
            label: Option<&'label str>,
        }

        self.query("./v0/find-edges", &Body { from, to, label }).await
    }

    /// Adds a tag to an `Atom` with the given kind and value.
    pub async fn create_tag(&self, atom: Atom, kind: &str, value: &str) -> Result<(), QueryError> {
        #[derive(Serialize)]
        struct Body<'kind, 'value> {
            atom: Atom,
            kind: &'kind str,
            value: &'value str,
        }

        self.query("./v0/create-tag", &Body { atom, kind, value })
            .await
    }

    /// Find the tag with the given kind on the `Atom`.
    pub async fn find_tag(&self, atom: Atom, kind: &str) -> Result<Option<String>, QueryError> {
        #[derive(Serialize)]
        struct Body<'kind> {
            atom: Atom,
            kind: &'kind str,
        }

        self.query("./v0/find-tag", &Body { atom, kind }).await
    }

    /// Deletes the tag with the given kind on the `Atom`, returning whether it was found.
    pub async fn delete_tag(&self, atom: Atom, kind: &str) -> Result<bool, QueryError> {
        #[derive(Serialize)]
        struct Body<'kind> {
            atom: Atom,
            kind: &'kind str,
        }

        self.query("./v0/delete-tag", &Body { atom, kind }).await
    }

    /// Adds a blob to an `Atom` with the given disposition, MIME type and value.
    pub async fn create_blob(
        &self,
        atom: Atom,
        disposition: &str,
        mime: Mime,
        contents: &[u8],
    ) -> Result<(), QueryError> {
        #[derive(Serialize)]
        struct Body<'disposition, 'contents> {
            atom: Atom,
            disposition: &'disposition str,
            #[serde(with = "utils::string")]
            mime: Mime,
            contents: &'contents [u8],
        }

        self.query(
            "./v0/create-blob",
            &Body {
                atom,
                disposition,
                mime,
                contents,
            },
        )
        .await
    }

    /// Deletes the blob with the given disposition and MIME type on the `Atom`, returning whether
    /// it was found.
    pub async fn delete_blob(
        &self,
        atom: Atom,
        disposition: &str,
        mime: Mime,
    ) -> Result<bool, QueryError> {
        #[derive(Serialize)]
        struct Body<'disposition> {
            atom: Atom,
            disposition: &'disposition str,
            #[serde(with = "utils::string")]
            mime: Mime,
        }

        self.query(
            "./v0/delete-blob",
            &Body {
                atom,
                disposition,
                mime,
            },
        )
        .await
    }

    /// Streams a blob as a set of `Bytes`.
    pub async fn stream_blob(
        &self,
        hash: Hash,
    ) -> Result<impl Stream<Item = Result<Bytes, QueryError>>, QueryError> {
        #[derive(Serialize)]
        struct Body {
            hash: Hash,
        }

        self.query_body("./v0/get-blob", &Body { hash }).await
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
*/

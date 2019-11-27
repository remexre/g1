//! A simple graph store.
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
use serde::{de::DeserializeOwned, Serialize};
use serde_derive::{Deserialize, Serialize};
use std::error::Error;
use tokio::prelude::*;
use url::Url;
use uuid::Uuid;

/// Atoms are the nodes of the graph. Each is represented as a UUID.
#[derive(Debug, Deserialize, Display, FromStr, Serialize)]
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

    /// Finds the `Atom` corresponding to the given name, if any.
    pub async fn find_atom(&self, _name: &str) -> Result<Option<Atom>, QueryError> {
        let () = self.query("./v0/find-atom", &()).await?;
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

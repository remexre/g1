//! Common types and traits between the client and server portion of the G1 graph store.
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

#[cfg(feature = "parser")]
use lalrpop_util::lalrpop_mod;

#[cfg(feature = "parser")]
mod lexer;
pub mod nameless;
#[cfg(feature = "parser")]
lalrpop_mod!(parser);
pub mod query;
#[cfg(test)]
mod strategies;
#[cfg(test)]
mod tests;
mod utils;

use crate::{nameless::NamelessQuery, query::Value};
use async_trait::async_trait;
use bytes::Bytes;
use derive_more::{Display, FromStr};
use futures::prelude::*;
pub use mime::Mime;
use serde_derive::{Deserialize, Serialize};
use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    str::FromStr,
};
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

/// Hashes are identifiers for blobs. Each is a SHA256 hash of the blob, possibly with some
/// additional metadata.
///
/// (In other words, it's not sound to hash the file on the client-side.)
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

impl std::error::Error for HashParseError {}

/// The basic interface to a G1 server. This exposes all the operations which must be atomic
/// without transactions.
#[async_trait]
pub trait Connection: Send + Sync {
    /// The error returned by operations on this connection.
    type Error: Error;

    /// Creates a new atom in the database, returning it.
    async fn create_atom(&self) -> Result<Atom, Self::Error>;

    /// Deletes any names referring to an atom, all edges going to or from it, any tags attached
    /// to it, and any blobs attached to it.
    ///
    /// Note that the atom itself is not deleted, so `create_atom` will not reuse it. At some
    /// point, an operation to do this may exist, but note that doing so will break useful
    /// properties for most operations.
    ///
    /// Returns whether the atom existed prior to the call.
    async fn delete_atom(&self, atom: Atom) -> Result<bool, Self::Error>;

    /// Creates a new name for an atom.
    ///
    /// If the name already exists, it is an error unless `upsert` is `true`, in which case the
    /// existing name will be deleted. Returns whether a name was deleted due to `upsert`.
    async fn create_name(
        &self,
        atom: Atom,
        ns: &str,
        title: &str,
        upsert: bool,
    ) -> Result<bool, Self::Error>;

    /// Deletes a name.
    ///
    /// Returns whether the name existed prior to the call.
    async fn delete_name(&self, ns: &str, title: &str) -> Result<bool, Self::Error>;

    /// Creates a new edge between two atoms. If either of the atoms does not exist, creates them.
    ///
    /// Returns `true` if an edge already exists with the same endpoints and label.
    async fn create_edge(&self, from: Atom, to: Atom, label: &str) -> Result<bool, Self::Error>;

    /// Deletes the edge with the given endpoints and label.
    ///
    /// Returns whether the edge existed prior to the call.
    async fn delete_edge(&self, from: Atom, to: Atom, label: &str) -> Result<bool, Self::Error>;

    /// Creates a tag attached to an atom with the given key and value.
    ///
    /// If a tag with the given key already exists on the atom, it is an error unless `upsert` is
    /// `true`, in which case the existing value will be replaced by the given one. Returns whether
    /// a value was replaced due to `upsert`.
    async fn create_tag(
        &self,
        atom: Atom,
        key: &str,
        value: &str,
        upsert: bool,
    ) -> Result<bool, Self::Error>;

    /// Deletes the tag with the given key from the given atom.
    ///
    /// Returns whether the tag existed prior to the call.
    async fn delete_tag(&self, atom: Atom, key: &str) -> Result<bool, Self::Error>;

    /// Creates a blob attached to an atom with the given kind, MIME type, and hash.
    ///
    /// If a blob with the given kind and MIME type already exists on the atom, it is an error
    /// unless `upsert` is `true`, in which case the existing hash will be replaced by the given
    /// ones. Returns whether the hash was replaced due to `upsert`.
    async fn create_blob(
        &self,
        atom: Atom,
        kind: &str,
        mime: Mime,
        hash: Hash,
        upsert: bool,
    ) -> Result<bool, Self::Error>;

    /// Deletes the blob with the given kind and MIME type from the given atom.
    ///
    /// Returns whether the blob existed prior to the call.
    async fn delete_blob(&self, atom: Atom, kind: &str, mime: Mime) -> Result<bool, Self::Error>;

    /// Fetches a blob from the server by its hash.
    async fn fetch_blob(
        &self,
        hash: Hash,
    ) -> Result<Box<dyn Stream<Item = Result<Bytes, Self::Error>> + Send>, Self::Error>;

    /// Stores a blob on the server, returning its hash.
    async fn store_blob(
        &self,
        data: impl Stream<Item = Result<Bytes, Self::Error>>,
    ) -> Result<Hash, Self::Error>;

    /// Performs a query, returning multiple results (at most `limit`).
    async fn query(
        &self,
        limit: usize,
        query: NamelessQuery,
    ) -> Result<Vec<Vec<Value>>, Self::Error>;

    /// Performs a query, returning at most one result.
    async fn query_first(&self, query: NamelessQuery) -> Result<Option<Vec<Value>>, Self::Error> {
        let mut v = self.query(1, query).await?;
        debug_assert!(v.len() < 2);
        Ok(v.pop())
    }

    /// Performs a query, returning whether it had results.
    ///
    /// Note that the default implementation can be inefficient.
    async fn query_has_results(&self, query: NamelessQuery) -> Result<bool, Self::Error> {
        Ok(self.query_first(query).await?.is_some())
    }
}

/// The error returned by operations on a G1 server.
pub trait Error: std::error::Error {
    /// Creates an error representing an invalid query.
    fn invalid_query(msg: &'static str) -> Self;

    /// Returns whether the error was created by `invalid_query`.
    fn is_invalid_query(&self) -> bool;
}

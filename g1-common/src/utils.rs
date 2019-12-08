//! Utilities. These are unstable, don't depend on these.

use bytes::Bytes;
use futures::prelude::*;
use std::{collections::HashSet, path::Path, pin::Pin, sync::Arc};
use tokio::{fs::File, io::AsyncRead};

/// A pool for deduplicating strings.
#[derive(Debug, Default)]
pub struct StringPool(HashSet<Arc<str>>);

impl StringPool {
    /// Stores a string in the pool, returning an `Arc<str>`.
    pub fn store(&mut self, s: String) -> Arc<str> {
        let s = Arc::from(s);
        match self.0.get(&s) {
            Some(s) => s.clone(),
            None => {
                let _ = self.0.insert(s.clone());
                s
            }
        }
    }
}

/// Reads a file as a stream of chunks.
pub async fn file_to_stream<P: AsRef<Path>>(
    path: P,
) -> Result<impl Stream<Item = Result<Bytes, tokio::io::Error>>, tokio::io::Error> {
    let mut file = File::open(path).await?;
    Ok(stream::poll_fn(move |cx| {
        let mut buf = [0; 4096];
        Pin::new(&mut file)
            .poll_read(cx, &mut buf)
            .map(move |r| match r {
                Ok(0) => None,
                Ok(n) => Some(Ok(Bytes::copy_from_slice(&buf[..n]))),
                Err(e) => Some(Err(e.into())),
            })
    }))
}

pub mod string {
    //! Serde support via `Display`/`FromStr`.
    //!
    //! ## Example
    //!
    //! ```
    //! use std::net::Ipv4Addr;
    //! let s = Ipv4Addr::from([127, 0, 0, 1]);
    //! ```

    use serde::{de::Error, Deserialize, Deserializer, Serializer};
    use std::{fmt::Display, str::FromStr};

    /// Serializes a value as a string with its `Display` impl.
    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Display,
        S: Serializer,
    {
        serializer.collect_str(value)
    }

    /// Deserializes a value as a string with its `FromStr` impl
    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: FromStr,
        T::Err: Display,
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(Error::custom)
    }
}

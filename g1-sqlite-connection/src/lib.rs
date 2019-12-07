//! A G1 connection based on an SQLite database, using the FS for blobs.
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

mod cmd;
mod run;

use crate::cmd::Command;
use bytes::BytesMut;
use futures::{executor::block_on, prelude::*};
use g1_common::{nameless::NamelessQuery, Atom, Bytes, Connection, Hash, Mime};
use sha2::{Digest, Sha256};
use std::{
    os::unix::ffi::OsStrExt,
    path::PathBuf,
    pin::Pin,
    sync::Arc,
    thread::{spawn, JoinHandle},
};
use thiserror::Error;
use tokio::{
    fs::{create_dir_all, rename, File},
    io::AsyncRead,
    prelude::*,
    sync::{
        mpsc::{channel, Sender},
        oneshot, Mutex,
    },
    task::spawn_blocking,
};
use uuid::Uuid;

/// A G1 connection based on an SQLite database, using the FS for blobs.
///
/// TODO: Make this not use `g1_common::naive_solve`...
#[derive(Debug)]
pub struct SqliteConnection {
    join: JoinHandle<()>,
    path: PathBuf,
    send: Mutex<Sender<Command>>,
}

const INITDB: &str = r#"
create table if not exists atoms
  ( atom text not null
  , constraint atomUnique unique (atom)
  );
create table if not exists names
  ( atom text not null
  , ns text not null
  , title text not null
  , constraint nameUnique unique (ns, title)
  );
create table if not exists edges
  ( edge_from text not null
  , edge_to text not null
  , label text not null
  , constraint edgeUnique unique (edge_from, edge_to, label)
  );
create table if not exists tags
  ( atom text not null
  , key text not null
  , value text not null
  , constraint tagUnique unique (atom, key)
  );
create table if not exists blobs
  ( atom text not null
  , kind text not null
  , mime text not null
  , hash text not null
  , constraint blobUnique unique (atom, kind, mime)
  );"#;

impl SqliteConnection {
    /// Opens a connection to the database, given a directory to store the database and blobs in.
    pub async fn open(path: PathBuf) -> Result<SqliteConnection, SqliteConnectionError> {
        create_dir_all(path.join("blobs")).await?;
        create_dir_all(path.join("tmp")).await?;

        let mut conn_path = path.clone();
        conn_path.push("g1.db");
        let conn = spawn_blocking(move || -> rusqlite::Result<rusqlite::Connection> {
            let conn = rusqlite::Connection::open(conn_path)?;
            conn.execute_batch(INITDB)?;
            Ok(conn)
        })
        .await
        .map_err(tokio::io::Error::from)??;

        let (send, mut recv) = channel::<Command>(1);
        let join = spawn(move || {
            let mut conn = conn;

            while let Some(cmd) = block_on(recv.recv()) {
                cmd.run(&mut conn);
            }

            for _ in 0..3 {
                match conn.close() {
                    Ok(()) => break,
                    Err((c, err)) => {
                        conn = c;
                        log::error!("Failed to close SQLite: {}", err);
                    }
                }
            }
        });
        Ok(SqliteConnection {
            join,
            path,
            send: Mutex::new(send),
        })
    }

    async fn send_command<F, T>(&self, make_command: F) -> Result<T, SqliteConnectionError>
    where
        F: FnOnce(oneshot::Sender<Result<T, SqliteConnectionError>>) -> Command,
    {
        let (send, recv) = oneshot::channel();
        let mut send_send = self.send.lock().await;
        send_send
            .send(make_command(send))
            .await
            .map_err(|_| SqliteConnectionError::SQLitePanic)?;
        recv.await.map_err(|_| SqliteConnectionError::SQLitePanic)?
    }
}

#[async_trait::async_trait]
impl Connection for SqliteConnection {
    type Error = SqliteConnectionError;

    async fn create_atom(&self) -> Result<Atom, Self::Error> {
        self.send_command(|send| Command::CreateAtom(send)).await
    }

    async fn delete_atom(&self, atom: Atom) -> Result<bool, Self::Error> {
        self.send_command(move |send| Command::DeleteAtom(atom, send))
            .await
    }

    async fn create_name(
        &self,
        atom: Atom,
        ns: &str,
        title: &str,
        upsert: bool,
    ) -> Result<bool, Self::Error> {
        self.send_command(move |send| {
            Command::CreateName(atom, ns.to_string(), title.to_string(), upsert, send)
        })
        .await
    }

    async fn delete_name(&self, ns: &str, title: &str) -> Result<bool, Self::Error> {
        self.send_command(move |send| Command::DeleteName(ns.to_string(), title.to_string(), send))
            .await
    }

    async fn create_edge(&self, from: Atom, to: Atom, label: &str) -> Result<bool, Self::Error> {
        self.send_command(move |send| Command::CreateEdge(from, to, label.to_string(), send))
            .await
    }

    async fn delete_edge(&self, from: Atom, to: Atom, label: &str) -> Result<bool, Self::Error> {
        self.send_command(move |send| Command::DeleteEdge(from, to, label.to_string(), send))
            .await
    }

    async fn create_tag(
        &self,
        atom: Atom,
        key: &str,
        value: &str,
        upsert: bool,
    ) -> Result<bool, Self::Error> {
        self.send_command(move |send| {
            Command::CreateTag(atom, key.to_string(), value.to_string(), upsert, send)
        })
        .await
    }

    async fn delete_tag(&self, atom: Atom, key: &str) -> Result<bool, Self::Error> {
        self.send_command(move |send| Command::DeleteTag(atom, key.to_string(), send))
            .await
    }

    async fn create_blob(
        &self,
        atom: Atom,
        kind: &str,
        mime: Mime,
        hash: Hash,
        upsert: bool,
    ) -> Result<bool, Self::Error> {
        self.send_command(move |send| {
            Command::CreateBlob(atom, kind.to_string(), mime, hash, upsert, send)
        })
        .await
    }

    async fn delete_blob(&self, atom: Atom, kind: &str, mime: Mime) -> Result<bool, Self::Error> {
        self.send_command(move |send| Command::DeleteBlob(atom, kind.to_string(), mime, send))
            .await
    }

    async fn fetch_blob(
        &self,
        hash: Hash,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<Bytes, Self::Error>> + Send>>, Self::Error> {
        let mut path = self.path.clone();
        path.push("blobs");
        path.push(hash.to_string());

        let mut file = File::open(path).await?;
        Ok(stream::poll_fn(move |cx| {
            let mut buf = BytesMut::new();
            Pin::new(&mut file)
                .poll_read(cx, &mut buf)
                .map(|r| match r {
                    Ok(0) => None,
                    Ok(_) => Some(Ok(buf.freeze())),
                    Err(e) => Some(Err(e.into())),
                })
        })
        .boxed())
    }

    async fn store_blob(
        &self,
        mut data: Pin<Box<dyn Stream<Item = Result<Bytes, Self::Error>> + Send + 'static>>,
    ) -> Result<Hash, Self::Error> {
        let mut tmp_path = self.path.clone();
        tmp_path.push("tmp");
        tmp_path.push(Uuid::new_v4().to_string()); // Using a UUID as an easy random string.

        let mut file = File::create(&tmp_path).await?;
        let mut hasher = Sha256::new();
        while let Some(r) = data.next().await {
            let chunk = r?;
            hasher.input(&chunk);
            let _ = file.write(&chunk).await?;
        }
        file.sync_all().await?;
        let hash = Hash::from_bytes(hasher.result().as_slice());

        let mut path = self.path.clone();
        path.push("blobs");
        path.push(hash.to_string());

        rename(tmp_path, &path).await?;
        let _ = path.pop();
        // TODO: Replace with https://github.com/tokio-rs/tokio/issues/1922
        spawn_blocking(move || {
            let path = path.as_os_str().as_bytes().as_ptr() as *const libc::c_char;

            #[allow(unsafe_code)]
            unsafe {
                let errno = libc::__errno_location();

                let fd = libc::open(path, libc::O_DIRECTORY | libc::O_RDONLY);
                if fd == -1 {
                    return Err(*errno);
                }

                if libc::fsync(fd) != 0 {
                    let _ = libc::close(fd);
                    return Err(*errno);
                }

                if libc::close(fd) != 0 {
                    return Err(*errno);
                }
            }
            Ok(())
        })
        .await
        .map_err(tokio::io::Error::from)?
        .map_err(std::io::Error::from_raw_os_error)?;

        Ok(hash)
    }

    async fn query(
        &self,
        limit: Option<usize>,
        query: &NamelessQuery,
    ) -> Result<Vec<Vec<Arc<str>>>, Self::Error> {
        self.send_command(move |send| Command::Query(limit, query.clone(), send))
            .await
    }
}

/// An error performing an operation on an `SqliteConnection`.
#[derive(Debug, Error)]
pub enum SqliteConnectionError {
    /// An I/O error occurred.
    #[error("IO error: {0}")]
    IO(#[from] tokio::io::Error),

    /// A query was invalid.
    ///
    /// This could be the result of a syntax error, or a non-stratifiable query.
    #[error("Invalid query: {0}")]
    InvalidQuery(String),

    /// An error from SQLite.
    #[error("SQLite error: {0}")]
    SQLite(#[from] rusqlite::Error),

    /// The SQLite thread panicked.
    #[error("The SQLite thread panicked")]
    SQLitePanic,
}

impl g1_common::Error for SqliteConnectionError {
    fn invalid_query(msg: String) -> SqliteConnectionError {
        SqliteConnectionError::InvalidQuery(msg)
    }
}

use crate::SqliteConnectionError;
use g1_common::{nameless::NamelessQuery, Atom, Hash, Mime};
use std::sync::Arc;
use tokio::sync::oneshot::Sender;

#[derive(derivative::Derivative)]
#[derivative(Debug)]
pub enum Command {
    CreateAtom(#[derivative(Debug = "ignore")] Sender<Result<Atom, SqliteConnectionError>>),
    DeleteAtom(
        Atom,
        #[derivative(Debug = "ignore")] Sender<Result<bool, SqliteConnectionError>>,
    ),
    CreateName(
        Atom,
        String,
        String,
        bool,
        #[derivative(Debug = "ignore")] Sender<Result<bool, SqliteConnectionError>>,
    ),
    DeleteName(
        String,
        String,
        #[derivative(Debug = "ignore")] Sender<Result<bool, SqliteConnectionError>>,
    ),
    CreateEdge(
        Atom,
        Atom,
        String,
        #[derivative(Debug = "ignore")] Sender<Result<bool, SqliteConnectionError>>,
    ),
    DeleteEdge(
        Atom,
        Atom,
        String,
        #[derivative(Debug = "ignore")] Sender<Result<bool, SqliteConnectionError>>,
    ),
    CreateTag(
        Atom,
        String,
        String,
        bool,
        #[derivative(Debug = "ignore")] Sender<Result<bool, SqliteConnectionError>>,
    ),
    DeleteTag(
        Atom,
        String,
        #[derivative(Debug = "ignore")] Sender<Result<bool, SqliteConnectionError>>,
    ),
    CreateBlob(
        Atom,
        String,
        Mime,
        Hash,
        bool,
        #[derivative(Debug = "ignore")] Sender<Result<bool, SqliteConnectionError>>,
    ),
    DeleteBlob(
        Atom,
        String,
        Mime,
        #[derivative(Debug = "ignore")] Sender<Result<bool, SqliteConnectionError>>,
    ),
    Query(
        Option<usize>,
        NamelessQuery,
        #[derivative(Debug = "ignore")] Sender<Result<Vec<Vec<Arc<str>>>, SqliteConnectionError>>,
    ),
}

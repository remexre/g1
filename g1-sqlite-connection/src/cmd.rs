use crate::SqliteError;
use g1_common::{nameless::NamelessQuery, Atom, Hash, Mime};
use std::sync::Arc;
use tokio::sync::oneshot::Sender;

#[derive(derivative::Derivative)]
#[derivative(Debug)]
pub enum Command {
    CreateAtom(#[derivative(Debug = "ignore")] Sender<Result<Atom, SqliteError>>),
    DeleteAtom(
        Atom,
        #[derivative(Debug = "ignore")] Sender<Result<bool, SqliteError>>,
    ),
    CreateName(
        Atom,
        String,
        String,
        bool,
        #[derivative(Debug = "ignore")] Sender<Result<bool, SqliteError>>,
    ),
    DeleteName(
        String,
        String,
        #[derivative(Debug = "ignore")] Sender<Result<bool, SqliteError>>,
    ),
    CreateEdge(
        Atom,
        Atom,
        String,
        #[derivative(Debug = "ignore")] Sender<Result<bool, SqliteError>>,
    ),
    DeleteEdge(
        Atom,
        Atom,
        String,
        #[derivative(Debug = "ignore")] Sender<Result<bool, SqliteError>>,
    ),
    CreateTag(
        Atom,
        String,
        String,
        bool,
        #[derivative(Debug = "ignore")] Sender<Result<bool, SqliteError>>,
    ),
    DeleteTag(
        Atom,
        String,
        #[derivative(Debug = "ignore")] Sender<Result<bool, SqliteError>>,
    ),
    CreateBlob(
        Atom,
        String,
        Mime,
        Hash,
        bool,
        #[derivative(Debug = "ignore")] Sender<Result<bool, SqliteError>>,
    ),
    DeleteBlob(
        Atom,
        String,
        Mime,
        #[derivative(Debug = "ignore")] Sender<Result<bool, SqliteError>>,
    ),
    Query(
        Option<usize>,
        NamelessQuery,
        #[derivative(Debug = "ignore")] Sender<Result<Vec<Vec<Arc<str>>>, SqliteError>>,
    ),
}

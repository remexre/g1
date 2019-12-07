use crate::G1SqliteError;
use g1_common::{nameless::NamelessQuery, Atom, Hash, Mime};
use std::sync::Arc;
use tokio::sync::oneshot::Sender;

#[derive(derivative::Derivative)]
#[derivative(Debug)]
pub enum Command {
    CreateAtom(#[derivative(Debug = "ignore")] Sender<Result<Atom, G1SqliteError>>),
    DeleteAtom(
        Atom,
        #[derivative(Debug = "ignore")] Sender<Result<bool, G1SqliteError>>,
    ),
    CreateName(
        Atom,
        String,
        String,
        bool,
        #[derivative(Debug = "ignore")] Sender<Result<bool, G1SqliteError>>,
    ),
    DeleteName(
        String,
        String,
        #[derivative(Debug = "ignore")] Sender<Result<bool, G1SqliteError>>,
    ),
    CreateEdge(
        Atom,
        Atom,
        String,
        #[derivative(Debug = "ignore")] Sender<Result<bool, G1SqliteError>>,
    ),
    DeleteEdge(
        Atom,
        Atom,
        String,
        #[derivative(Debug = "ignore")] Sender<Result<bool, G1SqliteError>>,
    ),
    CreateTag(
        Atom,
        String,
        String,
        bool,
        #[derivative(Debug = "ignore")] Sender<Result<bool, G1SqliteError>>,
    ),
    DeleteTag(
        Atom,
        String,
        #[derivative(Debug = "ignore")] Sender<Result<bool, G1SqliteError>>,
    ),
    CreateBlob(
        Atom,
        String,
        Mime,
        Hash,
        bool,
        #[derivative(Debug = "ignore")] Sender<Result<bool, G1SqliteError>>,
    ),
    DeleteBlob(
        Atom,
        String,
        Mime,
        #[derivative(Debug = "ignore")] Sender<Result<bool, G1SqliteError>>,
    ),
    Query(
        Option<usize>,
        NamelessQuery,
        #[derivative(Debug = "ignore")] Sender<Result<Vec<Vec<Arc<str>>>, G1SqliteError>>,
    ),
}

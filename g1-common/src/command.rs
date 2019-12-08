//! A command entered at the REPL.
//!
//! This lives in this crate largely so the same parser can be used as for queries.

use crate::{
    lexer::Lexer,
    parser::CommandParser,
    query::{Clause, Predicate},
};
use lalrpop_util::ParseError;
use std::str::FromStr;

/// A command entered at the REPL.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Command {
    /// Adds a clause to a predicate, possibly defining it.
    Clause(Clause),

    /// Creates a new atom in the database, printing it.
    CreateAtom,

    /// Deletes any names referring to an atom, all edges going to or from it, any tags attached
    /// to it, and any blobs attached to it.
    ///
    /// Note that the atom itself is not deleted, so `create_atom` will not reuse it. At some
    /// point, an operation to do this may exist, but note that doing so will break useful
    /// properties for most operations.
    DeleteAtom(String),

    /// Creates a new name for an atom.
    ///
    /// If the name already exists, it is an error unless `upsert` is `true`, in which case the
    /// existing name will be deleted. Prints whether a name was deleted due to `upsert`.
    CreateName(String, String, String, bool),

    /// Deletes a name.
    ///
    /// Prints whether the name existed prior to the call.
    DeleteName(String, String),

    /// Creates a new edge between two atoms.
    ///
    /// Prints whether an edge already exists with the same endpoints and label.
    CreateEdge(String, String, String),

    /// Deletes the edge with the given endpoints and label.
    ///
    /// Prints whether the edge existed prior to the call.
    DeleteEdge(String, String, String),

    /// Creates a tag attached to an atom with the given key and value.
    ///
    /// If a tag with the given key already exists on the atom, it is an error unless `upsert` is
    /// `true`, in which case the existing value will be replaced by the given one. Prints whether
    /// a value was replaced due to `upsert`.
    CreateTag(String, String, String, bool),

    /// Deletes the tag with the given key from the given atom.
    ///
    /// Prints whether the tag existed prior to the call.
    DeleteTag(String, String),

    /// Creates a blob attached to an atom with the given kind, MIME type, and hash.
    ///
    /// If a blob with the given kind and MIME type already exists on the atom, it is an error
    /// unless `upsert` is `true`, in which case the existing hash will be replaced by the given
    /// ones. Prints whether the hash was replaced due to `upsert`.
    CreateBlob(String, String, String, String, bool),

    /// Deletes the blob with the given kind and MIME type from the given atom.
    ///
    /// Prints whether the blob existed prior to the call.
    DeleteBlob(String, String, String),

    /// Asks for help.
    Help,

    /// Lists the existing predicates.
    List,

    /// Performs a query.
    Query(Predicate),

    /// Quits the REPL.
    Quit,

    /// Undefines defined predicates with the given functor.
    Undefine(String, u32),
}

impl FromStr for Command {
    type Err = ParseError<String, String, String>;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        CommandParser::new().parse(Lexer::new(src)).map_err(|err| {
            err.map_location(|()| "TODO".to_string())
                .map_token(|(_, l)| l.to_string())
        })
    }
}

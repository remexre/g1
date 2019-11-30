use crate::{
    query::{Clause, Predicate, Query},
    Error,
};

impl Predicate {
    /// Validates a predicate as the head of a clause.
    fn validate_head<E: Error>(&self) -> Result<(), E> {
        let name: &str = &self.name;
        match (name, self.args.len()) {
            ("name", 3) => Err(Error::invalid_query(
                "adding a clause to the name/3 predicate is illegal",
            )),
            ("edge", 3) => Err(Error::invalid_query(
                "adding a clause to the edge/3 predicate is illegal",
            )),
            ("tag", 3) => Err(Error::invalid_query(
                "adding a clause to the tag/3 predicate is illegal",
            )),
            ("blob", 4) => Err(Error::invalid_query(
                "adding a clause to the blob/4 predicate is illegal",
            )),
            _ => Ok(()),
        }
    }
}

impl Clause {
    /// Validates a clause.
    pub fn validate<E: Error>(&self) -> Result<(), E> {
        self.head.validate_head()?;

        // TODO: Rule 3.

        // TODO: Rule 4.

        unimplemented!()
    }
}

/// A representation of clauses that makes it easier to validate.
///
/// The stratified representation names clauses with indices, collects all clauses together, and
/// explicitly declares variables.
#[derive(Debug)]
pub struct StratifiedClause;

impl Query {
    /// Validates a query.
    pub fn validate<E: Error>(&self) -> Result<(), E> {
        for clause in &self.clauses {
            clause.validate()?;
        }

        // TODO: Rule 2.

        unimplemented!()
    }
}

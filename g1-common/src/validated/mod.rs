//! A type for queries that allows for validation of the queries.

mod map_span;
pub(crate) mod pool;
mod validate;
pub mod visitors;

pub use crate::validated::validate::ValidationError;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    sync::Arc,
};

/// A span.
pub trait Span: Clone + Debug {
    /// Formats a span as the start of a error mage.
    fn fmt_span(&self, fmt: &mut Formatter) -> std::fmt::Result;
}

impl Span for () {
    fn fmt_span(&self, _fmt: &mut Formatter) -> std::fmt::Result {
        Ok(())
    }
}

/// The kind of a value.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ValidatedValueInner {
    /// A string.
    Str(Arc<str>),

    /// A variable.
    Var(u32),
}

impl Display for ValidatedValueInner {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        match self {
            ValidatedValueInner::Str(s) => write!(fmt, "{:?}", s),
            ValidatedValueInner::Var(n) => write!(fmt, "#{}", n),
        }
    }
}

/// A data value.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ValidatedValue<S: Span> {
    /// The data.
    pub inner: ValidatedValueInner,

    /// The source span of the value.
    pub span: S,
}

impl<S: Span> Display for ValidatedValue<S> {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        write!(fmt, "{}", self.inner)
    }
}

/// A call to a rule.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ValidatedPredicate<S: Span> {
    /// The name of the predicate.
    pub name: i32,

    /// The arguments to the predicate.
    pub args: Vec<ValidatedValue<S>>,

    /// The source span of the predicate.
    pub span: S,
}

impl<S: Span> Display for ValidatedPredicate<S> {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        write!(fmt, "{}(", self.name)?;
        let mut first = true;
        for arg in self.args.iter() {
            if first {
                first = false;
            } else {
                write!(fmt, ", ")?;
            }

            write!(fmt, "{}", arg)?;
        }
        write!(fmt, ")")
    }
}

/// A single clause, used for deduction.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ValidatedClause<S: Span> {
    /// The head of the clause.
    pub head: ValidatedPredicate<S>,

    /// The body of the clause.
    ///
    /// The boolean corresponds to whether the predicate is negated; it is negated when the boolean
    /// is `true`.
    pub body: Vec<(bool, ValidatedPredicate<S>)>,

    /// The number of variables in the clause.
    pub vars: u32,

    /// The source span of the clause.
    pub span: S,
}

impl<S: Span> Display for ValidatedClause<S> {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        if self.body.is_empty() {
            writeln!(fmt, "{}.", self.head)
        } else {
            writeln!(fmt, "{} :-", self.head)?;
            let mut first = true;
            for (negated, arg) in self.body.iter() {
                if first {
                    first = false;
                } else {
                    writeln!(fmt, ",")?;
                }

                write!(fmt, "\t")?;
                if *negated {
                    write!(fmt, "! ")?;
                }
                write!(fmt, "{}", arg)?;
            }
            writeln!(fmt, ".")
        }
    }
}

/// A complete query to the database.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ValidatedQuery<S: Span> {
    /// The clauses to be used by the query.
    pub clauses: Vec<ValidatedClause<S>>,

    /// The predicate to solve for.
    pub goal: ValidatedPredicate<S>,

    /// The number of variables in the goal.
    pub goal_vars: u32,

    /// The source span of the query.
    pub span: S,
}

impl<S: Span> Display for ValidatedQuery<S> {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        for clause in self.clauses.iter() {
            write!(fmt, "{}", clause)?;
        }
        writeln!(fmt, "?- {}.", self.goal)
    }
}

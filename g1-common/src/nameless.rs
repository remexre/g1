//! A nameless representation of queries, which are easier to execute, easier to validate, and
//! easier to operate on.
//!
//! This representation names predicates with indices, collects clauses for the same predicate
//! together, sorts predicates to stratify them, and explicitly declare variables used in each
//! clause.

use crate::{
    query::{Clause, Predicate, Query, Value},
    Error,
};
use serde_derive::{Deserialize, Serialize};
use std::{collections::HashMap, convert::TryFrom};

/// A nameless representation of values.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum NamelessValue {
    /// A signed integer.
    Int(i64),

    /// A string.
    String(String),

    /// A variable.
    Var(u32),
}

impl Value {
    fn to_nameless<E: Error>(self, var_env: &mut Vec<String>) -> Result<NamelessValue, E> {
        match self {
            Value::Int(n) => Ok(NamelessValue::Int(n)),
            Value::String(s) => Ok(NamelessValue::String(s)),
            Value::Var(v) => {
                let n = var_env.iter().position(|v2| &v == v2).unwrap_or_else(|| {
                    let n = var_env.len();
                    var_env.push(v);
                    n
                });
                let n = u32::try_from(n)
                    .map_err(|_| Error::invalid_query("too many variables used"))?;
                Ok(NamelessValue::Var(n))
            }
        }
    }
}

/// A nameless representation of predicates.
#[derive(Debug)]
pub struct NamelessPredicate {
    /// The name of the predicate. Note that the names `0`-`4` refer to the builtin predicates
    /// `atom/1`, `name/3`, `edge/3`, `tag/3`, and `blob/4`, respectively.
    pub name: u32,

    /// The arguments to the predicate.
    pub args: Vec<NamelessValue>,
}

impl Predicate {
    fn to_nameless<E: Error>(
        self,
        pred_env: &HashMap<(&str, usize), u32>,
        var_env: &mut Vec<String>,
    ) -> Result<NamelessPredicate, E> {
        let name = pred_env
            .get(&(&self.name, self.args.len()))
            .cloned()
            .ok_or_else(|| Error::invalid_query("undeclared predicate"))?;
        let args = self
            .args
            .into_iter()
            .map(|v| v.to_nameless(var_env))
            .collect::<Result<_, _>>()?;
        Ok(NamelessPredicate { name, args })
    }
}

/// A nameless representation of clauses.
#[derive(Debug)]
pub struct NamelessClause {
    /// The number of variables used in the clause.
    pub vars: u32,

    /// The arguments to the head predicate of the clause.
    pub args: Vec<NamelessValue>,

    /// The body of the clause.
    ///
    /// The boolean corresponds to whether the predicate is negated; it is negated when the boolean
    /// is `true`.
    pub body: Vec<(bool, NamelessPredicate)>,
}

impl Clause {
    /// TODO private me
    pub fn to_nameless<E: Error>(
        self,
        pred_env: &HashMap<(&str, usize), u32>,
    ) -> Result<NamelessClause, E> {
        let mut var_env = Vec::new();
        let args = self
            .head
            .args
            .into_iter()
            .map(|v| v.to_nameless(&mut var_env))
            .collect::<Result<_, _>>()?;
        let body = self
            .body
            .into_iter()
            .map(|(n, p)| Ok((n, p.to_nameless(pred_env, &mut var_env)?)))
            .collect::<Result<_, _>>()?;
        let vars = u32::try_from(var_env.len()).map_err(|_| {
            Error::invalid_query(
                "too many variables used (though this should've been caught earlier?)",
            )
        })?;
        Ok(NamelessClause { vars, args, body })
    }
}

/// A nameless representation of queries.
#[derive(Debug)]
pub struct NamelessQuery {
    /// The clauses to be used by the query, grouped by predicate, in stratified order.
    pub clauses: Vec<Vec<Clause>>,

    /// The value to solve for.
    pub predicate: Predicate,
}

impl From<Query> for NamelessQuery {
    fn from(q: Query) -> NamelessQuery {
        // Group the clauses by their functor.
        let mut clauses = HashMap::<_, Vec<_>>::new();
        for clause in q.clauses {
            let functor = (clause.head.name, clause.head.args.len());
            clauses
                .entry(functor)
                .or_default()
                .push((clause.head.args, clause.body));
        }
        dbg!(clauses);

        // Create the original predicate environment.
        // let mut pred_names = Vec::new(); // For ownership of the names.
        let mut pred_env = HashMap::new();
        let _ = pred_env.insert(("atom", 1), 0);
        let _ = pred_env.insert(("name", 3), 1);
        let _ = pred_env.insert(("edge", 3), 2);
        let _ = pred_env.insert(("tag", 3), 3);
        let _ = pred_env.insert(("blob", 4), 4);
        // let mut pred_env_counter = 5;

        // Collect the predicates.
        /*
        for clause in &q.clauses {
            let name: &str = &clause.head.name;
            let functor = (name, clause.head.args.len());
            if !pred_env.contains_key(&functor) {
                unimplemented!()
            }
        }
        */

        unimplemented!()
    }
}

//! A nameless representation of queries, which are easier to execute, easier to validate, and
//! easier to operate on.
//!
//! This representation names predicates with indices, collects clauses for the same predicate
//! together, sorts predicates to stratify them, and explicitly declare variables used in each
//! clause.

use crate::{
    query::{Predicate, Query, Value},
    utils::StringPool,
    Error,
};
use serde_derive::{Deserialize, Serialize};
use std::{collections::HashMap, convert::TryFrom, sync::Arc};

/// A nameless representation of values.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum NamelessValue {
    /// A signed integer.
    Int(i64),

    /// A string.
    Str(Arc<str>),

    /// A variable.
    Var(u32),
}

impl Value {
    fn to_nameless<E: Error>(
        self,
        strings: &mut StringPool,
        var_env: &mut Vec<String>,
    ) -> Result<NamelessValue, E> {
        match self {
            Value::Int(n) => Ok(NamelessValue::Int(n)),
            Value::Str(s) => Ok(NamelessValue::Str(strings.store(s))),
            Value::Var(v) => {
                let n = var_env.iter().position(|v2| &v == v2).unwrap_or_else(|| {
                    let n = var_env.len();
                    var_env.push(v);
                    n
                });
                let n = u32::try_from(n)
                    .map_err(|_| Error::invalid_query("too many variables used".to_string()))?;
                Ok(NamelessValue::Var(n))
            }
        }
    }
}

/// A nameless representation of predicates.
#[derive(Clone, Debug)]
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
        strings: &mut StringPool,
        pred_env: &HashMap<(String, usize), u32>,
        var_env: &mut Vec<String>,
    ) -> Result<NamelessPredicate, E> {
        let name = pred_env
            .get(&(self.name.to_string(), self.args.len()))
            .cloned()
            .ok_or_else(|| {
                Error::invalid_query(format!(
                    "undeclared predicate: {}/{}",
                    self.name,
                    self.args.len()
                ))
            })?;
        let args = self
            .args
            .into_iter()
            .map(|v| v.to_nameless(strings, var_env))
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

impl NamelessClause {
    fn from_args_body<E: Error>(
        args: Vec<Value>,
        body: Vec<(bool, Predicate)>,
        strings: &mut StringPool,
        pred_env: &HashMap<(String, usize), u32>,
    ) -> Result<NamelessClause, E> {
        let mut var_env = Vec::new();
        let args = args
            .into_iter()
            .map(|v| v.to_nameless(strings, &mut var_env))
            .collect::<Result<_, _>>()?;
        let body = body
            .into_iter()
            .map(|(n, p)| Ok((n, p.to_nameless(strings, pred_env, &mut var_env)?)))
            .collect::<Result<_, _>>()?;
        let vars = u32::try_from(var_env.len()).map_err(|_| {
            Error::invalid_query(
                "too many variables used (though this should've been caught earlier?)".to_string(),
            )
        })?;
        Ok(NamelessClause { vars, args, body })
    }
}

/// A nameless representation of queries.
#[derive(Debug)]
pub struct NamelessQuery {
    /// The clauses to be used by the query, grouped by predicate, in stratified order.
    pub clauses: Vec<Vec<NamelessClause>>,

    /// The number of variables used in the predicate to solve for.
    pub goal_vars: u32,

    /// The predicate to solve for.
    pub goal: NamelessPredicate,
}

impl NamelessQuery {
    /// Tries to parse a query, convert it to a `NamelessQuery`, and validate it.
    pub fn from_str<E: Error>(src: &str) -> Result<NamelessQuery, E> {
        let query = src
            .parse::<Query>()
            .map_err(|err| E::syntax_error(err.to_string()))?;
        let query = NamelessQuery::from_query(query)?;
        query.validate()?;
        Ok(query)
    }

    /// Tries to convert a `Query` to a `NamelessQuery`.
    pub fn from_query<E: Error>(q: Query) -> Result<NamelessQuery, E> {
        // Group the clauses by their functor.
        let mut all_clauses = HashMap::<_, Vec<_>>::new();
        for clause in q.clauses {
            let functor = (clause.head.name, clause.head.args.len());
            all_clauses
                .entry(functor)
                .or_default()
                .push((clause.head.args, clause.body));
        }

        // Sort the clauses.
        let mut toposort = topological_sort::TopologicalSort::<(&str, usize)>::new();
        for ((caller_name, caller_argn), clauses) in all_clauses.iter() {
            let caller_functor: (&str, _) = (caller_name, *caller_argn);
            let _ = toposort.insert(caller_functor);
            for (_, body) in clauses {
                for (_, pred) in body {
                    let callee_functor: (&str, _) = (&pred.name, pred.args.len());
                    if callee_functor != caller_functor {
                        toposort.add_dependency(callee_functor, caller_functor);
                    }
                }
            }
        }

        // Check the toposort.
        let toposort_size = toposort.len();
        let toposort = toposort
            .map(|(f, c)| (f.to_string(), c))
            .collect::<Vec<_>>();
        if toposort.len() != toposort_size {
            return Err(E::invalid_query(format!("failed to stratify query")));
        }

        // Create the original predicate environment.
        let mut pred_env = HashMap::new();
        let _ = pred_env.insert(("atom".to_string(), 1), 0);
        let _ = pred_env.insert(("name".to_string(), 3), 1);
        let _ = pred_env.insert(("edge".to_string(), 3), 2);
        let _ = pred_env.insert(("tag".to_string(), 3), 3);
        let _ = pred_env.insert(("blob".to_string(), 4), 4);
        let mut pred_env_counter = 5;

        // Convert the clauses, filling in the predicate environment.
        let mut strings = StringPool::default();
        let clauses = toposort
            .into_iter()
            .map(|functor| {
                let clauses = all_clauses
                    .remove(&(functor.0.to_string(), functor.1))
                    .ok_or_else(|| {
                        E::invalid_query(format!(
                            "undeclared predicate: {}/{}",
                            functor.0, functor.1
                        ))
                    })?;

                // Add the predicate to the environment.
                let n = pred_env_counter;
                if pred_env.insert(functor.clone(), n).is_some() {
                    return Err(E::invalid_query(format!(
                        "redefining existing predicate: {}/{}",
                        functor.0, functor.1
                    )));
                }
                pred_env_counter += 1;

                // Transform each clause
                clauses
                    .into_iter()
                    .map(|(args, body)| {
                        NamelessClause::from_args_body(args, body, &mut strings, &pred_env)
                    })
                    .collect()
            })
            .collect::<Result<Vec<Vec<NamelessClause>>, _>>()?;

        // Convert the predicate to solve for.
        let mut var_env = Vec::new();
        let goal = q.goal.to_nameless(&mut strings, &pred_env, &mut var_env)?;
        let goal_vars = u32::try_from(var_env.len())
            .map_err(|_| Error::invalid_query("too many variables used".to_string()))?;

        // Return.
        Ok(NamelessQuery {
            clauses,
            goal_vars,
            goal,
        })
    }
}

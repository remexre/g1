//! A naive solver for queries.
//!
//! This should probably not be used for anything except for very small databases and tests. It can
//! also serve as a reference implementation to compare against for more optimized versions.

use crate::nameless::{NamelessClause, NamelessPredicate, NamelessQuery, NamelessValue};
use std::{collections::HashSet, iter::once, sync::Arc};

/// Naively solves the given query in a self-contained way (i.e. with all builtin goals failing).
pub fn naive_solve_selfcontained(query: &NamelessQuery) -> Vec<Vec<Arc<str>>> {
    naive_solve(&[], &[], &[], &[], &[], None, query)
}

/// Naively solves the given query.
///
/// TODO: prose
///
/// - `atoms`: `atom`
/// - `names`: `atom, namespace, title`
/// - `edges`: `to, from, label`
/// - `tags`: `atom, key, value`
/// - `blobs`: `atom, kind, mime, hash`
pub fn naive_solve(
    atoms: &[Arc<str>],
    names: &[(Arc<str>, Arc<str>, Arc<str>)],
    edges: &[(Arc<str>, Arc<str>, Arc<str>)],
    tags: &[(Arc<str>, Arc<str>, Arc<str>)],
    blobs: &[(Arc<str>, Arc<str>, Arc<str>, Arc<str>)],
    limit: Option<usize>,
    query: &NamelessQuery,
) -> Vec<Vec<Arc<str>>> {
    let mut tuples = vec![HashSet::new(); query.clauses.len() + 5];

    // Add all the builtin tuples.
    tuples[0].extend(atoms.iter().map(|atom| vec![atom.clone()]));
    tuples[1].extend(
        names
            .iter()
            .map(|(atom, ns, title)| vec![atom.clone(), ns.clone(), title.clone()]),
    );
    tuples[2].extend(
        edges
            .iter()
            .map(|(to, from, label)| vec![to.clone(), from.clone(), label.clone()]),
    );
    tuples[3].extend(
        tags.iter()
            .map(|(atom, key, value)| vec![atom.clone(), key.clone(), value.clone()]),
    );
    tuples[4].extend(blobs.iter().map(|(atom, kind, mime, hash)| {
        vec![atom.clone(), kind.clone(), mime.clone(), hash.clone()]
    }));

    // For each predicate, compute its tuples.
    for (pred_idx, pred) in query.clauses.iter().enumerate() {
        // Repeatedly compute new tuples until no new tuples are added. This is needed to handle
        // recursion.
        loop {
            let mut new_tuples = HashSet::new();
            for clause in pred {
                new_tuples.extend(compute_new_tuples(&tuples, clause));
            }

            // Remove the tuples already computed.
            new_tuples.retain(|x| !tuples[pred_idx + 5].contains(x));

            // If no new tuples were computed, we can stop.
            if new_tuples.is_empty() {
                break;
            }

            // Otherwise, add the new tuples in.
            tuples[pred_idx + 5].extend(new_tuples);
        }
    }

    // Grab the tuples of the goal.
    let iter = tuples
        .remove(query.goal.name as usize)
        .into_iter()
        .filter(|tuple| {
            let mut vars = (0..query.goal_vars).map(|_| None).collect::<Vec<_>>();
            tuple
                .iter()
                .zip(&query.goal.args)
                .all(|(val, arg)| match arg {
                    NamelessValue::MetaVar(v) => panic!("unfilled metavariable: ${}", v),
                    NamelessValue::Str(s) => s == val,
                    NamelessValue::Var(n) => match &vars[*n as usize] {
                        Some(s) => s == &val,
                        None => {
                            vars[*n as usize] = Some(val);
                            true
                        }
                    },
                })
        });
    if let Some(limit) = limit {
        iter.take(limit).collect()
    } else {
        iter.collect()
    }
}

fn compute_new_tuples(
    tuples: &Vec<HashSet<Vec<Arc<str>>>>,
    clause: &NamelessClause,
) -> HashSet<Vec<Arc<str>>> {
    assert!(clause.body_neg.is_empty(), "TODO negation");

    make_envs(tuples, &clause.body_pos, clause.vars)
        .map(|env| {
            clause
                .head
                .iter()
                .map(|x| match x {
                    NamelessValue::MetaVar(v) => panic!("unfilled metavariable: ${}", v),
                    NamelessValue::Str(s) => s,
                    NamelessValue::Var(n) => env[*n as usize].as_ref().unwrap(),
                })
                .cloned()
                .collect()
        })
        .collect()
}

fn make_envs<'a>(
    tuples: &'a Vec<HashSet<Vec<Arc<str>>>>,
    body: &'a [NamelessPredicate],
    vars: u32,
) -> Box<dyn Iterator<Item = Vec<Option<Arc<str>>>> + 'a> {
    if body.is_empty() {
        Box::new(once((0..vars).map(|_| None).collect::<Vec<_>>()))
    } else {
        let pred = &body[0];
        Box::new(make_envs(tuples, &body[1..], vars).flat_map(move |env| {
            tuples[pred.name as usize].iter().filter_map(move |tuple| {
                let mut env = env.clone();
                for (arg, val) in pred.args.iter().zip(tuple) {
                    match arg {
                        NamelessValue::MetaVar(v) => panic!("unfilled metavariable: ${}", v),
                        NamelessValue::Str(s) => {
                            if s != val {
                                return None;
                            }
                        }
                        NamelessValue::Var(n) => {
                            let slot = &mut env[*n as usize];
                            if let Some(s) = slot {
                                if s != val {
                                    return None;
                                }
                            } else {
                                *slot = Some(val.clone());
                            }
                        }
                    }
                }
                Some(env)
            })
        }))
    }
}

//! A naive solver for queries.
//!
//! This should probably not be used for anything except for very small databases and tests. It can
//! also serve as a reference implementation to compare against for more optimized versions.

use crate::nameless::{NamelessClause, NamelessPredicate, NamelessQuery, NamelessValue};
use futures::{
    never::Never,
    prelude::*,
    stream::{empty, select},
};
use std::{collections::HashSet, sync::Arc};

/// Naively solves the given query in a self-contained way (i.e. with all builtin goals failing).
pub fn naive_solve_selfcontained(query: &NamelessQuery) -> Vec<Vec<NamelessValue>> {
    naive_solve(empty(), empty(), empty(), empty(), empty(), query)
        .now_or_never()
        .unwrap()
        .unwrap_or_else(|e: Never| match e {})
}

/// Naively solves the given query.
pub async fn naive_solve<E, PA, PN, PE, PT, PB>(
    pred_atom: PA,
    pred_name: PN,
    pred_edge: PE,
    pred_tag: PT,
    pred_blob: PB,
    query: &NamelessQuery,
) -> Result<Vec<Vec<NamelessValue>>, E>
where
    PA: Stream<Item = Result<Arc<str>, E>>,
    PN: Stream<Item = Result<(Arc<str>, Arc<str>, Arc<str>), E>>,
    PE: Stream<Item = Result<(Arc<str>, Arc<str>, Arc<str>), E>>,
    PT: Stream<Item = Result<(Arc<str>, Arc<str>, Arc<str>), E>>,
    PB: Stream<Item = Result<(Arc<str>, Arc<str>, Arc<str>, Arc<str>), E>>,
{
    // Create a stream of all the builtin predicates.
    let mut stream = Box::pin(select(
        select(
            select(
                select(
                    pred_atom.map_ok(|atom| NamelessPredicate {
                        name: 0,
                        args: vec![NamelessValue::Str(atom)],
                    }),
                    pred_name.map_ok(|(atom, ns, title)| NamelessPredicate {
                        name: 1,
                        args: vec![
                            NamelessValue::Str(atom),
                            NamelessValue::Str(ns),
                            NamelessValue::Str(title),
                        ],
                    }),
                ),
                pred_edge.map_ok(|(from, to, label)| NamelessPredicate {
                    name: 2,
                    args: vec![
                        NamelessValue::Str(from),
                        NamelessValue::Str(to),
                        NamelessValue::Str(label),
                    ],
                }),
            ),
            pred_tag.map_ok(|(atom, key, value)| NamelessPredicate {
                name: 3,
                args: vec![
                    NamelessValue::Str(atom),
                    NamelessValue::Str(key),
                    NamelessValue::Str(value),
                ],
            }),
        ),
        pred_blob.map_ok(|(atom, kind, mime, hash)| NamelessPredicate {
            name: 4,
            args: vec![
                NamelessValue::Str(atom),
                NamelessValue::Str(kind),
                NamelessValue::Str(mime),
                NamelessValue::Str(hash),
            ],
        }),
    ));

    // Construct the model of the query.
    let mut model = Model {
        states: (0..5)
            .map(|_| State::default())
            .chain(query.clauses.iter().map(|clauses| State {
                clauses,
                ..State::default()
            }))
            .collect(),
    };

    // Fill out the facts in the query.
    for i in 0..model.states.len() {
        let state = &mut model.states[i];
        for clause in state.clauses.iter() {
            if clause.body.is_empty() {
                assert_eq!(clause.vars, 0);
                let _ = state.new.insert(clause.args.clone());
            }
        }

        model.propagate();
    }

    // Push each tuple from the stream into the model.
    while let Some(result) = stream.next().await {
        let tuple = result?;
        let _ = model.states[tuple.name as usize].new.insert(tuple.args);
        model.propagate();
    }

    unimplemented!("{:?}", model)
}

/// The model of a query being solved.
#[derive(Debug)]
struct Model<'a> {
    states: Vec<State<'a>>,
}

impl<'a> Model<'a> {
    /// Preconditions:
    ///
    /// 1. At most one state has a non-empty `new`.
    ///
    /// Postconditions:
    ///
    /// 1. For each state, `end.old = union(start.new, start.old)`.
    /// 2. For each state, `end.new.is_empty()`.
    fn propagate(&mut self) {
        // Get the index of the predicate to start at.
        let start = self.states.iter().position(|state| !state.new.is_empty());
        let start = if let Some(start) = start {
            start
        } else {
            // Implicitly we've checked that Precondition 1 holds, since zero states have a
            // non-empty `new`.
            //
            // Postcondition 1 holds, since we didn't modify anything.
            // Postcondition 2 holds, since all `new`s are empty.
            return;
        };

        // Checks precondition 1 on debug builds.
        cfg_if::cfg_if! {
            if #[cfg(debug_assertions)] {
                for state in &self.states[start+1..] {
                    assert_eq!(&state.new, &HashSet::new());
                }
            }
        }

        // For each predicate that could be affected...
        for i in start..self.states.len() {
            let state = &mut self.states[i];

            // Remove `new` items that were already in `old`.
            // Extra vars to appease borrowck.
            let old = &mut state.old;
            let new = &mut state.new;
            new.retain(|x| !old.contains(x));

            // If there are no longer any `new` options, we can skip further processing as an
            // optimization.
            if state.new.is_empty() {
                continue;
            }

            for state in &mut self.states[i..] {
                println!("{:#?}", state);
            }
        }

        unimplemented!("{:#?}", &self.states[start])
    }
}

/// The state of a predicate in the model.
#[derive(Debug, Default)]
struct State<'a> {
    /// The clauses that define the predicate.
    pub clauses: &'a [NamelessClause],

    /// Whether the predicate is frozen; i.e. no more tuples will be added to it.
    pub frozen: bool,

    /// The tuples that were just added.
    pub new: HashSet<Vec<NamelessValue>>,

    /// Tuples that have already had their effects processed.
    pub old: HashSet<Vec<NamelessValue>>,
}

/*
fn call<'a, E, FA, FN, FE, FT, FB>(
    pred_atom: &'a mut FA,
    pred_name: &'a mut FN,
    pred_edge: &'a mut FE,
    pred_tag: &'a mut FT,
    pred_blob: &'a mut FB,
    all_clauses: &'a [Vec<NamelessClause>],
    name: u32,
    mut args: Vec<NamelessValue>,
    state: State,
) -> Pin<Box<dyn Stream<Item = Result<State, E>> + 'a>>
where
    E: 'static,
    FA: FnMut(NamelessValue, State) -> Pin<Box<dyn Stream<Item = Result<State, E>>>>,
    FN: FnMut(
        NamelessValue,
        NamelessValue,
        NamelessValue,
        State,
    ) -> Pin<Box<dyn Stream<Item = Result<State, E>>>>,
    FE: FnMut(
        NamelessValue,
        NamelessValue,
        NamelessValue,
        State,
    ) -> Pin<Box<dyn Stream<Item = Result<State, E>>>>,
    FT: FnMut(
        NamelessValue,
        NamelessValue,
        NamelessValue,
        State,
    ) -> Pin<Box<dyn Stream<Item = Result<State, E>>>>,
    FB: FnMut(
        NamelessValue,
        NamelessValue,
        NamelessValue,
        NamelessValue,
        State,
    ) -> Pin<Box<dyn Stream<Item = Result<State, E>>>>,
{
    match name {
        0 => {
            debug_assert_eq!(args.len(), 1);
            let arg0 = args.pop().unwrap();
            pred_atom(arg0, state)
        }
        1 => {
            debug_assert_eq!(args.len(), 3);
            let arg2 = args.pop().unwrap();
            let arg1 = args.pop().unwrap();
            let arg0 = args.pop().unwrap();
            pred_name(arg0, arg1, arg2, state)
        }
        2 => {
            debug_assert_eq!(args.len(), 3);
            let arg2 = args.pop().unwrap();
            let arg1 = args.pop().unwrap();
            let arg0 = args.pop().unwrap();
            pred_edge(arg0, arg1, arg2, state)
        }
        3 => {
            debug_assert_eq!(args.len(), 3);
            let arg2 = args.pop().unwrap();
            let arg1 = args.pop().unwrap();
            let arg0 = args.pop().unwrap();
            pred_tag(arg0, arg1, arg2, state)
        }
        4 => {
            debug_assert_eq!(args.len(), 4);
            let arg3 = args.pop().unwrap();
            let arg2 = args.pop().unwrap();
            let arg1 = args.pop().unwrap();
            let arg0 = args.pop().unwrap();
            pred_blob(arg0, arg1, arg2, arg3, state)
        }
        _ => all_clauses[name as usize - 5]
            .iter()
            .map(|clause| {
                debug_assert_eq!(args.len(), clause.args.len());
                let _fresh = state.fresh;
                let mut state = State {
                    fresh: state.fresh + clause.vars,
                    subst: state.subst.clone(),
                };

                for (l, mut r) in args.iter().zip(clause.args.iter().rev().cloned()) {
                    r.offset_vars(clause.vars);
                    match unify(l, &r, state) {
                        Some(new_state) => state = new_state,
                        None => return empty().boxed(),
                    }
                }

                clause
                    .body
                    .iter()
                    .fold(once(ok(state)).boxed(), |stream, (neg, pred)| {
                        assert!(!neg); // TODO
                        let mut pred = (*pred).clone();
                        pred.offset_vars(clause.vars);
                        dbg!(pred);
                        stream
                            .map_ok(move |state| {
                                call(
                                    pred_atom,
                                    pred_name,
                                    pred_edge,
                                    pred_tag,
                                    pred_blob,
                                    all_clauses,
                                    pred.name,
                                    pred.args,
                                    state,
                                )
                            })
                            .try_flatten()
                            .boxed()
                    })
            })
            .fold(empty().boxed(), |l, r| l.chain(r).boxed()),
    }
}

impl NamelessValue {
    pub(crate) fn offset_vars(&mut self, amount: u32) {
        if let NamelessValue::Var(n) = self {
            *n += amount;
        }
    }
}

impl NamelessPredicate {
    pub(crate) fn offset_vars(&mut self, amount: u32) {
        for value in &mut self.args {
            value.offset_vars(amount);
        }
    }
}

fn reify(state: State, argn: u32) -> Vec<NamelessValue> {
    (0..argn)
        .map(|n| {
            let mut var = NamelessValue::Var(n);
            while let NamelessValue::Var(n) = var {
                if let Some(next) = state.subst.get(&n) {
                    var = next.clone();
                } else {
                    break;
                }
            }
            var
        })
        .collect()
}

fn unify(l: &NamelessValue, r: &NamelessValue, mut state: State) -> Option<State> {
    match (l, r) {
        (NamelessValue::Var(l), NamelessValue::Var(r)) if l == r => Some(state),
        (NamelessValue::Int(l), NamelessValue::Int(r)) if l == r => Some(state),
        (NamelessValue::Str(l), NamelessValue::Str(r)) if l == r => Some(state),

        (NamelessValue::Var(l), r) => {
            dbg!(state.subst.insert(*l, r.clone()));
            Some(state)
        }
        (l, NamelessValue::Var(r)) => {
            dbg!(state.subst.insert(*r, l.clone()));
            Some(state)
        }

        _ => None,
    }
}
*/

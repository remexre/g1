//! A naive solver for queries.

use crate::nameless::{NamelessClause, NamelessQuery, NamelessValue};
use futures::{never::Never, stream::empty, FutureExt, Stream};
use std::{collections::HashSet, sync::Arc};

/// Naively solves the given query in a self-contained way (i.e. with all builtin goals failing).
pub fn naive_solve_selfcontained(
    limit: Option<usize>,
    query: &NamelessQuery,
) -> Vec<Vec<NamelessValue>> {
    naive_solve(empty(), empty(), empty(), empty(), empty(), limit, query)
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
    limit: Option<usize>,
    query: &NamelessQuery,
) -> Result<Vec<Vec<NamelessValue>>, E>
where
    PA: Stream<Item = Vec<Arc<str>>>,
    PN: Stream<Item = Vec<(Arc<str>, Arc<str>, Arc<str>)>>,
    PE: Stream<Item = Vec<(Arc<str>, Arc<str>, Arc<str>)>>,
    PT: Stream<Item = Vec<(Arc<str>, Arc<str>, Arc<str>)>>,
    PB: Stream<Item = Vec<(Arc<str>, Arc<str>, Arc<str>, Arc<str>)>>,
{
    unimplemented!()
}

/// The state of a query being solved.
#[derive(Debug)]
struct State<'a> {
    clauses: &'a [NamelessClause],
}

/// The state of a predicate.
#[derive(Debug)]
struct PredState {
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

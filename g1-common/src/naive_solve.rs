//! A naive solver for queries.
//!
//! This is approximately a miniKanren implementation, actually.

use crate::nameless::{NamelessClause, NamelessPredicate, NamelessQuery, NamelessValue};
use futures::{
    future::ok,
    never::Never,
    stream::{empty, once},
    FutureExt, Stream, StreamExt, TryStreamExt,
};
use std::{collections::HashMap, pin::Pin};

/// The type of the state being passed between goals.
#[derive(Clone, Debug)]
pub struct State {
    /// The index of the next fresh variable to generate.
    pub fresh: u32,

    /// The current substitution between variables.
    pub subst: HashMap<u32, NamelessValue>,
}

/// Naively solves the given query in a self-contained way (i.e. with all builtin goals failing).
pub fn naive_solve_selfcontained(
    limit: Option<usize>,
    query: &NamelessQuery,
) -> Vec<Vec<NamelessValue>> {
    naive_solve(
        |_, _| empty().boxed(),
        |_, _, _, _| empty().boxed(),
        |_, _, _, _| empty().boxed(),
        |_, _, _, _| empty().boxed(),
        |_, _, _, _, _| empty().boxed(),
        limit,
        query,
    )
    .now_or_never()
    .unwrap()
    .unwrap_or_else(|e: Never| match e {})
}

/// Naively solves the given query.
pub async fn naive_solve<E, FA, FN, FE, FT, FB>(
    mut pred_atom: FA,
    mut pred_name: FN,
    mut pred_edge: FE,
    mut pred_tag: FT,
    mut pred_blob: FB,
    limit: Option<usize>,
    query: &NamelessQuery,
) -> Result<Vec<Vec<NamelessValue>>, E>
where
    E: 'static + Send,
    FA: FnMut(NamelessValue, State) -> Pin<Box<dyn Stream<Item = Result<State, E>> + Send>> + Send,
    FN: FnMut(
            NamelessValue,
            NamelessValue,
            NamelessValue,
            State,
        ) -> Pin<Box<dyn Stream<Item = Result<State, E>> + Send>>
        + Send,
    FE: FnMut(
            NamelessValue,
            NamelessValue,
            NamelessValue,
            State,
        ) -> Pin<Box<dyn Stream<Item = Result<State, E>> + Send>>
        + Send,
    FT: FnMut(
            NamelessValue,
            NamelessValue,
            NamelessValue,
            State,
        ) -> Pin<Box<dyn Stream<Item = Result<State, E>> + Send>>
        + Send,
    FB: FnMut(
            NamelessValue,
            NamelessValue,
            NamelessValue,
            NamelessValue,
            State,
        ) -> Pin<Box<dyn Stream<Item = Result<State, E>> + Send>>
        + Send,
{
    let argn = query.goal.args.len() as u32;
    let stream = call(
        &mut pred_atom,
        &mut pred_name,
        &mut pred_edge,
        &mut pred_tag,
        &mut pred_blob,
        &query.clauses,
        query.goal.name,
        query.goal.args.clone(),
        State {
            fresh: argn,
            subst: HashMap::new(),
        },
    );
    if let Some(limit) = limit {
        stream
            .take(limit)
            .map_ok(|state| reify(state, argn))
            .try_collect()
            .await
    } else {
        stream
            .map_ok(|state| reify(state, argn))
            .try_collect()
            .await
    }
}

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
) -> Pin<Box<dyn Stream<Item = Result<State, E>> + Send + 'a>>
where
    E: 'static + Send,
    FA: FnMut(NamelessValue, State) -> Pin<Box<dyn Stream<Item = Result<State, E>> + Send>> + Send,
    FN: FnMut(
            NamelessValue,
            NamelessValue,
            NamelessValue,
            State,
        ) -> Pin<Box<dyn Stream<Item = Result<State, E>> + Send>>
        + Send,
    FE: FnMut(
            NamelessValue,
            NamelessValue,
            NamelessValue,
            State,
        ) -> Pin<Box<dyn Stream<Item = Result<State, E>> + Send>>
        + Send,
    FT: FnMut(
            NamelessValue,
            NamelessValue,
            NamelessValue,
            State,
        ) -> Pin<Box<dyn Stream<Item = Result<State, E>> + Send>>
        + Send,
    FB: FnMut(
            NamelessValue,
            NamelessValue,
            NamelessValue,
            NamelessValue,
            State,
        ) -> Pin<Box<dyn Stream<Item = Result<State, E>> + Send>>
        + Send,
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

//! A simple graph store.

use proc_macro_hack::proc_macro_hack;

pub use g1_common as common;

/// Embeds a `Query`.
///
/// ```
/// # use g1::{
/// #     common::validated::{
/// #         ValidatedClause, ValidatedPredicate, ValidatedValue, ValidatedValueInner,
/// #     },
/// #     Query, query
/// # };
/// # use pretty_assertions::assert_eq;
/// let val = "dunno";
/// let macro_query = query! {
///     thingy(X) :- atom(X).
///     thingy("dunno").
///
///     related(X, X) :- thingy(X).
///     related(X, Y) :-
///         edge(X, Y, "related"),
///         !edge(Y, X, "actually ignore that").
///
///     ?- related($val, X).
/// };
///
/// let manual_query = Query {
///     clauses: vec![
///         ValidatedClause {
///             head: ValidatedPredicate {
///                 name: 0,
///                 args: vec![
///                     ValidatedValue {
///                         inner: ValidatedValueInner::Var(0),
///                         span: (),
///                     },
///                 ],
///                 span: (),
///             },
///             body: vec![
///                 (false, ValidatedPredicate {
///                     name: -1,
///                     args: vec![
///                         ValidatedValue {
///                             inner: ValidatedValueInner::Var(0),
///                             span: (),
///                         },
///                     ],
///                     span: (),
///                 }),
///             ],
///             vars: 1,
///             span: (),
///         },
///         ValidatedClause {
///             head: ValidatedPredicate {
///                 name: 0,
///                 args: vec![
///                     ValidatedValue {
///                         inner: ValidatedValueInner::Str("dunno".into()),
///                         span: (),
///                     },
///                 ],
///                 span: (),
///             },
///             body: Vec::new(),
///             vars: 0,
///             span: (),
///         },
///         ValidatedClause {
///             head: ValidatedPredicate {
///                 name: 1,
///                 args: vec![
///                     ValidatedValue {
///                         inner: ValidatedValueInner::Var(0),
///                         span: (),
///                     },
///                     ValidatedValue {
///                         inner: ValidatedValueInner::Var(0),
///                         span: (),
///                     },
///                 ],
///                 span: (),
///             },
///             body: vec![
///                 (false, ValidatedPredicate {
///                     name: 0,
///                     args: vec![
///                         ValidatedValue {
///                             inner: ValidatedValueInner::Var(0),
///                             span: (),
///                         },
///                     ],
///                     span: (),
///                 }),
///             ],
///             vars: 1,
///             span: (),
///         },
///         ValidatedClause {
///             head: ValidatedPredicate {
///                 name: 1,
///                 args: vec![
///                     ValidatedValue {
///                         inner: ValidatedValueInner::Var(0),
///                         span: (),
///                     },
///                     ValidatedValue {
///                         inner: ValidatedValueInner::Var(1),
///                         span: (),
///                     },
///                 ],
///                 span: (),
///             },
///             body: vec![
///                 (false, ValidatedPredicate {
///                     name: -3,
///                     args: vec![
///                         ValidatedValue {
///                             inner: ValidatedValueInner::Var(0),
///                             span: (),
///                         },
///                         ValidatedValue {
///                             inner: ValidatedValueInner::Var(1),
///                             span: (),
///                         },
///                         ValidatedValue {
///                             inner: ValidatedValueInner::Str("related".into()),
///                             span: (),
///                         },
///                     ],
///                     span: (),
///                 }),
///                 (true, ValidatedPredicate {
///                     name: -3,
///                     args: vec![
///                         ValidatedValue {
///                             inner: ValidatedValueInner::Var(1),
///                             span: (),
///                         },
///                         ValidatedValue {
///                             inner: ValidatedValueInner::Var(0),
///                             span: (),
///                         },
///                         ValidatedValue {
///                             inner: ValidatedValueInner::Str("actually ignore that".into()),
///                             span: (),
///                         },
///                     ],
///                     span: (),
///                 }),
///             ],
///             vars: 2,
///             span: (),
///         },
///     ],
///     goal: ValidatedPredicate {
///         name: 1,
///         args: vec![
///             ValidatedValue {
///                 inner: ValidatedValueInner::Str("dunno".into()),
///                 span: (),
///             },
///             ValidatedValue {
///                 inner: ValidatedValueInner::Var(0),
///                 span: (),
///             },
///         ],
///         span: (),
///     },
///     goal_vars: 1,
///     span: (),
/// };
///
/// assert_eq!(
///     format!("{:?}", macro_query),
///     format!("{:?}", manual_query),
/// );
/// ```
#[proc_macro_hack]
pub use g1_macros::query;

/// A complete query to the database.
pub type Query<Span = ()> = g1_common::validated::ValidatedQuery<Span>;

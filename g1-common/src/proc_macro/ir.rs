//! The types used by the `proc_macro` IR.

use crate::proc_macro::{
    parser::QueryParser,
    token::{tokenstream_to_tokens, Span, Token},
};
use lalrpop_util::ParseError;
use proc_macro2::{Ident, TokenStream};
use std::convert::Infallible;
use syn::LitStr;

/// The actual data inside the `Value` type.
#[derive(Debug)]
pub enum Value {
    /// A hole.
    Hole(Span),

    /// An identifier. This represents a Rust variable being interpolated in.
    Ident(String, Ident),

    /// A string literal.
    String(String, LitStr),

    /// A variable.
    Var(String, Ident),
}

/// A call to a rule.
#[derive(Debug)]
pub struct Predicate {
    /// The name of the predicate.
    pub name: String,

    /// The name of the predicate, as an identifier.
    pub name_ident: Ident,

    /// The arguments to the predicate.
    pub args: Vec<Value>,

    /// The source span of the predicate.
    pub span: Span,
}

/// A single clause, used for deduction.
#[derive(Debug)]
pub struct Clause {
    /// The head of the clause.
    pub head: Predicate,

    /// The body of the clause.
    ///
    /// The boolean corresponds to whether the predicate is negated; it is negated when the boolean
    /// is `true`.
    pub body: Vec<(bool, Predicate)>,

    /// The source span of the clause.
    pub span: Span,
}

/// A complete query to the database.
///
/// ```
/// # use g1_common::proc_macro::{ir::{Clause, Predicate, Query, Value}, token::Span};
/// # use pretty_assertions::assert_eq;
/// # use proc_macro2::Ident;
/// # use quote::quote;
/// # use syn::LitStr;
/// let actual = Query::parse(quote! {
///     edge("A", "B").
///     edge("A", "C").
///     edge("B", "C").
///
///     path(X, X).
///     path(X, Z) :-
///         path(X, Y),
///         edge(Y, Z).
///
///     ?- path($a, X).
/// }).unwrap();
/// let expected = Query {
///     clauses: vec![
///         Clause {
///             head: Predicate {
///                 name: Ident::new("edge", Span::default().into()),
///                 args: vec![
///                     Value::String(LitStr::new("A", Span::default().into())),
///                     Value::String(LitStr::new("B", Span::default().into())),
///                 ],
///                 span: Span::default(),
///             },
///             body: Vec::new(),
///             span: Span::default(),
///         },
///         Clause {
///             head: Predicate {
///                 name: Ident::new("edge", Span::default().into()),
///                 args: vec![
///                     Value::String(LitStr::new("A", Span::default().into())),
///                     Value::String(LitStr::new("C", Span::default().into())),
///                 ],
///                 span: Span::default(),
///             },
///             body: Vec::new(),
///             span: Span::default(),
///         },
///         Clause {
///             head: Predicate {
///                 name: Ident::new("edge", Span::default().into()),
///                 args: vec![
///                     Value::String(LitStr::new("B", Span::default().into())),
///                     Value::String(LitStr::new("C", Span::default().into())),
///                 ],
///                 span: Span::default(),
///             },
///             body: Vec::new(),
///             span: Span::default(),
///         },
///         Clause {
///             head: Predicate {
///                 name: Ident::new("path", Span::default().into()),
///                 args: vec![
///                     Value::Var(Ident::new("X", Span::default().into())),
///                     Value::Var(Ident::new("X", Span::default().into())),
///                 ],
///                 span: Span::default(),
///             },
///             body: Vec::new(),
///             span: Span::default(),
///         },
///         Clause {
///             head: Predicate {
///                 name: Ident::new("path", Span::default().into()),
///                 args: vec![
///                     Value::Var(Ident::new("X", Span::default().into())),
///                     Value::Var(Ident::new("Z", Span::default().into())),
///                 ],
///                 span: Span::default(),
///             },
///             body: vec![
///                 (false, Predicate {
///                     name: Ident::new("path", Span::default().into()),
///                     args: vec![
///                         Value::Var(Ident::new("X", Span::default().into())),
///                         Value::Var(Ident::new("Y", Span::default().into())),
///                     ],
///                     span: Span::default(),
///                 }),
///                 (false, Predicate {
///                     name: Ident::new("edge", Span::default().into()),
///                     args: vec![
///                         Value::Var(Ident::new("Y", Span::default().into())),
///                         Value::Var(Ident::new("Z", Span::default().into())),
///                     ],
///                     span: Span::default(),
///                 }),
///             ],
///             span: Span::default(),
///         },
///     ],
///     goal: Predicate {
///         name: Ident::new("path", Span::default().into()),
///         args: vec![
///             Value::Ident(Ident::new("a", Span::default().into())),
///             Value::Var(Ident::new("X", Span::default().into())),
///         ],
///         span: Span::default(),
///     },
///     span: Span::default(),
/// };
/// assert_eq!(
///     format!("{:?}", actual),
///     format!("{:?}", expected),
/// );
/// ```
#[derive(Debug)]
pub struct Query {
    /// The clauses to be used by the query.
    pub clauses: Vec<Clause>,

    /// The predicate to solve for.
    pub goal: Predicate,

    /// The source span of the query.
    pub span: Span,
}

impl Query {
    /// Parses a query from a `TokenStream`.
    pub fn parse(token_stream: TokenStream) -> Result<Query, ParseError<Span, Token, Infallible>> {
        let tokens = tokenstream_to_tokens(token_stream);
        QueryParser::new().parse(tokens.into_iter().map(|tok| {
            let span = tok.span();
            (span, tok, span)
        }))
    }
}

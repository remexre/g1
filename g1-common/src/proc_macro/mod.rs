//! An implementation of the G1 query language made for the `query!` proc macro.

mod parser {
    pub use self::parser::*;
    use lalrpop_util::lalrpop_mod;

    lalrpop_mod!(parser, "/proc_macro/parser.rs");
}
pub mod token;

use crate::proc_macro::token::{tokenstream_to_tokens, Span, Token};
use lalrpop_util::ParseError;
use proc_macro2::{Ident, Literal, TokenStream};
use std::convert::Infallible;

/// The actual data inside the `Value` type.
#[derive(Debug)]
pub enum Value {
    /// A hole.
    Hole(Span),

    /// An identifier. This represents a Rust variable being interpolated in.
    Ident(Ident),

    /// A literal.
    Literal(Literal),

    /// A variable.
    Var(Ident),
}

/// A call to a rule.
#[derive(Debug)]
pub struct Predicate {
    /// The name of the predicate.
    pub name: Ident,

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
/// # use g1_common::proc_macro::{token::Span, Clause, Predicate, Query, Value};
/// # use pretty_assertions::assert_eq;
/// # use proc_macro2::{Ident, Literal};
/// # use quote::quote;
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
///                     Value::Literal(Literal::string("A")),
///                     Value::Literal(Literal::string("B")),
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
///                     Value::Literal(Literal::string("A")),
///                     Value::Literal(Literal::string("C")),
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
///                     Value::Literal(Literal::string("B")),
///                     Value::Literal(Literal::string("C")),
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
        parser::QueryParser::new().parse(tokens.into_iter().map(|tok| {
            let span = tok.span();
            (span, tok, span)
        }))
    }
}

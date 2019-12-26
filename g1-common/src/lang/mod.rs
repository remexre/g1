//! The implementation of the G1 query language.
//!
//! This module declares an AST that is very close to the CST -- a `Query` from this module must be
//! `validate`d into a `ValidatedQuery<()>` in order to be sent.

mod lexer;
mod parser {
    pub use self::parser::*;
    use lalrpop_util::lalrpop_mod;

    lalrpop_mod!(parser, "/lang/parser.rs");
}

pub use crate::lang::lexer::{Point, Span, Token};
use crate::lang::{
    lexer::Lexer,
    parser::{ClauseParser, PredicateParser, QueryParser, ValueParser},
};
use derive_more::Display;
use lalrpop_util::ParseError;
use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    str::FromStr,
};

fn fmt_var(s: &str, fmt: &mut Formatter) -> FmtResult {
    let printable = s
        .chars()
        .all(|ch| ('A' <= ch && ch <= 'Z') || ('a' <= ch && ch <= 'z') || ch == '_' || ch == '-');
    if s.len() == 0 || !printable {
        fmt.write_str("'")?;
        for c in s.escape_default() {
            write!(fmt, "{}", c)?;
        }
        fmt.write_str("'")
    } else {
        write!(fmt, "{}", s)
    }
}

/// The actual data inside the `Value` type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ValueInner {
    /// A hole.
    Hole,

    /// A string.
    Str(String),

    /// A variable.
    Var(String),
}

impl Display for ValueInner {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        match self {
            ValueInner::Hole => fmt.write_str("_"),
            ValueInner::Str(s) => write!(fmt, "{:?}", s),
            ValueInner::Var(v) => fmt_var(v, fmt),
        }
    }
}

/// A data value.
///
/// ```
/// # use g1_common::lang::{Point, Span, Value, ValueInner};
/// # use pretty_assertions::assert_eq;
/// assert_eq!(r#""hello,\nworld!""#.parse(), Ok(Value {
///     inner: ValueInner::Str("hello,\nworld!".to_string()),
///     span: Span(Point(1, 0), Point(1, 16)),
/// }));
/// assert_eq!(r#"game"#.parse(), Ok(Value {
///     inner: ValueInner::Var("game".to_string()),
///     span: Span(Point(1, 0), Point(1, 4)),
/// }));
/// assert_eq!(r#"'osu!'"#.parse(), Ok(Value {
///     inner: ValueInner::Var("osu!".to_string()),
///     span: Span(Point(1, 0), Point(1, 6)),
/// }));
/// ```
#[derive(Clone, Debug, Display, Eq, PartialEq)]
#[display(fmt = "{}", inner)]
pub struct Value {
    /// The data.
    pub inner: ValueInner,

    /// The source span of the value.
    pub span: Span,
}

impl FromStr for Value {
    type Err = ParseError<Point, Token, String>;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        ValueParser::new().parse(Lexer::new(src))
    }
}

/// A call to a rule.
///
/// ```
/// # use g1_common::lang::{Point, Predicate, Span, Value, ValueInner};
/// # use pretty_assertions::assert_eq;
/// assert_eq!("''()".parse(), Ok(Predicate {
///     name: "".to_string(),
///     args: Vec::new(),
///     span: Span(Point(1, 0), Point(1, 4)),
/// }));
/// assert_eq!(r#"'not equal'("one", "two")"#.parse(), Ok(Predicate {
///     name: "not equal".to_string(),
///     args: vec![
///         Value {
///             inner: ValueInner::Str("one".into()),
///             span: Span(Point(1, 12), Point(1, 17)),
///         },
///         Value {
///             inner: ValueInner::Str("two".into()),
///             span: Span(Point(1, 19), Point(1, 24)),
///         },
///     ],
///     span: Span(Point(1, 0), Point(1, 25)),
/// }));
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Predicate {
    /// The name of the predicate.
    pub name: String,

    /// The arguments to the predicate.
    pub args: Vec<Value>,

    /// The source span of the predicate.
    pub span: Span,
}

impl Display for Predicate {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        fmt_var(&self.name, fmt)?;
        fmt.write_str("(")?;
        let mut first = true;
        for arg in &self.args {
            if first {
                first = false;
            } else {
                fmt.write_str(", ")?;
            }
            write!(fmt, "{}", arg)?;
        }
        fmt.write_str(")")
    }
}

impl FromStr for Predicate {
    type Err = ParseError<Point, Token, String>;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        PredicateParser::new().parse(Lexer::new(src))
    }
}

/// A single clause, used for deduction.
///
/// ```
/// # use g1_common::lang::{Clause, Point, Predicate, Span, Value, ValueInner};
/// # use pretty_assertions::assert_eq;
/// assert_eq!("foo().".parse(), Ok(Clause {
///     head: Predicate {
///         name: "foo".to_string(),
///         args: Vec::new(),
///         span: Span(Point(1, 0), Point(1, 5)),
///     },
///     body: Vec::new(),
///     span: Span(Point(1, 0), Point(1, 6)),
/// }));
///
/// assert_eq!("bar(x) :- !baz(x), quux(x).".parse(), Ok(Clause {
///     head: Predicate {
///         name: "bar".to_string(),
///         args: vec![
///             Value {
///                 inner: ValueInner::Var("x".to_string()),
///                 span: Span(Point(1, 4), Point(1, 5)),
///             }
///         ],
///         span: Span(Point(1, 0), Point(1, 6)),
///     },
///     body: vec![
///         (true, Predicate {
///             name: "baz".to_string(),
///             args: vec![
///                 Value {
///                     inner: ValueInner::Var("x".to_string()),
///                     span: Span(Point(1, 15), Point(1, 16)),
///                 }
///             ],
///             span: Span(Point(1, 11), Point(1, 17)),
///         }),
///         (false, Predicate {
///             name: "quux".to_string(),
///             args: vec![
///                 Value {
///                     inner: ValueInner::Var("x".to_string()),
///                     span: Span(Point(1, 24), Point(1, 25)),
///                 }
///             ],
///             span: Span(Point(1, 19), Point(1, 26)),
///         }),
///     ],
///     span: Span(Point(1, 0), Point(1, 27)),
/// }));
///
/// assert_eq!("bar2(x) :- baz(x), !quux(x).".parse(), Ok(Clause {
///     head: Predicate {
///         name: "bar2".to_string(),
///         args: vec![
///             Value {
///                 inner: ValueInner::Var("x".to_string()),
///                 span: Span(Point(1, 5), Point(1, 6)),
///             }
///         ],
///         span: Span(Point(1, 0), Point(1, 7)),
///     },
///     body: vec![
///         (false, Predicate {
///             name: "baz".to_string(),
///             args: vec![
///                 Value {
///                     inner: ValueInner::Var("x".to_string()),
///                     span: Span(Point(1, 15), Point(1, 16)),
///                 }
///             ],
///             span: Span(Point(1, 11), Point(1, 17)),
///         }),
///         (true, Predicate {
///             name: "quux".to_string(),
///             args: vec![
///                 Value {
///                     inner: ValueInner::Var("x".to_string()),
///                     span: Span(Point(1, 25), Point(1, 26)),
///                 }
///             ],
///             span: Span(Point(1, 20), Point(1, 27)),
///         }),
///     ],
///     span: Span(Point(1, 0), Point(1, 28)),
/// }));
///
/// assert_eq!(
///     r#"
///         // Start searching from the end.
///         path(x, y) :-
///             path(x, z),
///             edge(z, y).
///     "#.parse(),
///     Ok(Clause {
///         head: Predicate {
///             name: "path".to_string(),
///             args: vec![
///                 Value {
///                     inner: ValueInner::Var("x".to_string()),
///                     span: Span(Point(3, 14), Point(3, 15)),
///                 },
///                 Value {
///                     inner: ValueInner::Var("y".to_string()),
///                     span: Span(Point(3, 17), Point(3, 18)),
///                 },
///             ],
///             span: Span(Point(3, 9), Point(3, 19)),
///         },
///         body: vec![
///             (false, Predicate {
///                 name: "path".to_string(),
///                 args: vec![
///                     Value {
///                         inner: ValueInner::Var("x".to_string()),
///                         span: Span(Point(4, 18), Point(4, 19)),
///                     },
///                     Value {
///                         inner: ValueInner::Var("z".to_string()),
///                         span: Span(Point(4, 21), Point(4, 22)),
///                     },
///                 ],
///                 span: Span(Point(4, 13), Point(4, 23)),
///             }),
///             (false, Predicate {
///                 name: "edge".to_string(),
///                 args: vec![
///                     Value {
///                         inner: ValueInner::Var("z".to_string()),
///                         span: Span(Point(5, 18), Point(5, 19)),
///                     },
///                     Value {
///                         inner: ValueInner::Var("y".to_string()),
///                         span: Span(Point(5, 21), Point(5, 22)),
///                     },
///                 ],
///                 span: Span(Point(5, 13), Point(5, 23)),
///             }),
///         ],
///         span: Span(Point(3, 9), Point(5, 24)),
///     }
/// ));
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
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

impl Display for Clause {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        if self.body.is_empty() {
            write!(fmt, "{}.", self.head)
        } else {
            writeln!(fmt, "{} :-", self.head)?;
            let l = self.body.len();
            for (i, (negated, pred)) in self.body.iter().enumerate() {
                fmt.write_str("    ")?;
                if *negated {
                    fmt.write_str("!")?;
                }
                write!(fmt, "{}", pred)?;
                fmt.write_str(if i == l - 1 { "." } else { ",\n" })?;
            }
            Ok(())
        }
    }
}

impl FromStr for Clause {
    type Err = ParseError<Point, Token, String>;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        ClauseParser::new().parse(Lexer::new(src))
    }
}

/// A complete query to the database.
///
/// ```
/// # use g1_common::lang::{Clause, Point, Predicate, Query, Span, Value, ValueInner};
/// # use pretty_assertions::assert_eq;
/// assert_eq!(
///     r#"
///         edge("A", "B").
///         edge("A", "C").
///         edge("B", "C").
///
///         path(X, X).
///         path(X, Z) :-
///             path(X, Y),
///             edge(Y, Z).
///
///         ?- path("A", X).
///     "#.parse(),
///     Ok(Query {
///         clauses: vec![
///             Clause {
///                 head: Predicate {
///                     name: "edge".to_string(),
///                     args: vec![
///                         Value {
///                             inner: ValueInner::Str("A".to_string()),
///                             span: Span(Point(2, 14), Point(2, 17)),
///                         },
///                         Value {
///                             inner: ValueInner::Str("B".to_string()),
///                             span: Span(Point(2, 19), Point(2, 22)),
///                         },
///                     ],
///                     span: Span(Point(2, 9), Point(2, 23)),
///                 },
///                 body: Vec::new(),
///                 span: Span(Point(2, 9), Point(2, 24)),
///             },
///             Clause {
///                 head: Predicate {
///                     name: "edge".to_string(),
///                     args: vec![
///                         Value {
///                             inner: ValueInner::Str("A".to_string()),
///                             span: Span(Point(3, 14), Point(3, 17)),
///                         },
///                         Value {
///                             inner: ValueInner::Str("C".to_string()),
///                             span: Span(Point(3, 19), Point(3, 22)),
///                         },
///                     ],
///                     span: Span(Point(3, 9), Point(3, 23)),
///                 },
///                 body: Vec::new(),
///                 span: Span(Point(3, 9), Point(3, 24)),
///             },
///             Clause {
///                 head: Predicate {
///                     name: "edge".to_string(),
///                     args: vec![
///                         Value {
///                             inner: ValueInner::Str("B".to_string()),
///                             span: Span(Point(4, 14), Point(4, 17)),
///                         },
///                         Value {
///                             inner: ValueInner::Str("C".to_string()),
///                             span: Span(Point(4, 19), Point(4, 22)),
///                         },
///                     ],
///                     span: Span(Point(4, 9), Point(4, 23)),
///                 },
///                 body: Vec::new(),
///                 span: Span(Point(4, 9), Point(4, 24)),
///             },
///             Clause {
///                 head: Predicate {
///                     name: "path".to_string(),
///                     args: vec![
///                         Value {
///                             inner: ValueInner::Var("X".to_string()),
///                             span: Span(Point(6, 14), Point(6, 15)),
///                         },
///                         Value {
///                             inner: ValueInner::Var("X".to_string()),
///                             span: Span(Point(6, 17), Point(6, 18)),
///                         },
///                     ],
///                     span: Span(Point(6, 9), Point(6, 19)),
///                 },
///                 body: Vec::new(),
///                 span: Span(Point(6, 9), Point(6, 20)),
///             },
///             Clause {
///                 head: Predicate {
///                     name: "path".to_string(),
///                     args: vec![
///                         Value {
///                             inner: ValueInner::Var("X".to_string()),
///                             span: Span(Point(7, 14), Point(7, 15)),
///                         },
///                         Value {
///                             inner: ValueInner::Var("Z".to_string()),
///                             span: Span(Point(7, 17), Point(7, 18)),
///                         },
///                     ],
///                     span: Span(Point(7, 9), Point(7, 19)),
///                 },
///                 body: vec![
///                     (false, Predicate {
///                         name: "path".to_string(),
///                         args: vec![
///                             Value {
///                                 inner: ValueInner::Var("X".to_string()),
///                                 span: Span(Point(8, 18), Point(8, 19)),
///                             },
///                             Value {
///                                 inner: ValueInner::Var("Y".to_string()),
///                                 span: Span(Point(8, 21), Point(8, 22)),
///                             },
///                         ],
///                         span: Span(Point(8, 13), Point(8, 23)),
///                     }),
///                     (false, Predicate {
///                         name: "edge".to_string(),
///                         args: vec![
///                             Value {
///                                 inner: ValueInner::Var("Y".to_string()),
///                                 span: Span(Point(9, 18), Point(9, 19)),
///                             },
///                             Value {
///                                 inner: ValueInner::Var("Z".to_string()),
///                                 span: Span(Point(9, 21), Point(9, 22)),
///                             },
///                         ],
///                         span: Span(Point(9, 13), Point(9, 23)),
///                     }),
///                 ],
///                 span: Span(Point(7, 9), Point(9, 24)),
///             },
///         ],
///         goal: Predicate {
///             name: "path".to_string(),
///             args: vec![
///                 Value {
///                     inner: ValueInner::Str("A".to_string()),
///                     span: Span(Point(11, 17), Point(11, 20)),
///                 },
///                 Value {
///                     inner: ValueInner::Var("X".to_string()),
///                     span: Span(Point(11, 22), Point(11, 23)),
///                 },
///             ],
///             span: Span(Point(11, 12), Point(11, 24)),
///         },
///         span: Span(Point(1, 0), Point(11, 25)),
///     })
/// );
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Query {
    /// The clauses to be used by the query.
    pub clauses: Vec<Clause>,

    /// The predicate to solve for.
    pub goal: Predicate,

    /// The source span of the query.
    pub span: Span,
}

impl Display for Query {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        for clause in &self.clauses {
            writeln!(fmt, "{}", clause)?;
        }
        write!(fmt, "?- {}.", self.goal)
    }
}

impl FromStr for Query {
    type Err = ParseError<Point, Token, String>;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        QueryParser::new().parse(Lexer::new(src))
    }
}

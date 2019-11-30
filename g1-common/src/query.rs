#[cfg(feature = "parser")]
use crate::{
    lexer::{Lexer, Token},
    parser::{ClauseParser, PredicateParser, QueryParser, ValueParser},
};
#[cfg(feature = "parser")]
use lalrpop_util::ParseError;
use serde_derive::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result as FmtResult};
#[cfg(feature = "parser")]
use std::str::FromStr;

fn fmt_var(s: &str, fmt: &mut Formatter) -> FmtResult {
    if s.len() == 0
        || !s
            .chars()
            .all(|ch| ('A' <= ch && ch <= 'Z') || ('a' <= ch && ch <= 'z') || ch == '-')
    {
        fmt.write_str("'")?;
        for c in s.escape_default() {
            write!(fmt, "{}", c)?;
        }
        fmt.write_str("'")
    } else {
        write!(fmt, "{}", s)
    }
}

/// A data value.
///
/// ```
/// # use g1_common::Value;
/// assert_eq!("123".parse(), Ok(Value::Int(123)));
/// assert_eq!("-123".parse(), Ok(Value::Int(-123)));
/// assert_eq!(r#""hello,\nworld!""#.parse(), Ok(Value::String("hello,\nworld!".to_string())));
/// assert_eq!(r#"game"#.parse(), Ok(Value::Var("game".to_string())));
/// assert_eq!(r#"'osu!'"#.parse(), Ok(Value::Var("osu!".to_string())));
/// ```
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Value {
    /// A signed integer.
    Int(i64),

    /// A string.
    String(String),

    /// A free variable.
    Var(String),
}

impl Display for Value {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        match self {
            Value::Int(n) => write!(fmt, "{}", n),
            Value::String(s) => write!(fmt, "{:?}", s),
            Value::Var(v) => fmt_var(v, fmt),
        }
    }
}

#[cfg(feature = "parser")]
impl FromStr for Value {
    type Err = ParseError<(), (Token, String), &'static str>;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        ValueParser::new()
            .parse(Lexer::new(src))
            .map_err(|err| err.map_token(|(t, l)| (t, l.to_string())))
    }
}

/// A call to a rule.
///
/// ```
/// # use g1_common::{Predicate, Value};
/// assert_eq!("''()".parse(), Ok(Predicate {
///     name: "".to_string(),
///     args: Vec::new(),
/// }));
/// assert_eq!("'not equal'(1, -2)".parse(), Ok(Predicate {
///     name: "not equal".to_string(),
///     args: vec![
///         Value::Int(1),
///         Value::Int(-2),
///     ],
/// }));
/// ```
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Predicate {
    /// The name of the predicate.
    pub name: String,

    /// The arguments to the predicate.
    pub args: Vec<Value>,
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

#[cfg(feature = "parser")]
impl FromStr for Predicate {
    type Err = ParseError<(), (Token, String), &'static str>;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        PredicateParser::new()
            .parse(Lexer::new(src))
            .map_err(|err| err.map_token(|(t, l)| (t, l.to_string())))
    }
}

/// A single clause, used for deduction.
///
/// ```
/// # use g1_common::{Clause, Predicate, Value};
/// assert_eq!("foo().".parse(), Ok(Clause {
///     head: Predicate {
///         name: "foo".to_string(),
///         args: Vec::new(),
///     },
///     body: Vec::new(),
/// }));
///
/// assert_eq!("bar(x) :- !baz(x), quux(x).".parse(), Ok(Clause {
///     head: Predicate {
///         name: "bar".to_string(),
///         args: vec![Value::Var("x".to_string())],
///     },
///     body: vec![
///         (true, Predicate {
///             name: "baz".to_string(),
///             args: vec![Value::Var("x".to_string())],
///         }),
///         (false, Predicate {
///             name: "quux".to_string(),
///             args: vec![Value::Var("x".to_string())],
///         }),
///     ],
/// }));
///
/// assert_eq!("bar2(x) :- baz(x), !quux(x).".parse(), Ok(Clause {
///     head: Predicate {
///         name: "bar2".to_string(),
///         args: vec![Value::Var("x".to_string())],
///     },
///     body: vec![
///         (false, Predicate {
///             name: "baz".to_string(),
///             args: vec![Value::Var("x".to_string())],
///         }),
///         (true, Predicate {
///             name: "quux".to_string(),
///             args: vec![Value::Var("x".to_string())],
///         }),
///     ],
/// }));
///
/// assert_eq!(
///     r#"
///         % Start searching from the end.
///         path(x, y) :-
///             path(x, z),
///             edge(z, y).
///     "#.parse(),
///     Ok(Clause {
///         head: Predicate {
///             name: "path".to_string(),
///             args: vec![
///                 Value::Var("x".to_string()),
///                 Value::Var("y".to_string()),
///             ],
///         },
///         body: vec![
///             (false, Predicate {
///                 name: "path".to_string(),
///                 args: vec![
///                     Value::Var("x".to_string()),
///                     Value::Var("z".to_string()),
///                 ],
///             }),
///             (false, Predicate {
///                 name: "edge".to_string(),
///                 args: vec![
///                     Value::Var("z".to_string()),
///                     Value::Var("y".to_string()),
///                 ],
///             }),
///         ],
///     }
/// ));
/// ```
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Clause {
    /// The head of the clause.
    pub head: Predicate,

    /// The body of the clause.
    ///
    /// The boolean corresponds to whether the predicate is negated; it is negated when the boolean
    /// is `true`.
    pub body: Vec<(bool, Predicate)>,
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

#[cfg(feature = "parser")]
impl FromStr for Clause {
    type Err = ParseError<(), (Token, String), &'static str>;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        ClauseParser::new()
            .parse(Lexer::new(src))
            .map_err(|err| err.map_token(|(t, l)| (t, l.to_string())))
    }
}

/// A complete query to the database.
///
/// ```
/// # use g1_common::{Clause, Predicate, Query, Value};
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
///                         Value::String("A".to_string()),
///                         Value::String("B".to_string()),
///                     ],
///                 },
///                 body: Vec::new(),
///             },
///             Clause {
///                 head: Predicate {
///                     name: "edge".to_string(),
///                     args: vec![
///                         Value::String("A".to_string()),
///                         Value::String("C".to_string()),
///                     ],
///                 },
///                 body: Vec::new(),
///             },
///             Clause {
///                 head: Predicate {
///                     name: "edge".to_string(),
///                     args: vec![
///                         Value::String("B".to_string()),
///                         Value::String("C".to_string()),
///                     ],
///                 },
///                 body: Vec::new(),
///             },
///             Clause {
///                 head: Predicate {
///                     name: "path".to_string(),
///                     args: vec![
///                         Value::Var("X".to_string()),
///                         Value::Var("X".to_string()),
///                     ],
///                 },
///                 body: Vec::new(),
///             },
///             Clause {
///                 head: Predicate {
///                     name: "path".to_string(),
///                     args: vec![
///                         Value::Var("X".to_string()),
///                         Value::Var("Z".to_string()),
///                     ],
///                 },
///                 body: vec![
///                     (false, Predicate {
///                         name: "path".to_string(),
///                         args: vec![
///                             Value::Var("X".to_string()),
///                             Value::Var("Y".to_string()),
///                         ],
///                     }),
///                     (false, Predicate {
///                         name: "edge".to_string(),
///                         args: vec![
///                             Value::Var("Y".to_string()),
///                             Value::Var("Z".to_string()),
///                         ],
///                     }),
///                 ],
///             },
///         ],
///         predicate: Predicate {
///             name: "path".to_string(),
///             args: vec![
///                 Value::String("A".to_string()),
///                 Value::Var("X".to_string()),
///             ],
///         },
///     })
/// );
/// ```
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Query {
    /// The clauses to be used by the query.
    pub clauses: Vec<Clause>,

    /// The value to solve for.
    pub predicate: Predicate,
}

impl Display for Query {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        for clause in &self.clauses {
            writeln!(fmt, "{}", clause)?;
        }
        write!(fmt, "?- {}.", self.predicate)
    }
}

#[cfg(feature = "parser")]
impl FromStr for Query {
    type Err = ParseError<(), (Token, String), &'static str>;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        QueryParser::new()
            .parse(Lexer::new(src))
            .map_err(|err| err.map_token(|(t, l)| (t, l.to_string())))
    }
}

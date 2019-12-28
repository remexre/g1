use derive_more::Display;
use logos::Logos;

/// A point.
#[derive(Clone, Copy, Debug, Display, Eq, PartialEq)]
#[display(fmt = "{}:{}", _0, _1)]
pub struct Point(pub usize, pub usize);

impl Default for Point {
    fn default() -> Point {
        Point(1, 0)
    }
}

/// A span in the file.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Span(pub Point, pub Point);

/// A lexer over strings, producing `Token`s.
pub struct Lexer<'src> {
    inner: logos::Lexer<Tok, &'src str>,
    last_line: usize,
    last_line_pos: usize,
    last_pos: usize,
}

impl<'src> Lexer<'src> {
    /// Creates a new `Lexer`.
    pub fn new(src: &'src str) -> Lexer {
        Lexer {
            inner: Tok::lexer(src),
            last_line: 1,
            last_line_pos: 0,
            last_pos: 0,
        }
    }

    fn point(&mut self, n: usize) -> Point {
        assert!(n >= self.last_pos);
        let bytes = self.inner.source.as_bytes();
        for i in self.last_pos..n {
            if bytes[i] == b'\n' {
                self.last_line_pos = i;
                self.last_line += 1;
            }
        }
        self.last_pos = n;
        Point(self.last_line, n - self.last_line_pos)
    }

    fn range(&mut self) -> (Point, Point) {
        let range = self.inner.range();
        let start = self.point(range.start);
        let end = self.point(range.end);
        (start, end)
    }
}

impl<'src> Iterator for Lexer<'src> {
    type Item = Result<(Point, Token, Point), String>;

    fn next(&mut self) -> Option<Result<(Point, Token, Point), String>> {
        let out = loop {
            break match self.inner.token {
                Tok::End => None,
                Tok::Error => {
                    let start = self.point(self.inner.range().start);
                    Some(Err(format!("lexer error at {}", start)))
                }
                Tok::Comment => {
                    self.inner.advance();
                    continue;
                }
                Tok::ParenClose => Some(Ok(Token::ParenClose)),
                Tok::ParenOpen => Some(Ok(Token::ParenOpen)),
                Tok::Underscore => Some(Ok(Token::Underscore)),
                Tok::Period => Some(Ok(Token::Period)),
                Tok::Comma => Some(Ok(Token::Comma)),
                Tok::Query => Some(Ok(Token::Query)),
                Tok::Turnstile => Some(Ok(Token::Turnstile)),
                Tok::Not => Some(Ok(Token::Not)),
                Tok::String => {
                    let s = parse_stringish(self.inner.slice());
                    Some(Ok(Token::String(s)))
                }
                Tok::EscapedVar => {
                    let s = parse_stringish(self.inner.slice());
                    Some(Ok(Token::Var(s)))
                }
                Tok::Var => Some(Ok(Token::Var(self.inner.slice().to_string()))),
            };
        };
        let (start, end) = self.range();
        self.inner.advance();
        out.map(|r| r.map(|tok| (start, tok, end)))
    }
}

/// A token in the source code.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Token {
    /// A close parenthesis character (`)`).
    ParenClose,

    /// An open parenthesis character (`(`).
    ParenOpen,

    /// An underscore character ('_').
    Underscore,

    /// An exclaimation mark, `!`.
    Not,

    /// A comma character (',').
    Comma,

    /// A period character ('.').
    Period,

    /// The turnstile operator, `:-`.
    Turnstile,

    /// The "query operator", `?-`.
    Query,

    /// A string enclosed in double-quotes.
    String(String),

    /// A variable, either unescaped or enclosed in single-quotes.
    Var(String),
}

#[derive(Clone, Copy, Debug, Eq, Logos, PartialEq)]
enum Tok {
    #[end]
    End,

    #[error]
    Error,

    #[regex = "//[^\\r\\n]*"]
    Comment,

    #[token = ")"]
    ParenClose,

    #[token = "("]
    ParenOpen,

    #[token = "_"]
    Underscore,

    #[token = "."]
    Period,

    #[token = ","]
    Comma,

    #[token = "?-"]
    Query,

    #[token = ":-"]
    Turnstile,

    #[token = "!"]
    Not,

    #[regex = "\"([^'\"\\\\]|\\\\[trn'\"\\\\])*\""]
    String,

    #[regex = "'([^'\"\\\\]|\\\\[trn'\"\\\\])*'"]
    EscapedVar,

    #[regex = "[A-Za-z][0-9A-Za-z_]*"]
    Var,
}

#[derive(Clone, Copy, Debug, Eq, Logos, PartialEq)]
enum StringToken {
    #[end]
    End,

    #[error]
    Error,

    #[token = "\\t"]
    EscTab,

    #[token = "\\r"]
    EscCR,

    #[token = "\\n"]
    EscNL,

    #[token = "\\'"]
    EscSQuote,

    #[token = "\\\""]
    EscDQuote,

    #[token = "\\\\"]
    EscBackslash,

    #[regex = "[^'\"\\\\]"]
    Char,
}

fn parse_stringish(s: &str) -> String {
    assert!(s.len() >= 2);
    let s = &s[1..s.len() - 1];

    let mut lexer = StringToken::lexer(s);
    let mut out = String::new();

    loop {
        match lexer.token {
            StringToken::End => break,
            StringToken::Error => unreachable!(),
            StringToken::EscTab => out.push('\t'),
            StringToken::EscCR => out.push('\r'),
            StringToken::EscNL => out.push('\n'),
            StringToken::EscSQuote => out.push('\''),
            StringToken::EscDQuote => out.push('"'),
            StringToken::EscBackslash => out.push('\\'),
            StringToken::Char => out.push_str(lexer.slice()),
        }
        lexer.advance();
    }
    out
}

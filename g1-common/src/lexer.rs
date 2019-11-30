use logos::Logos;

pub struct Lexer<'src>(logos::Lexer<Token, &'src str>);

impl<'src> Lexer<'src> {
    pub fn new(src: &'src str) -> Lexer {
        Lexer(Token::lexer(src))
    }
}

impl<'src> Iterator for Lexer<'src> {
    type Item = (Token, &'src str);

    fn next(&mut self) -> Option<(Token, &'src str)> {
        if self.0.token == Token::End {
            None
        } else {
            while self.0.token == Token::Comment {
                self.0.advance();
            }

            let out = (self.0.token, self.0.slice());
            self.0.advance();
            Some(out)
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Logos, PartialEq)]
pub enum Token {
    #[end]
    End,

    #[error]
    Error,

    #[regex = "%[^\\r\\n]*"]
    Comment,

    #[token = ")"]
    ParenClose,

    #[token = "("]
    ParenOpen,

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

    #[regex = "-?[0-9]+"]
    Int,

    #[regex = "\"([^'\"\\\\]|\\\\[trn'\"\\\\])*\""]
    String,

    #[regex = "'([^'\"\\\\]|\\\\[trn'\"\\\\])*'"]
    EscapedVar,

    #[regex = "[A-Za-z_-][0-9A-Za-z_-]*"]
    Var,
}

#[derive(Clone, Copy, Debug, Eq, Logos, PartialEq)]
pub enum StringToken {
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

pub fn parse_stringish(s: &str) -> String {
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

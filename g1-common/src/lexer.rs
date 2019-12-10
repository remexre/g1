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

    #[token = ".create_atom"]
    DotCreateAtom,

    #[token = ".delete_atom"]
    DotDeleteAtom,

    #[token = ".create_name"]
    DotCreateName,

    #[token = ".delete_name"]
    DotDeleteName,

    #[token = ".upsert_name"]
    DotUpsertName,

    #[token = ".create_edge"]
    DotCreateEdge,

    #[token = ".delete_edge"]
    DotDeleteEdge,

    #[token = ".create_tag"]
    DotCreateTag,

    #[token = ".delete_tag"]
    DotDeleteTag,

    #[token = ".upsert_tag"]
    DotUpsertTag,

    #[token = ".create_blob"]
    DotCreateBlob,

    #[token = ".delete_blob"]
    DotDeleteBlob,

    #[token = ".help"]
    DotHelp,

    #[token = ".list"]
    DotList,

    #[token = ".upsert_blob"]
    DotUpsertBlob,

    #[token = ".quit"]
    DotQuit,

    #[token = ".undefine"]
    DotUndefine,

    #[token = ")"]
    ParenClose,

    #[token = "("]
    ParenOpen,

    #[token = "."]
    Period,

    #[token = ","]
    Comma,

    #[token = "/"]
    Slash,

    #[token = "_"]
    Underscore,

    #[token = "?-"]
    Query,

    #[token = ":-"]
    Turnstile,

    #[token = "!"]
    Not,

    #[regex = "\"([^'\"\\\\]|\\\\[trn'\"\\\\])*\""]
    String,

    #[regex = "[0-9]+"]
    U32,

    #[regex = "'([^'\"\\\\]|\\\\[trn'\"\\\\])*'"]
    EscapedVar,

    #[regex = "[A-Za-z_-][0-9A-Za-z_-]*"]
    Var,

    #[regex = "\\$[a-zA-Z_][a-zA-Z0-9_]*"]
    MetaVar,
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

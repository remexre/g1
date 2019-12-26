use derive_more::{From, Into};
use proc_macro2::{Delimiter, Ident, Literal, TokenStream, TokenTree};

/// A wrapper around `proc_macro2::Span`, to give a `Default` impl.
#[derive(Clone, Copy, Debug, From, Into)]
pub struct Span(proc_macro2::Span);

impl Span {
    /// Combines two `Span`s, defaulting to `Span::call_site()` if `proc_macro2::Span::join` would
    /// return `None`.
    pub fn join(self, other: Self) -> Self {
        self.0.join(other.0).map(Span).unwrap_or_else(Self::default)
    }
}

impl Default for Span {
    fn default() -> Span {
        Span(proc_macro2::Span::call_site())
    }
}

/// A `Token` flattened from a `TokenStream`.
#[derive(Clone, Debug)]
pub enum Token {
    /// A close curly brace character (`}`).
    BraceClose(Span),

    /// An open curly brace character (`{`).
    BraceOpen(Span),

    /// A close square bracket character (`]`).
    BracketClose(Span),

    /// An open square bracket character (`[`).
    BracketOpen(Span),

    /// A hole. This is technically an identifier, but LALRPOP needs it in order to match against
    /// it.
    Hole(Span),

    /// An identifier.
    Ident(Ident),

    /// A literal character (`'a'`), string (`"hello"`), number (`2.3`), etc.
    Literal(Literal),

    /// A close parenthesis character (`)`).
    ParenClose(Span),

    /// An open parenthesis character (`(`).
    ParenOpen(Span),

    /// A single punctuation character (`+`, `,`, `$`, etc.).
    Punct(char, Span),
}

impl Token {
    /// Returns the span of the token.
    pub fn span(&self) -> Span {
        match self {
            Token::BraceClose(span)
            | Token::BraceOpen(span)
            | Token::BracketClose(span)
            | Token::BracketOpen(span)
            | Token::Hole(span)
            | Token::ParenClose(span)
            | Token::ParenOpen(span)
            | Token::Punct(_, span) => *span,
            Token::Ident(ident) => ident.span().into(),
            Token::Literal(literal) => literal.span().into(),
        }
    }
}

fn append_tokenstream(tokens: &mut Vec<Token>, stream: TokenStream) {
    for tree in stream {
        append_tokentree(tokens, tree);
    }
}

fn append_tokentree(tokens: &mut Vec<Token>, tree: TokenTree) {
    match tree {
        TokenTree::Group(group) => {
            match group.delimiter() {
                Delimiter::Brace => tokens.push(Token::BraceOpen(group.span_open().into())),
                Delimiter::Bracket => tokens.push(Token::BracketOpen(group.span_open().into())),
                Delimiter::Parenthesis => tokens.push(Token::ParenOpen(group.span_open().into())),
                Delimiter::None => {}
            }

            append_tokenstream(tokens, group.stream());

            match group.delimiter() {
                Delimiter::Brace => tokens.push(Token::BraceClose(group.span_close().into())),
                Delimiter::Bracket => tokens.push(Token::BracketClose(group.span_close().into())),
                Delimiter::Parenthesis => tokens.push(Token::ParenClose(group.span_close().into())),
                Delimiter::None => {}
            }
        }
        TokenTree::Ident(ident) => tokens.push(Token::Ident(ident)),
        TokenTree::Literal(literal) => tokens.push(Token::Literal(literal)),
        TokenTree::Punct(punct) => tokens.push(Token::Punct(punct.as_char(), punct.span().into())),
    }
}

/// Converts a Rust `TokenStream` to a `Vec<Token>`.
///
/// ```
/// # use g1_common::proc_macro::token::{Token, tokenstream_to_tokens};
/// # use proc_macro2::{Ident, Punct, Spacing, Span};
/// # use quote::quote;
/// let token_stream = quote! { foo(bar, baz, _). };
/// let tokens_actual = tokenstream_to_tokens(token_stream);
/// let tokens_expected = vec![
///     Token::Ident(Ident::new("foo", Span::call_site().into())),
///     Token::ParenOpen(Span::call_site().into()),
///     Token::Ident(Ident::new("bar", Span::call_site().into())),
///     Token::Punct(',', Span::call_site().into()),
///     Token::Ident(Ident::new("baz", Span::call_site().into())),
///     Token::Punct(',', Span::call_site().into()),
///     Token::Ident(Ident::new("_", Span::call_site().into())),
///     Token::ParenClose(Span::call_site().into()),
///     Token::Punct('.', Span::call_site().into()),
/// ];
/// assert_eq!(
///     format!("{:?}", tokens_actual),
///     format!("{:?}", tokens_expected),
/// );
/// ```
pub fn tokenstream_to_tokens(stream: TokenStream) -> Vec<Token> {
    let mut tokens = Vec::new();
    append_tokenstream(&mut tokens, stream);
    tokens
}

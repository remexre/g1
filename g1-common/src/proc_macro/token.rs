//! A flat representation of tokens, to allow using LALRPOP with a `TokenTree`.

use proc_macro2::{Delimiter, Ident, Literal, TokenStream, TokenTree};
use quote::quote;
use syn::LitStr;

/// A wrapper around `proc_macro2::Span`, to give a `Default` impl.
#[derive(Clone, Copy, Debug)]
pub struct Span {
    /// This is a massive hack, to allow slipping information about whether a string is an
    /// identifier or not into `ValidatedQuery`.
    pub is_ident: bool,

    inner: proc_macro2::Span,
}

impl Span {
    /// Combines two `Span`s, defaulting to `Span::call_site()` if `proc_macro2::Span::join` would
    /// return `None`.
    pub fn join(self, other: Self) -> Self {
        self.inner
            .join(other.inner)
            .map(Span::from)
            .unwrap_or_else(Span::default)
    }
}

impl Default for Span {
    fn default() -> Span {
        Span::from(proc_macro2::Span::call_site())
    }
}

impl From<proc_macro2::Span> for Span {
    fn from(span: proc_macro2::Span) -> Span {
        Span {
            is_ident: false,
            inner: span,
        }
    }
}

impl Into<proc_macro2::Span> for Span {
    fn into(self) -> proc_macro2::Span {
        self.inner
    }
}

impl crate::validated::Span for Span {
    fn fmt_span(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        // TODO: It should be possible to slap some #[cfg()]s on this to get a pretty printing, at
        // a minimum on nightly in the context of the proc macro.
        write!(fmt, "at {:?}: ", self.inner)
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

    /// A literal character (`'a'`), number (`2.3`), etc. Notably does not include a literal string
    /// (`"hello"`).
    Literal(Literal),

    /// A literal string (`"hello"`).
    LiteralString(LitStr),

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
            Token::LiteralString(lit_str) => lit_str.span().into(),
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
        TokenTree::Literal(literal) => {
            if let Ok(lit_str) = syn::parse2(quote! { #literal }) {
                tokens.push(Token::LiteralString(lit_str))
            } else {
                tokens.push(Token::Literal(literal))
            }
        }
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

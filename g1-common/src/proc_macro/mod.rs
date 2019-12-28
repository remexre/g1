//! An implementation of the G1 query language made for the `query!` proc macro.

pub mod ir;
mod parser {
    pub use self::parser::*;
    use lalrpop_util::lalrpop_mod;

    lalrpop_mod!(parser, "/proc_macro/parser.rs");
}
pub mod token;
mod validate;

use crate::{
    proc_macro::token::Span,
    validated::{
        ValidatedClause, ValidatedPredicate, ValidatedQuery, ValidatedValue, ValidatedValueInner,
    },
};
use proc_macro2::{Ident, TokenStream};
use quote::quote;

/// The `query!` proc macro, as a function.
pub fn query_proc_macro(token_stream: TokenStream) -> Result<TokenStream, String> {
    let query = ir::Query::parse(token_stream).map_err(|e| format!("{:?}", e))?;
    let query = query.to_validated().map_err(|e| e.to_string())?;
    query.validate().map_err(|e| e.to_string())?;
    Ok(query.to_tokens())
}

fn items_to_tokens<F, T>(items: &[T], func: F) -> TokenStream
where
    F: FnMut(&T) -> TokenStream,
{
    if items.is_empty() {
        quote! { std::vec::Vec::new() }
    } else {
        let items = items
            .iter()
            .map(func)
            .fold(quote! {}, |l, r| quote! { #l #r , });
        quote! { vec![#items] }
    }
}

impl ValidatedValue<Span> {
    fn to_tokens(&self) -> TokenStream {
        let inner = match &self.inner {
            ValidatedValueInner::Str(s) if self.span.is_ident => {
                let ident = Ident::new(&s, self.span.into());
                quote! {
                    g1::common::validated::ValidatedValueInner::Str(#ident.into())
                }
            }
            ValidatedValueInner::Str(s) => {
                let s = s.as_ref();
                quote! {
                    g1::common::validated::ValidatedValueInner::Str(std::sync::Arc::from(#s))
                }
            }
            ValidatedValueInner::Var(v) => {
                quote! { g1::common::validated::ValidatedValueInner::Var(#v) }
            }
        };
        quote! {
            g1::common::validated::ValidatedValue {
                inner: #inner,
                span: (),
            }
        }
    }
}

impl ValidatedPredicate<Span> {
    fn to_tokens(&self) -> TokenStream {
        let name = self.name;
        let args = items_to_tokens(&self.args, |arg| arg.to_tokens());
        quote! {
            g1::common::validated::ValidatedPredicate {
                name: #name,
                args: #args,
                span: (),
            }
        }
    }
}

impl ValidatedClause<Span> {
    fn to_tokens(&self) -> TokenStream {
        let head = self.head.to_tokens();
        let body = items_to_tokens(&self.body, |(n, p)| {
            let p = p.to_tokens();
            quote! { (#n, #p) }
        });
        let vars = self.vars;
        quote! {
            g1::common::validated::ValidatedClause {
                head: #head,
                body: #body,
                vars: #vars,
                span: (),
            }
        }
    }
}

impl ValidatedQuery<Span> {
    fn to_tokens(&self) -> TokenStream {
        let clauses = items_to_tokens(&self.clauses, |clause| clause.to_tokens());
        let goal = self.goal.to_tokens();
        let goal_vars = self.goal_vars;
        quote! {
            g1::common::validated::ValidatedQuery {
                clauses: #clauses,
                goal: #goal,
                goal_vars: #goal_vars,
                span: (),
            }
        }
    }
}

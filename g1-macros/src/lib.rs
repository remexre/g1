//! G1 proc macros.

extern crate proc_macro;

use g1_common::{
    nameless::{NamelessClause, NamelessPredicate, NamelessQuery, NamelessValue},
    query::Query,
    SimpleError,
};
use proc_macro2::TokenStream;
use quote::quote;

#[proc_macro_hack::proc_macro_hack]
pub fn query(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut src = String::new();
    for tok in input {
        src += &tok.to_string();
    }

    match process(&src) {
        Ok(ts) => ts.into(),
        Err(err) => quote! { compile_error!(#err) }.into(),
    }
}

fn process(src: &str) -> Result<TokenStream, String> {
    let query = src.parse::<Query>().map_err(|e| e.to_string())?;
    let query = NamelessQuery::from_query(query).map_err(|e: SimpleError| e.0)?;
    let query = query_to_tokens(&query);
    Ok(quote! {
        {
            g1::lazy_static! {
                static ref QUERY: g1::NamelessQuery = #query;
            }
            let query: &'static g1::NamelessQuery = &*QUERY;
            query
        }
    })
}

fn query_to_tokens(query: &NamelessQuery) -> TokenStream {
    let clauses = vec_to_tokens(|cs| vec_to_tokens(clause_to_tokens, cs), &query.clauses);
    let goal_vars = query.goal_vars;
    let goal = predicate_to_tokens(&query.goal);
    quote! {
        g1::NamelessQuery {
            clauses: #clauses,
            goal_vars: #goal_vars,
            goal: #goal,
        }
    }
}

fn clause_to_tokens(clause: &NamelessClause) -> TokenStream {
    let vars = clause.vars;
    let head = vec_to_tokens(value_to_tokens, &clause.head);
    let body_pos = vec_to_tokens(predicate_to_tokens, &clause.body_pos);
    let body_neg = vec_to_tokens(predicate_to_tokens, &clause.body_neg);
    quote! {
        g1::NamelessClause {
            vars: #vars,
            head: #head,
            body_pos: #body_pos,
            body_neg: #body_neg,
        }
    }
}

fn predicate_to_tokens(pred: &NamelessPredicate) -> TokenStream {
    let name = pred.name;
    let args = vec_to_tokens(value_to_tokens, &pred.args);
    quote! {
        g1::NamelessPredicate {
            name: #name,
            args: #args,
        }
    }
}

fn value_to_tokens(value: &NamelessValue) -> TokenStream {
    match value {
        NamelessValue::Str(s) => {
            let s = s.to_string();
            quote!(g1::NamelessValue::Str(std::sync::Arc::from(#s)))
        }
        NamelessValue::Var(v) => quote!(g1::NamelessValue::Var(#v)),
    }
}

fn vec_to_tokens<F, T>(mut f: F, vals: &[T]) -> TokenStream
where
    F: FnMut(&T) -> TokenStream,
{
    let mut ts = quote!();
    for val in vals {
        ts.extend(f(val));
        ts.extend(quote!(,));
    }
    quote!(vec![#ts])
}

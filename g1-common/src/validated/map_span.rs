use crate::validated::{Span, ValidatedClause, ValidatedPredicate, ValidatedQuery, ValidatedValue};

impl<S: Span> ValidatedValue<S> {
    /// Changes the `Span` type inside the `ValidatedValue`.
    pub fn map_span<F: FnOnce(S) -> S2, S2: Span>(self, f: F) -> ValidatedValue<S2> {
        ValidatedValue {
            inner: self.inner,
            span: f(self.span),
        }
    }
}

impl<S: Span> ValidatedPredicate<S> {
    /// Changes the `Span` type inside the `ValidatedPredicate`.
    pub fn map_span<F: FnMut(S) -> S2, S2: Span>(self, f: &mut F) -> ValidatedPredicate<S2> {
        ValidatedPredicate {
            name: self.name,
            args: self.args.into_iter().map(|v| v.map_span(&mut *f)).collect(),
            span: f(self.span),
        }
    }
}

impl<S: Span> ValidatedClause<S> {
    /// Changes the `Span` type inside the `ValidatedClause`.
    pub fn map_span<F: FnMut(S) -> S2, S2: Span>(self, f: &mut F) -> ValidatedClause<S2> {
        ValidatedClause {
            head: self.head.map_span(f),
            body: self
                .body
                .into_iter()
                .map(|(n, p)| (n, p.map_span(f)))
                .collect(),
            vars: self.vars,
            span: f(self.span),
        }
    }
}

impl<S: Span> ValidatedQuery<S> {
    /// Changes the `Span` type inside the `ValidatedQuery`.
    pub fn map_span<F: FnMut(S) -> S2, S2: Span>(self, f: &mut F) -> ValidatedQuery<S2> {
        ValidatedQuery {
            clauses: self.clauses.into_iter().map(|c| c.map_span(f)).collect(),
            goal: self.goal.map_span(f),
            goal_vars: self.goal_vars,
            span: f(self.span),
        }
    }
}

use crate::{
    proc_macro::{
        ir::{Query, Value},
        token::Span,
    },
    validated::{
        visitors::{QueryVisitor, ValueVisitor},
        ValidatedQuery, ValidationError,
    },
};

impl Value {
    fn build_on<'a, T: ValueVisitor<'a, Span>>(&'a self, visitor: &mut T) {
        match self {
            Value::Hole(span) => visitor.visit_arg_hole(*span),
            Value::Ident(s, lit) => {
                let mut span = Span::from(lit.span());
                span.is_ident = true;
                visitor.visit_arg_string(s, span)
            }
            Value::String(s, lit) => visitor.visit_arg_string(s, lit.span().into()),
            Value::Var(s, ident) => visitor.visit_arg_var(s, ident.span().into()),
        }
    }
}

impl Query {
    /// Converts a query to a `ValidatedQuery`. Note that this does not actually perform
    /// validation.
    pub fn to_validated(self) -> Result<ValidatedQuery<Span>, ValidationError<Span>> {
        self.build_on(QueryVisitor::new())
    }

    fn build_on(
        &self,
        visitor: QueryVisitor<Span>,
    ) -> Result<ValidatedQuery<Span>, ValidationError<Span>> {
        let mut visitor = Some(visitor);
        for clause in self.clauses.iter() {
            let query_visitor = visitor.take().unwrap();

            let mut clause_visitor =
                query_visitor.visit_clause(&clause.head.name, clause.head.span, clause.span);
            for arg in clause.head.args.iter() {
                arg.build_on(&mut clause_visitor);
            }

            let mut clause_visitor = Some(clause_visitor);
            for (negated, pred) in clause.body.iter() {
                let mut pred_visitor = clause_visitor
                    .take()
                    .unwrap()
                    .visit_body(*negated, &pred.name, pred.span);
                for arg in pred.args.iter() {
                    arg.build_on(&mut pred_visitor);
                }
                clause_visitor = Some(pred_visitor.finish());
            }

            visitor = Some(clause_visitor.take().unwrap().finish());
        }

        let query_visitor = visitor.take().unwrap();
        let mut goal_visitor = query_visitor.visit_goal(&self.goal.name, self.goal.span);
        for arg in self.goal.args.iter() {
            arg.build_on(&mut goal_visitor);
        }
        goal_visitor.finish(self.span)
    }
}

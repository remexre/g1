//! Visitors used to convert frontend implementations of the query language to the `ValidatedQuery`
//! IR.

use crate::validated::{
    pool::Pool, Span, ValidatedClause, ValidatedPredicate, ValidatedQuery, ValidatedValue,
    ValidatedValueInner, ValidationError,
};
use std::{collections::HashMap, sync::Arc};
use topological_sort::TopologicalSort;

static BUILTINS: &[(&str, usize)] = &[
    ("=", 2),
    ("atom", 1),
    ("name", 3),
    ("edge", 3),
    ("tag", 3),
    ("blob", 4),
];

/// A visitor that can be visited by a value.
pub trait ValueVisitor<'a, S: Span> {
    /// Visits with a hole.
    fn visit_arg_hole(&mut self, span: S);

    /// Visits with a string literal.
    fn visit_arg_string(&mut self, string: &'a str, span: S);

    /// Visits with a variable.
    fn visit_arg_var(&mut self, var: &'a str, span: S);
}

/// A half-built clause.
#[derive(Debug)]
struct TemporaryClause<'a, S: Span> {
    head: (&'a str, Vec<ValidatedValue<S>>, S),
    body: Vec<(bool, &'a str, Vec<ValidatedValue<S>>, S)>,
    vars: u32,
    span: S,
}

/// A visitor provided to frontend IR types in order to build a `ValidatedQuery`.
#[derive(Debug)]
pub struct QueryVisitor<'a, S: Span> {
    clauses: Vec<TemporaryClause<'a, S>>,
    dependencies: TopologicalSort<(&'a str, usize)>,
    string_pool: HashMap<&'a str, Arc<str>>,
}

impl<'a, S: Span> QueryVisitor<'a, S> {
    /// Interns a string in the string pool.
    fn intern_string(&mut self, string: &'a str) -> Arc<str> {
        self.string_pool
            .entry(&string)
            .or_insert_with(|| Arc::from(string))
            .clone()
    }

    /// Creates a new `QueryVisitor`, without any clauses.
    pub fn new() -> QueryVisitor<'a, S> {
        let mut dependencies = TopologicalSort::new();
        for functor in BUILTINS.iter().copied() {
            let _ = dependencies.insert(functor);
        }
        QueryVisitor {
            clauses: Vec::new(),
            dependencies,
            string_pool: HashMap::new(),
        }
    }

    /// A method used to attach a `Clause` to a `ValidatedQuery`.
    pub fn visit_clause(self, name: &'a str, head_span: S, span: S) -> ClauseVisitor<'a, S> {
        ClauseVisitor {
            name,
            args: Vec::new(),
            body: Vec::new(),
            head_span,
            span,
            var_pool: Pool::default(),
            query_visitor: self,
        }
    }

    /// A method used to finish building the `ValidatedQuery` by filling in the `goal` field.
    pub fn visit_goal(self, name: &'a str, span: S) -> GoalVisitor<'a, S> {
        GoalVisitor {
            name,
            args: Vec::new(),
            span,
            var_pool: Pool::default(),
            query_visitor: self,
        }
    }
}

/// A visitor provided to frontend IR types in order to attach `Clause`s to a `ValidatedQuery`.
#[derive(Debug)]
pub struct ClauseVisitor<'a, S: Span> {
    name: &'a str,
    args: Vec<ValidatedValue<S>>,
    body: Vec<(bool, &'a str, Vec<ValidatedValue<S>>, S)>,
    head_span: S,
    span: S,
    var_pool: Pool<&'a str, u32>,
    query_visitor: QueryVisitor<'a, S>,
}

impl<'a, S: Span> ClauseVisitor<'a, S> {
    /// Adds a possibly-negated predicate to the body of the clause.
    pub fn visit_body(self, negated: bool, name: &'a str, span: S) -> PredicateVisitor<'a, S> {
        PredicateVisitor {
            negated,
            name,
            args: Vec::new(),
            span,
            clause_visitor: self,
        }
    }

    /// Finishes the clause.
    pub fn finish(mut self) -> QueryVisitor<'a, S> {
        for (negated, name, args, _) in self.body.iter() {
            if !negated && *name == self.name {
                continue;
            }

            let functor = (self.name, self.args.len());
            let _ = self.query_visitor.dependencies.insert(functor);
            self.query_visitor
                .dependencies
                .add_dependency((*name, args.len()), functor);
        }
        self.query_visitor.clauses.push(TemporaryClause {
            head: (self.name, self.args, self.head_span),
            body: self.body,
            vars: self.var_pool.next_index,
            span: self.span,
        });
        self.query_visitor
    }
}

impl<'a, S: Span> ValueVisitor<'a, S> for ClauseVisitor<'a, S> {
    /// Adds a hole as an argument to the head of the clause. This will always cause an error when
    /// validating the `ValidatedClause`.
    fn visit_arg_hole(&mut self, span: S) {
        let var = self.var_pool.intern_dummy();
        self.args.push(ValidatedValue {
            inner: ValidatedValueInner::Var(var),
            span,
        });
    }

    /// Adds an string literal as an argument to the head of the clause.
    fn visit_arg_string(&mut self, string: &'a str, span: S) {
        let string = self.query_visitor.intern_string(string);
        self.args.push(ValidatedValue {
            inner: ValidatedValueInner::Str(string),
            span,
        });
    }

    /// Adds a variable as an argument to the head of the clause.
    fn visit_arg_var(&mut self, var: &'a str, span: S) {
        let var = self.var_pool.intern(var);
        self.args.push(ValidatedValue {
            inner: ValidatedValueInner::Var(var),
            span,
        });
    }
}

/// A visitor proevided to frontend IR types in order to attach arguments to a predicates in the
/// body of a clause.
#[derive(Debug)]
pub struct PredicateVisitor<'a, S: Span> {
    negated: bool,
    name: &'a str,
    args: Vec<ValidatedValue<S>>,
    span: S,
    clause_visitor: ClauseVisitor<'a, S>,
}

impl<'a, S: Span> PredicateVisitor<'a, S> {
    /// Finishes adding variables.
    pub fn finish(mut self) -> ClauseVisitor<'a, S> {
        self.clause_visitor
            .body
            .push((self.negated, self.name, self.args, self.span));
        self.clause_visitor
    }
}

impl<'a, S: Span> ValueVisitor<'a, S> for PredicateVisitor<'a, S> {
    /// Adds a hole as an argument to the predicate.
    fn visit_arg_hole(&mut self, span: S) {
        let var = self.clause_visitor.var_pool.intern_dummy();
        self.args.push(ValidatedValue {
            inner: ValidatedValueInner::Var(var),
            span,
        });
    }

    /// Adds an string literal as an argument to the predicate.
    fn visit_arg_string(&mut self, string: &'a str, span: S) {
        let string = self.clause_visitor.query_visitor.intern_string(string);
        self.args.push(ValidatedValue {
            inner: ValidatedValueInner::Str(string),
            span,
        });
    }

    /// Adds a variable as an argument to the predicate.
    fn visit_arg_var(&mut self, var: &'a str, span: S) {
        let var = self.clause_visitor.var_pool.intern(var);
        self.args.push(ValidatedValue {
            inner: ValidatedValueInner::Var(var),
            span,
        });
    }
}

/// A visitor provided to frontend IR types in order to fill in the `goal` field of a
/// `ValidatedQuery`.
#[derive(Debug)]
pub struct GoalVisitor<'a, S: Span> {
    name: &'a str,
    args: Vec<ValidatedValue<S>>,
    span: S,
    var_pool: Pool<&'a str, u32>,
    query_visitor: QueryVisitor<'a, S>,
}

impl<'a, S: Span> GoalVisitor<'a, S> {
    /// Finishes visiting, returning the completed `ValidatedQuery`.
    pub fn finish(self, span: S) -> Result<ValidatedQuery<S>, ValidationError<S>> {
        let mut dependencies = self.query_visitor.dependencies;
        let mut names = BUILTINS
            .iter()
            .copied()
            .enumerate()
            .map(|(i, functor)| (functor, -(i as i32 + 1)))
            .collect::<HashMap<_, _>>();
        let mut remaining = dependencies.len();
        let mut n = 0;
        while let Some(functor) = dependencies.pop() {
            if !names.contains_key(&functor) {
                let _ = names.insert(functor, n);
                n += 1;
            }
            remaining -= 1;
        }
        if remaining != 0 {
            return Err(ValidationError::IllegalRecursion);
        }

        let mut clauses = self
            .query_visitor
            .clauses
            .into_iter()
            .map(
                |TemporaryClause {
                     head,
                     body,
                     vars,
                     span,
                 }| {
                    let functor = (head.0, head.1.len());
                    let head_name = names.get(&functor).copied().ok_or_else(|| {
                        ValidationError::NoSuchClause {
                            argn: functor.1,
                            name: functor.0.to_string(),
                            span: head.2.clone(),
                        }
                    })?;

                    let body = body
                        .into_iter()
                        .map(|(negated, name, args, span)| {
                            let functor = (name, args.len());
                            let name = names.get(&functor).copied().ok_or_else(|| {
                                ValidationError::NoSuchClause {
                                    argn: functor.1,
                                    name: functor.0.to_string(),
                                    span: span.clone(),
                                }
                            })?;

                            Ok((negated, ValidatedPredicate { name, args, span }))
                        })
                        .collect::<Result<_, _>>()?;

                    Ok(ValidatedClause {
                        head: ValidatedPredicate {
                            name: head_name,
                            args: head.1,
                            span: head.2,
                        },
                        body,
                        vars,
                        span,
                    })
                },
            )
            .collect::<Result<Vec<_>, _>>()?;
        clauses.sort_by_key(|clause| clause.head.name);

        let functor = (self.name, self.args.len());
        let goal_span = self.span.clone();
        let name = names
            .get(&functor)
            .copied()
            .ok_or_else(|| ValidationError::NoSuchClause {
                argn: functor.1,
                name: functor.0.to_string(),
                span: goal_span,
            })?;

        Ok(ValidatedQuery {
            clauses,
            goal: ValidatedPredicate {
                name,
                args: self.args,
                span: self.span,
            },
            goal_vars: self.var_pool.next_index,
            span,
        })
    }
}

impl<'a, S: Span> ValueVisitor<'a, S> for GoalVisitor<'a, S> {
    /// Adds a hole as an argument to the goal.
    fn visit_arg_hole(&mut self, span: S) {
        let var = self.var_pool.intern_dummy();
        self.args.push(ValidatedValue {
            inner: ValidatedValueInner::Var(var),
            span,
        });
    }

    /// Adds an string literal as an argument to the goal.
    fn visit_arg_string(&mut self, string: &'a str, span: S) {
        let string = self.query_visitor.intern_string(string);
        self.args.push(ValidatedValue {
            inner: ValidatedValueInner::Str(string),
            span,
        });
    }

    /// Adds a variable as an argument to the goal.
    fn visit_arg_var(&mut self, var: &'a str, span: S) {
        let var = self.var_pool.intern(var);
        self.args.push(ValidatedValue {
            inner: ValidatedValueInner::Var(var),
            span,
        });
    }
}

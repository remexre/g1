use crate::validated::{
    Span, ValidatedClause, ValidatedPredicate, ValidatedQuery, ValidatedValueInner,
};
use maplit::hashmap;
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};

/// An error during validation.
#[derive(Debug)]
pub enum ValidationError<S: Span> {
    /// An invalid number of arguments were used in a call.
    BadArgn {
        /// The number of arguments the call should have.
        expected: usize,

        /// The number of arguments the call actually had.
        found: usize,

        /// The span of the call.
        span: S,
    },

    /// Ill-formed recursion was detected.
    BadRecursion {
        /// The head of the clause doing the calling.
        caller: ValidatedPredicate<S>,

        /// The predicate being called.
        callee: ValidatedPredicate<S>,

        /// Whether the clause was being called negatively.
        negated: bool,
    },

    /// A variable with a bad index was found.
    BadVariable {
        /// The number of variables declared in the clause or goal.
        max_vars: u32,

        /// The span of the variable.
        span: S,

        /// The invalid index.
        var: u32,
    },

    /// Ill-formed recursion was detected while building the `ValidationQuery`.
    IllegalRecursion,

    /// A variable was never used in a positive position.
    NeverUsedPositively {
        /// The clause in which the variable is not used.
        clause: ValidatedClause<S>,

        /// The variable.
        var: u32,
    },

    /// No clause with the given functor existed.
    NoSuchClause {
        /// The number of arguments.
        argn: usize,

        /// The name part of the functor.
        name: i32,

        /// The span of the call.
        span: S,
    },

    /// No clause with the given functor existed while building the `ValidatedQuery`.
    NoSuchClauseBuilding {
        /// The number of arguments.
        argn: usize,

        /// The name part of the functor.
        name: String,

        /// The span of the call.
        span: S,
    },
}

impl<S: Span> Display for ValidationError<S> {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        match self {
            ValidationError::BadArgn {
                expected,
                found,
                span,
            } => {
                span.fmt_span(fmt)?;
                write!(
                    fmt,
                    "cannot call a clause that expects {} args with {}",
                    expected, found
                )
            }

            ValidationError::BadRecursion {
                caller,
                callee,
                negated: false,
            } => {
                caller.span.fmt_span(fmt)?;
                write!(fmt, "{} cannot call {}", caller, callee)
            }

            ValidationError::BadRecursion {
                caller,
                callee,
                negated: true,
            } => {
                caller.span.fmt_span(fmt)?;
                write!(fmt, "{} cannot call !{}", caller, callee)
            }

            ValidationError::BadVariable {
                max_vars,
                span,
                var,
            } => {
                span.fmt_span(fmt)?;
                write!(
                    fmt,
                    "cannot use variable #{} in a clause with {} vars",
                    var, max_vars
                )
            }

            ValidationError::IllegalRecursion => {
                // TODO: Better diagnostic...
                write!(fmt, "invalid recursion detected")
            }

            ValidationError::NeverUsedPositively { clause, var } => {
                clause.span.fmt_span(fmt)?;
                write!(fmt, "variable #{} was never used positively", var)
            }

            ValidationError::NoSuchClause { argn, name, span } => {
                span.fmt_span(fmt)?;
                write!(fmt, "no such clause {}/{}", name, argn)
            }

            ValidationError::NoSuchClauseBuilding { argn, name, span } => {
                span.fmt_span(fmt)?;
                write!(fmt, "no such clause {}/{}", name, argn)
            }
        }
    }
}

impl<S: Span> Error for ValidationError<S> {}

impl<S: Span> ValidatedQuery<S> {
    /// Validates the query.
    pub fn validate(&self) -> Result<(), ValidationError<S>> {
        // First, check each clause for positivity.
        for clause in self.clauses.iter() {
            clause.validate()?;
        }

        // Then, check each clause for stratification.
        for clause in self.clauses.iter() {
            let i = clause.head.name;
            for &(negated, ref pred) in clause.body.iter() {
                let j = pred.name;
                if negated {
                    if j >= i {
                        return Err(ValidationError::BadRecursion {
                            caller: clause.head.clone(),
                            callee: pred.clone(),
                            negated,
                        });
                    }
                } else {
                    if j > i {
                        return Err(ValidationError::BadRecursion {
                            caller: clause.head.clone(),
                            callee: pred.clone(),
                            negated,
                        });
                    }
                }
            }
        }

        // Thirdly, check the arities for every call, and that every referenced predicate exists.
        let mut arities = hashmap! {
            -1 => 2,
            -2 => 1,
            -3 => 3,
            -4 => 3,
            -5 => 3,
            -6 => 4,
        };
        for clause in self.clauses.iter() {
            let name = clause.head.name;
            let argn = clause.head.args.len();
            if arities.contains_key(&name) {
                let expected = arities[&name];
                if expected != argn {
                    return Err(ValidationError::BadArgn {
                        expected,
                        found: argn,
                        span: clause.head.span.clone(),
                    });
                }
            } else {
                let _ = arities.insert(name, argn);
            }
        }
        for clause in self.clauses.iter() {
            for (_, pred) in clause.body.iter() {
                let argn = pred.args.len();
                match arities.get(&pred.name).copied() {
                    Some(expected) => {
                        if expected != argn {
                            return Err(ValidationError::BadArgn {
                                expected,
                                found: argn,
                                span: pred.span.clone(),
                            });
                        }
                    }
                    None => {
                        return Err(ValidationError::NoSuchClause {
                            argn,
                            name: pred.name,
                            span: pred.span.clone(),
                        })
                    }
                }
            }
        }

        // Lastly, check that goal_vars is accurate.
        self.goal.for_each_var(|var, span| {
            if var < self.goal_vars {
                Ok(())
            } else {
                Err(ValidationError::BadVariable {
                    max_vars: self.goal_vars,
                    span: span.clone(),
                    var,
                })
            }
        })
    }
}

impl<S: Span> ValidatedClause<S> {
    /// Validates the clause for positivity and variable count.
    pub fn validate(&self) -> Result<(), ValidationError<S>> {
        // First, check that goal_vars is accurate for the head.
        self.head.for_each_var(|var, span| {
            if var < self.vars {
                Ok(())
            } else {
                Err(ValidationError::BadVariable {
                    max_vars: self.vars,
                    span: span.clone(),
                    var,
                })
            }
        })?;

        // Next, do the same for each body predicate.
        for (_, pred) in self.body.iter() {
            pred.for_each_var(|var, span| {
                if var < self.vars {
                    Ok(())
                } else {
                    Err(ValidationError::BadVariable {
                        max_vars: self.vars,
                        span: span.clone(),
                        var,
                    })
                }
            })?;
        }

        // Lastly, check positivity. See the blog post (G1's Query Language) for details.
        let mut used_positively = vec![false; self.vars as usize];
        let mut eq_vars = Vec::new();
        let mut neq_vars = Vec::new();
        for (negated, pred) in self.body.iter() {
            if pred.name == -1 {
                if pred.args.len() != 2 {
                    return Err(ValidationError::BadArgn {
                        expected: 2,
                        found: pred.args.len(),
                        span: pred.span.clone(),
                    });
                }
                match (&pred.args[0].inner, &pred.args[1].inner) {
                    (ValidatedValueInner::Var(l), ValidatedValueInner::Var(r)) => {
                        if *negated {
                            neq_vars.push((l, r));
                        } else {
                            eq_vars.push((l, r));
                        }
                    }
                    (ValidatedValueInner::Var(var), ValidatedValueInner::Str(_))
                    | (ValidatedValueInner::Str(_), ValidatedValueInner::Var(var)) => {
                        if !*negated {
                            used_positively[*var as usize] = true;
                        }
                    }
                    _ => {}
                }
            } else if !*negated {
                pred.for_each_var(|var, _| {
                    used_positively[var as usize] = true;
                    Ok(())
                })?;
            }
        }
        for (var, ok) in used_positively.iter().enumerate() {
            if !ok {
                return Err(ValidationError::NeverUsedPositively {
                    clause: self.clone(),
                    var: var as u32,
                });
            }
        }

        Ok(())
    }
}

impl<S: Span> ValidatedPredicate<S> {
    fn for_each_var<F>(&self, mut func: F) -> Result<(), ValidationError<S>>
    where
        F: FnMut(u32, &S) -> Result<(), ValidationError<S>>,
    {
        for arg in self.args.iter() {
            match &arg.inner {
                ValidatedValueInner::Var(n) => func(*n, &arg.span)?,
                _ => {}
            }
        }
        Ok(())
    }
}

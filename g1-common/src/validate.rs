use crate::{
    nameless::{NamelessClause, NamelessPredicate, NamelessQuery, NamelessValue},
    Error,
};
use std::convert::TryFrom;

impl NamelessQuery {
    /// Ensures the query is valid, returning an error if it is not.
    pub fn validate<E: Error>(&self) -> Result<(), E> {
        for (i, clauses) in self.clauses.iter().enumerate() {
            let i = u32::try_from(i)
                .map_err(|_| E::invalid_query("too many predicates".to_string()))?;
            for clause in clauses {
                clause.validate(i + 5)?;
            }
        }
        Ok(())
    }
}

impl NamelessClause {
    fn validate<E: Error>(&self, pred_num: u32) -> Result<(), E> {
        let mut positivities = vec![false; self.vars as usize];
        for arg in &self.head {
            arg.validate(true, &mut positivities)?
        }

        for pred in &self.body_pos {
            let max_pred = pred_num;
            pred.validate(max_pred, false, &mut positivities)?;
        }
        for pred in &self.body_neg {
            let max_pred = pred_num - 1;
            pred.validate(max_pred, true, &mut positivities)?;
        }

        for (i, positive) in positivities.into_iter().enumerate() {
            if !positive {
                return Err(E::invalid_query(format!("variable {} not positive", i)));
            }
        }

        Ok(())
    }
}

impl NamelessPredicate {
    fn validate<E: Error>(
        &self,
        max_pred: u32,
        negated: bool,
        positivities: &mut [bool],
    ) -> Result<(), E> {
        if self.name > max_pred {
            return Err(E::invalid_query("incorrect stratification".to_string()));
        }

        for arg in &self.args {
            arg.validate(negated, positivities)?
        }

        Ok(())
    }
}

impl NamelessValue {
    fn validate<E: Error>(&self, negated: bool, positivities: &mut [bool]) -> Result<(), E> {
        match self {
            NamelessValue::Var(n) => {
                let n = *n as usize;
                if n < positivities.len() {
                    positivities[n] |= !negated;
                    Ok(())
                } else {
                    Err(E::invalid_query("invalid variable number".to_string()))
                }
            }
            _ => Ok(()),
        }
    }
}

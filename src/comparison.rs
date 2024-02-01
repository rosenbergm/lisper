//! Comparison helpers for evaluation

use crate::{
    eval::{evaluate, EvalError},
    expr::Expr,
    scope::PassableScope,
};

/// Evaluates a list of expressions and compares them in windows with the provided function
pub fn compare_integers(
    args: &[Expr],
    env: &mut PassableScope,
    predicate: fn(i64, i64) -> bool,
) -> Result<Expr, EvalError> {
    let evaluated: Vec<_> = args.iter().map(|expr| evaluate(expr, env)).collect();

    // Check if there are any errors
    if let Some(Err(err)) = evaluated.iter().find(|r| r.is_err()) {
        return Err(err.clone());
    }

    // Check if all elements are numbers
    for expr in evaluated.iter() {
        match expr {
            Ok(Expr::Integer(_)) => {}
            _ => {
                return Err(EvalError::IllegalArgument(
                    "compare",
                    "All arguments must be numbers",
                ));
            }
        }
    }

    Ok(Expr::Boolean(
        evaluated
            .iter()
            .filter_map(|e| e.clone().ok())
            .collect::<Vec<Expr>>()
            .windows(2)
            .all(|w| {
                match (&w[0], &w[1]) {
                    (Expr::Integer(a), Expr::Integer(b)) => predicate(*a, *b),
                    _ => {
                        // This arm would never match, we panic if it does
                        panic!("Something unexpected happened in evaluating comparison");
                    }
                }
            }),
    ))
}

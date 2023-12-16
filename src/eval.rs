use crate::parser::Expr;

#[derive(Debug)]
pub enum EvalError {
    Generic,
}

pub fn evaluate(expr: &Expr) -> Result<Expr, EvalError> {
    match expr {
        Expr::Integer(value) => Ok(Expr::Integer(value.clone())),
        Expr::Boolean(value) => Ok(Expr::Boolean(value.clone())),
        Expr::If => todo!(),
        Expr::Op(_) => todo!(),
        Expr::Keyword(_) => todo!(),
        Expr::Symbol(_) => todo!(),
        Expr::List(list) => evaluate_list(&list),
        Expr::NoOp => Ok(Expr::NoOp),
    }
}

fn evaluate_list(list: &Vec<Expr>) -> Result<Expr, EvalError> {
    let head = list.first();

    match head {
        None => Ok(Expr::List(Vec::new())),
        Some(head_op) => match head_op {
            Expr::Op(_) => evaluate_binary_op(&list),
            Expr::If => todo!(),
            _ => {
                // FIX: We are throwing away errors here.
                let evaluated_list: Vec<_> = list
                    .iter()
                    .filter_map(|expr| match evaluate(expr) {
                        Ok(Expr::NoOp) => None,
                        Ok(expr) => Some(expr),
                        _ => None,
                    })
                    .collect();

                Ok(Expr::List(evaluated_list))
            }
        },
    }
}

/// Evaluates a list of expressions as an operations such as addition, subtraction, etc.
fn evaluate_binary_op(list: &Vec<Expr>) -> Result<Expr, EvalError> {
    if list.len() < 2 {
        return Err(EvalError::Generic);
    }

    let op = list.first().unwrap();
    let args = &list[1..];

    match op {
        Expr::Op(op) => match op.as_str() {
            "+" => {
                let mut sum = 0;

                let evaluated: Vec<_> = args.iter().map(|expr| evaluate(expr)).collect();

                if evaluated.iter().all(|expr| {
                    if let Ok(Expr::Integer(_)) = expr {
                        true
                    } else {
                        false
                    }
                }) {
                    for arg in evaluated {
                        match arg {
                            Ok(Expr::Integer(value)) => sum += value,
                            _ => return Err(EvalError::Generic),
                        }
                    }

                    Ok(Expr::Integer(sum))
                } else {
                    Err(EvalError::Generic)
                }
            }
            "-" => {
                let mut result = 0;

                let evaluated: Vec<_> = args.iter().map(|expr| evaluate(expr)).collect();

                if evaluated.iter().all(|expr| {
                    if let Ok(Expr::Integer(_)) = expr {
                        true
                    } else {
                        false
                    }
                }) {
                    for (i, arg) in evaluated.iter().enumerate() {
                        match arg {
                            Ok(Expr::Integer(value)) => {
                                if i == 0 {
                                    result = value.clone();
                                } else {
                                    result -= value.clone();
                                }
                            }
                            _ => return Err(EvalError::Generic),
                        }
                    }

                    Ok(Expr::Integer(result))
                } else {
                    Err(EvalError::Generic)
                }
            }
            _ => Err(EvalError::Generic),
        },
        _ => Err(EvalError::Generic),
    }
}

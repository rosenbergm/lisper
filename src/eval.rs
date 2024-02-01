//! Evaluation logic

use crate::comparison::compare_integers;
use crate::expr::Expr;
use crate::scope::*;

/// Defines the maximum recursion depth, meaning how many times can the `evaluate_expr` method can be called recursively.
pub static MAX_RECURSION_DEPTH: usize = 1024;

/// When an error occurs during evaluation, `EvalError` is returned
#[derive(Debug, Clone)]
pub enum EvalError {
    /// Occurs when a variable that is not defined is being accessed
    UndefinedVariable(String),

    /// Occurs when a function that is not defined is being called
    UndefinedFunction(String),

    /// An invalid argument count has been provided to a (built-in) function
    ArgumentCount(String, usize),

    /// The type of arguments passed to a function is not supported
    IllegalArgument(&'static str, &'static str),

    /// Can occur when the interpreter tries to call a function that is lexed as a built-in but hasn't been implemented yet
    Unimplemented,

    /// Execution was stopped due to reaching the recursion limit
    MaximumRecursionDepthReached(usize),

    /// Internal error that should never occur
    Unreachable,

    /// Generic error (should never be used because of its ambiguity)
    Internal,
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalError::UndefinedVariable(name) => write!(f, "Undefined variable: {}", name),
            EvalError::UndefinedFunction(name) => write!(f, "Undefined function: {}", name),
            EvalError::ArgumentCount(name, arg_count) => write!(
                f,
                "Invalid argument count for {}, {} needed",
                name, arg_count
            ),
            EvalError::Unreachable => write!(f, "Internal error (Unreachable)"),
            EvalError::Unimplemented => write!(f, "Internal error (Unimplemented)"),
            EvalError::MaximumRecursionDepthReached(max) => {
                write!(f, "Maximum recursion depth ({}) exceeded", max)
            }
            EvalError::IllegalArgument(name, msg) => write!(f, "Illegal argument in {name}: {msg}"),
            EvalError::Internal => write!(f, "Internal error"),
        }
    }
}

/// Top level function for starting the interpreter from other modules
pub fn evaluate(expr: &Expr, env: &mut PassableScope) -> Result<Expr, EvalError> {
    evaluate_expr(expr, env, 0)
}

/// Top level function for recursive evaluation of the provided expression
fn evaluate_expr(expr: &Expr, env: &mut PassableScope, depth: usize) -> Result<Expr, EvalError> {
    // Barebones recursion depth checking, only checks "stupid" recursion like
    // ```
    // fn a() {
    //   a();
    // }
    // ```
    if depth > MAX_RECURSION_DEPTH {
        return Err(EvalError::MaximumRecursionDepthReached(MAX_RECURSION_DEPTH));
    }

    match expr {
        Expr::List(list) => match list.first() {
            Some(head_op) => match head_op {
                Expr::Op(_) => evaluate_binary_op(list, env),
                Expr::If => {
                    if list.len() != 4 {
                        return Err(EvalError::ArgumentCount("if".to_string(), 4));
                    }

                    let condition = evaluate_expr(list.get(1).unwrap(), env, depth + 1)?;

                    match condition {
                        Expr::Boolean(true) => evaluate_expr(&list[2], env, depth + 1),
                        Expr::Boolean(false) => evaluate_expr(&list[3], env, depth + 1),
                        _ => Err(EvalError::IllegalArgument(
                            "if",
                            "Condition must evaluate to bool",
                        )),
                    }
                }
                Expr::Keyword(keyword) => match keyword.as_str() {
                    "def" => evaluate_def(list, env),
                    "defun" => evaluate_defun(list, env),
                    "print" => evaluate_print(list, env),
                    _ => Err(EvalError::Unimplemented),
                },
                Expr::Symbol(s) => {
                    let function = env
                        .borrow_mut()
                        .get(s)
                        .ok_or_else(|| EvalError::UndefinedVariable(s.to_string()))?;

                    match function {
                        Expr::Lambda(params, body, function_env) => {
                            let mut extended_env = Scope::extend(function_env);

                            for (i, param) in params.iter().enumerate() {
                                let value = evaluate_expr(&list[i + 1], env, depth + 1)?;

                                extended_env.borrow_mut().set(param.clone(), value);
                            }

                            evaluate_expr(&Expr::List(body), &mut extended_env, depth + 1)
                        }
                        _ => Err(EvalError::UndefinedFunction(s.clone())),
                    }
                }
                _ => {
                    let evaluated_list: Vec<_> =
                        list.iter().map(|expr| evaluate(expr, env)).collect();

                    match evaluated_list.iter().find(|r| r.is_err()) {
                        Some(Err(err)) => Err(err.clone()),
                        None => {
                            // We have an evaluated list without errors

                            Ok(Expr::List(
                                evaluated_list.iter().map(|e| e.clone().unwrap()).collect(),
                            ))
                        }
                        Some(_) => {
                            // Unreachable arm of `match`, this should never happen.

                            Err(EvalError::Unreachable)
                        }
                    }
                }
            },

            None => Ok(Expr::List(Vec::new())),
        },
        Expr::Integer(number) => Ok(Expr::Integer(*number)),
        Expr::Boolean(boolean) => Ok(Expr::Boolean(*boolean)),
        Expr::Symbol(variable) => {
            if let Some(value) = env.borrow().get(variable) {
                match value {
                    Expr::Lambda(_params, _body, _function_env) => Err(EvalError::Unimplemented),
                    other => Ok(other),
                }
            } else {
                Err(EvalError::UndefinedVariable(variable.clone()))
            }
        }
        Expr::Lambda(_params, _body, _function_env) => Ok(Expr::NoOp),
        _ => Err(EvalError::Unimplemented),
        // Expr::If => todo!(),
        // Expr::Op(_) => todo!(),
        // Expr::Keyword(_) => todo!(),
        // Expr::Symbol(_) => todo!(),
        // Expr::NoOp => todo!(),
    }
}

/// Evaluates "binary" operations. They are not really binary because they can take as many arguments as you wish.
fn evaluate_binary_op(list: &Vec<Expr>, env: &mut PassableScope) -> Result<Expr, EvalError> {
    let op = list.first().unwrap();

    if list.len() < 2 {
        let name = match op {
            Expr::Op(operator) => operator.to_string(),
            _ => "function".to_string(),
        };

        return Err(EvalError::ArgumentCount(name, 2));
    }

    let args = &list[1..];

    match op {
        Expr::Op(op) => match op.as_str() {
            "+" => {
                let mut sum = 0;

                let evaluated: Vec<_> = args.iter().map(|expr| evaluate(expr, env)).collect();

                if let Some(Err(err)) = evaluated.iter().find(|r| r.is_err()) {
                    return Err(err.clone());
                }

                for arg in evaluated {
                    match arg {
                        Ok(Expr::Integer(value)) => sum += value,
                        _ => {
                            return Err(EvalError::IllegalArgument(
                                "+",
                                "All arguments must be numbers",
                            ))
                        }
                    }
                }

                Ok(Expr::Integer(sum))
            }
            "-" => {
                let mut result = 0;

                let evaluated: Vec<_> = args.iter().map(|expr| evaluate(expr, env)).collect();

                if let Some(Err(err)) = evaluated.iter().find(|r| r.is_err()) {
                    return Err(err.clone());
                }

                for (i, arg) in evaluated.iter().enumerate() {
                    match arg {
                        Ok(Expr::Integer(value)) => {
                            if i == 0 {
                                result = *value;
                            } else {
                                result -= *value;
                            }
                        }
                        _ => {
                            return Err(EvalError::IllegalArgument(
                                "-",
                                "All arguments must be numbers",
                            ))
                        }
                    }
                }

                Ok(Expr::Integer(result))
            }
            "*" => {
                let mut result = 1;

                let evaluated: Vec<_> = args.iter().map(|expr| evaluate(expr, env)).collect();

                if let Some(Err(err)) = evaluated.iter().find(|r| r.is_err()) {
                    return Err(err.clone());
                }

                for arg in evaluated {
                    match arg {
                        Ok(Expr::Integer(value)) => result *= value,
                        _ => {
                            return Err(EvalError::IllegalArgument(
                                "*",
                                "All arguments must be numbers",
                            ))
                        }
                    }
                }

                Ok(Expr::Integer(result))
            }
            "/" => {
                let mut result = 0;

                let evaluated: Vec<_> = args.iter().map(|expr| evaluate(expr, env)).collect();

                if let Some(Err(err)) = evaluated.iter().find(|r| r.is_err()) {
                    return Err(err.clone());
                }

                for (i, arg) in evaluated.iter().enumerate() {
                    match arg {
                        Ok(Expr::Integer(value)) => {
                            if i == 0 {
                                result = *value;
                            } else {
                                result /= *value;
                            }
                        }
                        _ => {
                            return Err(EvalError::IllegalArgument(
                                "/",
                                "All arguments must be numbers",
                            ))
                        }
                    }
                }

                Ok(Expr::Integer(result))
            }
            "=" => {
                let evaluated: Vec<_> = args.iter().map(|expr| evaluate(expr, env)).collect();

                if let Some(Err(err)) = evaluated.iter().find(|r| r.is_err()) {
                    return Err(err.clone());
                }

                Ok(Expr::Boolean(
                    evaluated
                        .iter()
                        .filter_map(|e| e.clone().ok())
                        .collect::<Vec<Expr>>()
                        .windows(2)
                        .all(|w| w[0] == w[1]),
                ))
            }
            "!=" => {
                let evaluated: Vec<_> = args.iter().map(|expr| evaluate(expr, env)).collect();

                if let Some(Err(err)) = evaluated.iter().find(|r| r.is_err()) {
                    return Err(err.clone());
                }

                match evaluated.first() {
                    Some(first) => Ok(Expr::Boolean(
                        !evaluated
                            .iter()
                            .filter_map(|e| e.clone().ok())
                            .all(|e| e == first.clone().unwrap()),
                    )),
                    None => Ok(Expr::Boolean(false)),
                }
            }
            "<" => compare_integers(args, env, |a, b| a.lt(&b)),
            "<=" => compare_integers(args, env, |a, b| a.le(&b)),
            ">" => compare_integers(args, env, |a, b| a.gt(&b)),
            ">=" => compare_integers(args, env, |a, b| a.ge(&b)),
            "and" => {
                let mut result = true;

                let evaluated: Vec<_> = args.iter().map(|expr| evaluate(expr, env)).collect();

                if let Some(Err(err)) = evaluated.iter().find(|r| r.is_err()) {
                    return Err(err.clone());
                }

                for arg in evaluated {
                    match arg {
                        Ok(Expr::Boolean(value)) => result &= value,
                        _ => {
                            return Err(EvalError::IllegalArgument(
                                "and",
                                "All arguments must be booleans",
                            ))
                        }
                    }
                }

                Ok(Expr::Boolean(result))
            }
            "or" => {
                let mut result = false;

                let evaluated: Vec<_> = args.iter().map(|expr| evaluate(expr, env)).collect();

                if let Some(Err(err)) = evaluated.iter().find(|r| r.is_err()) {
                    return Err(err.clone());
                }

                for arg in evaluated {
                    match arg {
                        Ok(Expr::Boolean(value)) => result |= value,
                        _ => {
                            return Err(EvalError::IllegalArgument(
                                "and",
                                "All arguments must be booleans",
                            ))
                        }
                    }
                }

                Ok(Expr::Boolean(result))
            }
            "not" => {
                if list.len() != 2 {
                    return Err(EvalError::ArgumentCount("not".to_string(), 1));
                }

                match &list[1] {
                    Expr::Boolean(arg) => Ok(Expr::Boolean(!arg)),
                    _ => Err(EvalError::IllegalArgument(
                        "not",
                        "Argument must be a boolean",
                    )),
                }
            }
            _ => Err(EvalError::Unimplemented),
        },
        _ => Err(EvalError::Unreachable),
    }
}

/// Evaluates `def` built-in and sets the scope.
///
/// Expected Lisper syntax:
///
/// ```
/// (def x 10)
/// (def y 20)
/// (+ x y)
/// ```
fn evaluate_def(list: &Vec<Expr>, env: &mut PassableScope) -> Result<Expr, EvalError> {
    // Check argument count
    if list.len() != 3 {
        return Err(EvalError::ArgumentCount("def".to_string(), 3));
    }

    // Check if variable name is a symbol
    let variable_name = match &list[1] {
        Expr::Symbol(name) => name.clone(),

        _ => {
            return Err(EvalError::IllegalArgument(
                "def",
                "Variable name must be a symbol",
            ))
        }
    };

    // Eagerly evaluates expression that will be stored in scope
    let value = evaluate_expr(&list[2], env, 0)?;

    // Put it into the environment
    env.borrow_mut().set(variable_name, value);

    Ok(Expr::NoOp)
}

/// Evaluates `defun` built-in and sets the scope.
///
/// Expected Lisper syntax:
///
/// ```
/// (defun power (lambda (x y) (
///     if (= y 0) 1 (* x (power x (- y 1)))
/// )))
/// ```
fn evaluate_defun(list: &Vec<Expr>, env: &mut PassableScope) -> Result<Expr, EvalError> {
    // Check argument count
    if list.len() != 3 {
        return Err(EvalError::ArgumentCount("defun".to_string(), 3));
    }

    // Check if function name is a symbol
    let function_name = match &list[1] {
        Expr::Symbol(name) => name.clone(),

        _ => {
            return Err(EvalError::IllegalArgument(
                "defun",
                "Function name must be a symbol",
            ))
        }
    };

    // Get the lambda object
    let evaluated_lambda = evaluate_lambda(&list[2], env)?;

    // Put it into the environment
    env.borrow_mut().set(function_name, evaluated_lambda);

    Ok(Expr::NoOp)
}

/// Evaluates `lambda` built-in
///
/// Expected Lisper syntax:
///
/// ```(lambda (x y) (+ x y))```
fn evaluate_lambda(expr: &Expr, env: &mut PassableScope) -> Result<Expr, EvalError> {
    // The passed expression *has* to be a list.
    let list = match expr {
        Expr::List(l) => l,
        _ => return Err(EvalError::Internal),
    };

    // Check the argument count
    if list.len() != 3 {
        return Err(EvalError::ArgumentCount("lambda".to_string(), 3));
    }

    // Check if first element is the `lambda` keyword
    if list[0] != Expr::Keyword("lambda".to_string()) {
        return Err(EvalError::IllegalArgument("lambda", "Missing lambda"));
    }

    // Check if arguments are symbols
    let params = match &list[1] {
        Expr::List(l) => {
            let mut params = Vec::new();

            for param in l.iter() {
                match param {
                    Expr::Symbol(p) => params.push(p.clone()),
                    _ => {
                        return Err(EvalError::IllegalArgument(
                            "lambda",
                            "Function arguments must be symbols",
                        ))
                    }
                }
            }

            params
        }
        _ => {
            return Err(EvalError::IllegalArgument(
                "lambda",
                "Function arguments must be a list of symbols",
            ))
        }
    };

    // Check if function body is a list (is evaluable)
    let contents = match &list[2] {
        Expr::List(l) => l.clone(),
        _ => {
            return Err(EvalError::IllegalArgument(
                "lambda",
                "Function body must be an evaluable list",
            ))
        }
    };

    Ok(Expr::Lambda(params, contents, env.clone()))
}

/// Evaluates `print` built-in.
///
/// Expected Lisper syntax:
///
/// ```(print 4)```
fn evaluate_print(list: &Vec<Expr>, env: &mut PassableScope) -> Result<Expr, EvalError> {
    // Check argument count
    if list.len() != 2 {
        return Err(EvalError::ArgumentCount("print".to_string(), 1));
    }

    // Evaluates expression to be printed
    let to_print = evaluate_expr(&list[1], env, 0)?;

    // Outputs it
    println!("{to_print}");

    // Returns the evaluated code
    Ok(to_print)
}

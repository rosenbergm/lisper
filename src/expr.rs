//! Expression object used for evaluation

use crate::scope::PassableScope;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Integer(i64),
    Boolean(bool),

    If,
    Op(String),
    Keyword(String),
    Symbol(String),

    List(Vec<Expr>),

    Lambda(Vec<String>, Vec<Expr>, PassableScope),

    NoOp,
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Integer(num) => write!(f, "{num}"),
            Expr::Boolean(bool) => write!(f, "{bool}"),
            Expr::If => write!(f, "-=-"),
            Expr::Op(op) => write!(f, "Binary op {op}"),
            Expr::Keyword(kwd) => write!(f, "[{kwd}]"),
            Expr::Symbol(sym) => write!(f, "{sym}"),
            Expr::List(list) => {
                let mut output = "(".to_string();

                for (i, e) in list.iter().enumerate() {
                    output.push_str(format!("{e}").as_str());

                    if i != list.len() - 1 {
                        output.push(' ');
                    }
                }

                output.push(')');

                write!(f, "{output}")
            }
            Expr::Lambda(_, _, _) => write!(f, "-=-"),
            Expr::NoOp => write!(f, "-=-"),
        }
    }
}

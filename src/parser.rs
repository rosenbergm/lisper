//! Parser

use std::iter::Peekable;

use crate::{expr::Expr, lexer::Token};

pub enum ParseError {
    ParenExpected,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::ParenExpected => write!(f, "Opening parenthesis expected"),
        }
    }
}

pub fn parse<I>(tokens: &mut Peekable<I>) -> Result<Expr, ParseError>
where
    I: Iterator<Item = Token>,
{
    // Check if first token is a paranthesis
    if let Some(Token::OpenParen) = tokens.peek() {
        // Continue, everything is fine.
        tokens.next();
    } else {
        // Throw error
        return Err(ParseError::ParenExpected);
    }

    let mut exprs: Vec<Expr> = Vec::new();

    while let Some(token) = tokens.peek() {
        match token {
            Token::Integer(integer) => {
                exprs.push(Expr::Integer(*integer));
                tokens.next();
            }
            Token::Boolean(boolean) => {
                exprs.push(Expr::Boolean(*boolean));
                tokens.next();
            }
            Token::If => {
                exprs.push(Expr::If);
                tokens.next();
            }
            Token::BinaryOp(operator) => {
                exprs.push(Expr::Op(operator.clone()));
                tokens.next();
            }
            Token::Keyword(keyword) => {
                exprs.push(Expr::Keyword(keyword.clone()));
                tokens.next();
            }
            Token::Symbol(symbol) => {
                exprs.push(Expr::Symbol(symbol.clone()));
                tokens.next();
            }
            Token::OpenParen => {
                exprs.push(parse(tokens)?);
            }
            Token::CloseParen => {
                tokens.next();
                return Ok(Expr::List(exprs));
            }
        }
    }

    Ok(Expr::List(exprs))
}

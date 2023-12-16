use std::iter::Peekable;

use crate::lexer::Token;

#[derive(Debug)]
pub enum Expr {
    Integer(i64),
    Boolean(bool),

    If,
    Op(String),
    Keyword(String),
    Symbol(String),

    List(Vec<Expr>),

    NoOp,
}

pub fn parse<I>(tokens: &mut Peekable<I>) -> Expr
where
    I: Iterator<Item = Token>,
{
    if let Some(Token::OpenParen) = tokens.peek() {
        // Continue, everything is fine.
        tokens.next();
    } else {
        panic!("Expected open paren");
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
                exprs.push(parse(tokens));
            }
            Token::CloseParen => {
                tokens.next();
                return Expr::List(exprs);
            }
        }
    }

    return Expr::List(exprs);
}

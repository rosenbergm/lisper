use crate::{eval::evaluate, lexer::lex, parser::parse};

mod eval;
mod lexer;
mod parser;

fn main() {
    let line = "(+ (- 5 2) 3)";

    let tokens = lex(&line);

    println!("{:?}", tokens);

    let parsed = parse(&mut tokens.into_iter().peekable());

    println!("{:?}", parsed);

    let evaluated = evaluate(&parsed);

    println!("{:?}", evaluated);
}

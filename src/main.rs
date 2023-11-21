use crate::{lexer::lex, parser::parse};

mod lexer;
mod parser;

fn main() {
    let line = "(concat (1 2 3) (4 5 6 44 ahoj))";

    let tokens = lex(&line);

    println!("{:?}", tokens);

    let parsed = parse(&mut tokens.into_iter().peekable());

    println!("{:?}", parsed);
}

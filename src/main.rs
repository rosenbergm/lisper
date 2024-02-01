//! # Lisper
//!
//! A barebones LISP-family programming langauge interpreter.
//!
//! Created as a project for the Programming 1 course at MFF CUNI.
//!
//! ## Features
//!
//! ### Numbers
//!
//! Lisper can only handle integers now. It won't be complicated to implement
//! floating-point numbers, strings or other data types.
//!
//! Usage:
//! ```
//! (+ 1 2)
//! (* 1 2 3 4)
//! ```
//!
//! ### Built-in functions
//!
//! #### Arithmetic operations
//!
//! ##### `+`
//!
//! Addition on numbers
//!
//! Usage:
//! ```
//! > (+ 1 2)
//! 3
//!
//! > (+ 2 4 6)
//! 12
//!
//! > (+ 5 -5)
//! 0
//! ```
//!
//! ##### `-`
//!
//! Subtraction on numbers
//!
//! Usage:
//! ```
//! > (- 1 2)
//! -1
//!
//! > (- 5 5)
//! 0
//! ```
//!
//! ##### `*`
//!
//! Multiplication on numbers
//!
//! Usage:
//! ```
//! > (* 1 2)
//! 2
//!
//! > (* 0 3 4 6)
//! 0
//! ```
//!
//! ##### `/`
//!
//! **Integer** division on numbers
//!
//! Usage:
//! ```
//! > (/ 1 2)
//! 0
//!
//! > (/ 12 6 2)
//! 1
//! ```
//!
//! #### Logic operations and comparison
//!
//! ##### `and`
//!
//! Logic `and`
//!
//! Usage:
//! ```
//! > (and true false)
//! false
//!
//! > (and false)
//! false
//! ```
//!
//! ##### `or`
//!
//! Logic `or`
//!
//! Usage:
//! ```
//! > (or false false)
//! false
//!
//! > (or false)
//! false
//! ```
//!
//! ##### `not`
//!
//! Logical negation
//!
//! Usage:
//! ```
//! > (not true)
//! false
//! ```
//!
//! ##### `=`
//!
//! Equals
//!
//! Usage:
//! ```
//! > (= 2 2 (+ 2 0) (- 4 2))
//! true
//! ```
//!
//! ##### `!=`
//!
//! Not equals
//!
//! Usage:
//! ```
//! > (!= 2 (+ 1 2))
//! false
//! ```
//!
//! ##### `<`, `<=`, `>`, `>=`
//!
//! Comparison operators on numbers
//!
//! Usage:
//! ```
//! > (< 1 2 3)
//! true
//!
//! > (< 2 5 3)
//! false
//!
//! > (>= 8 6 (+ 3 3) 2)
//! true
//! ```
//!
//! #### Control flow
//!
//! ##### If
//!
//! Conditional execution of expression
//!
//! `(if <condition> <if-true> <if-false>)`
//!
//! Usage:
//! ```
//! > (if (= 4 (+ 2 2)) 42 0)
//! 42
//! ```
//!
//! #### Variables
//!
//! Variable definition is available using the `def` keyword.
//!
//! Usage:
//! ```
//! > (def x 10)
//! -=-
//! > (def y 20)
//! -=-
//! > (+ x y)
//! 30
//! ```
//!
//! #### Functions
//!
//! Functions can be defined using the `defun` and `lambda` keywords. The reason for the lambda keyword is to allow functions to be first-class citizens in the future.
//!
//! Usage:
//! ```
//! > (defun double (lambda (x) (* x 2)))
//! -=-
//! > (double 4)
//! 8
//! ```
//!
//! #### Printing to output
//!
//! To print something to the output, the `print` expression is available. It returns whatever it is given.
//!
//! Usage:
//! ```
//! > (defun power (lambda (x y) (if (= y 0) 1 (* x (print (power x (- y 1)))))))
//! -=-
//! > (power 2 3)
//! 1
//! 2
//! 4
//! 8
//! ```
//!

mod eval;
mod expr;
mod lexer;
mod parser;
mod repl;
mod scope;

mod comparison;

use eval::evaluate;
use lexer::lex;
use parser::parse;
use repl::run_repl;
use scope::Scope;

#[doc(hidden)]
fn main() -> rustyline::Result<()> {
    let filepath = std::env::args().collect::<Vec<String>>().get(1).cloned();

    match filepath {
        Some(path) => {
            run_from_file(path);

            rustyline::Result::Ok(())
        }
        None => run_repl(),
    }
}

/// Evaluates a file with Lisper code
fn run_from_file(path: String) {
    match std::fs::read_to_string(path) {
        Ok(content) => {
            let mut env = Scope::new().wrap();
            let tokens = lex(&content);

            match parse(&mut tokens.into_iter().peekable()) {
                Err(parser_error) => {
                    println!("PARSER ERROR: {parser_error}");
                }
                Ok(parsed) => {
                    let evaluated = evaluate(&parsed, &mut env);

                    if let Err(err) = evaluated {
                        println!("EVAL ERROR: {err}");
                    }
                }
            }
        }
        Err(_) => {
            println!("READ FILE ERROR")
        }
    }
}

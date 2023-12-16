#[derive(Debug, Clone)]
pub enum Token {
    // S-expression delimiters
    OpenParen,
    CloseParen,

    // Number types
    Integer(i64),

    // Binary types
    Boolean(bool),

    // Built-ins
    If,
    BinaryOp(String),
    Keyword(String),
    Symbol(String),
}

pub fn lex(input: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();

    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            // Lexing S-expression delimiters
            '(' => {
                chars.next();
                tokens.push(Token::OpenParen)
            }
            ')' => {
                chars.next();
                tokens.push(Token::CloseParen)
            }

            // Lex everything else
            _ => {
                let mut word = String::new();

                while let Some(&c) = chars.peek() {
                    if c.is_whitespace() {
                        chars.next();
                        break;
                    }

                    if c == '(' || c == ')' {
                        break;
                    }

                    chars.next();
                    word.push(c);
                }

                if word.is_empty() {
                    continue;
                }

                let parsed_token: Token = match word.as_str() {
                    "if" => Token::If,
                    "+" | "-" | "*" | "/" => Token::BinaryOp(word),
                    "true" => Token::Boolean(true),
                    "false" => Token::Boolean(false),
                    "print" | "len" | "concat" => Token::Keyword(word),
                    _ => {
                        if let Ok(int) = word.parse::<i64>() {
                            Token::Integer(int)
                        } else {
                            Token::Symbol(word)
                        }
                    }
                };

                tokens.push(parsed_token);
            }
        }
    }

    tokens
}

mod ast;
mod driver;
mod parser;

use crate::driver::Args;
use clap::Parser;
use std::fmt;
use std::fs;

/// Zea compiler

enum Token {
    Plus,
    Minus,
    Times,
    Divide,
    Number(u64),
}

fn tokenize(input: &String) -> Vec<Token> {
    let mut tokens: Vec<Token> = vec![];
    for c in input.chars() {
        if c.is_whitespace() {
            continue;
        }

        match c {
            '+' => tokens.push(Token::Plus),
            '-' => tokens.push(Token::Minus),
            '*' => tokens.push(Token::Times),
            '/' => tokens.push(Token::Divide),
            _ => todo!(),
        }
    }
    tokens
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Times => write!(f, "*"),
            Token::Divide => write!(f, "/"),
            Token::Number(x) => write!(f, "{x}"),
        }
    }
}

fn main() -> Result<(), std::io::Error> {
    let args = Args::parse();
    let contents = fs::read_to_string(args.filename)?;
    let tokens = tokenize(&contents);

    for token in tokens {
        println!("{token}");
    }

    Ok(())
}

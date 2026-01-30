use std::fs;

use crate::parser::{Parser, parse_decl};

mod ast;
mod lexer;
mod parser;
mod token;
mod vm;
mod interner;

fn main() {
    let source = fs::read_to_string("examples/example.op").unwrap();
    let mut parser = Parser::new(&source);
    let decl = match parse_decl(&mut parser) {
        Ok(decl) => decl,
        Err(_) => {
            println!("parse error: {:?}, expected: {:?}", parser.token, parser.expected);
            return;
        }
    };
    println!("{:#?}", decl);
}

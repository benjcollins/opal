use std::fs;

use opal_core::lexer::Lexer;

fn main() {
    let input = fs::read_to_string("examples/example.opal").unwrap();
    let lexer = Lexer::new(&input);
    for (token, span) in lexer {
        println!("{:?}", (token, span));
    }
}

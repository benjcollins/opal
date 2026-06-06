use std::fs;

use opal_core::parser::Parser;

fn main() {
    let input = fs::read_to_string("examples/example.opal").unwrap();
    let mut parser = Parser::new(&input);
    let expr = parser.parse_expr(0).unwrap();
    println!("{:#?}", expr);
}

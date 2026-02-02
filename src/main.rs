use std::fs;

use ast::ModuleItem;
use elsa::FrozenVec;
use infer::infer_fun;
use lower::lower_fun;
use parser::parse_module;
use vm::Value;

use crate::parser::Parser;

mod ast;
mod bytecode;
mod infer;
mod intern;
mod lexer;
mod lower;
mod parser;
mod scope;
mod token;
mod typed_ast;
mod vm;

fn main() {
    let source = fs::read_to_string("examples/example.op").unwrap();
    let mut parser = Parser::new(&source);
    let module = match parse_module(&mut parser) {
        Ok(module) => module,
        Err(_) => {
            println!(
                "parse error, token: {:?}, expected: {:?}",
                parser.token, parser.expected
            );
            return;
        }
    };

    println!("{:#?}", module);

    let funs = FrozenVec::new();

    for item in &module.items {
        let fun = match item {
            ModuleItem::Fun(fun) => fun,
        };
        let typed_fun = infer_fun(fun).unwrap();
        println!("{:#?}", fun);
        let compiled_fun = lower_fun(&typed_fun);
        println!("{:?}", compiled_fun.bytecode);
        let fun_ptr = Value::fun_ptr(funs.push_get(Box::new(compiled_fun)));
    }
}

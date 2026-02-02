use std::fs;

use ast::ModuleItem;
use bytecode::{BytecodeBuffer, Reg, Val};
use inferer::Inferer;
use parser::parse_module;

use crate::parser::Parser;

mod ast;
mod bytecode;
mod inferer;
mod interner;
mod lexer;
mod parser;
mod scope;
mod token;
mod typed_ast;
mod vm;

fn main() {
    // let source = fs::read_to_string("examples/example.op").unwrap();
    // let mut parser = Parser::new(&source);
    // let module = match parse_module(&mut parser) {
    //     Ok(module) => module,
    //     Err(_) => {
    //         println!(
    //             "parse error, token: {:?}, expected: {:?}",
    //             parser.token, parser.expected
    //         );
    //         return;
    //     }
    // };

    // println!("{:#?}", module);

    // let mut inferer = Inferer::new();

    // for item in &module.items {
    //     let fun_def = match item {
    //         ModuleItem::FunDef(fun_def) => fun_def,
    //     };
    //     let typed_fun_def = inferer.infer_fun_def(fun_def).unwrap();
    //     println!("{:#?}", typed_fun_def);
    // }

    let mut buf = BytecodeBuffer::new();

    buf.instr().mov(Reg(0), Val::Cst(2));
    buf.label("loop_start");
    buf.instr().beq(Val::Reg(0), Val::Cst(1), "loop_end");
    buf.instr().iadd(Reg(0), Val::Reg(0), Val::Cst(0));
    buf.instr().jmp("loop_start");
    buf.label("loop_end");

    let bytecode = buf.finish();

    println!("{:?}", bytecode);
}

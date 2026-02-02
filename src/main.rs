use std::fs;

use ast::ModuleItem;
use bytecode::{BytecodeBuffer, Reg, Val};
use infer::Inferer;
use parser::parse_module;

use crate::{parser::Parser, vm::{VM, Value}};

mod ast;
mod bytecode;
mod infer;
mod intern;
mod lexer;
mod parser;
mod scope;
mod token;
mod typed_ast;
mod vm;
mod lower;

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

    let mut inferer = Inferer::new();

    for item in &module.items {
        let fun_def = match item {
            ModuleItem::FunDef(fun_def) => fun_def,
        };
        let typed_fun_def = inferer.infer_fun_def(fun_def).unwrap();
        println!("{:#?}", typed_fun_def);
    }

    // let mut buf = BytecodeBuffer::new();

    // buf.instr().mov(Reg(0), Val::cst(2));
    // buf.label("loop_start");
    // buf.instr().beq(Val::reg(0), Val::cst(1), "loop_end");
    // buf.instr().iadd(Reg(0), Val::reg(0), Val::cst(0));
    // buf.instr().jmp("loop_start");
    // buf.label("loop_end");

    // let bytecode = buf.finish();

    // let mut vm = VM {
    //     bytecode: &bytecode,
    //     ip: 0,
    //     regs: vec![Value::int(0); 256],
    //     csts: vec![Value::int(1), Value::int(10), Value::int(5)],
    // };

    // for _ in 0..20 {
    //     vm.execute_next_instr();
    //     println!("{:?}", &vm.regs[0..5]);
    // }

    // println!("{:?}", bytecode);
}

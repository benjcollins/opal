use std::{collections::HashMap, fs};

use ast::ModuleItem;
use elsa::FrozenVec;
use infer::infer_fun;
use lower::lower_fun;
use parser::parse_module;
use vm::Value;

use crate::{infer::{FunSig, Type, resolve_type}, parser::Parser};

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

    let mut env = HashMap::new();
    for item in &module.items {
        match item {
            ModuleItem::Fun(fun) => {
                let params = fun.params.iter().map(|(_, ty)| resolve_type(ty).unwrap()).collect::<Vec<_>>();
                let returns = fun.returns.as_ref().map(|ty| resolve_type(&ty).unwrap()).unwrap_or(Type::Unit);
                env.insert(fun.name.clone(), FunSig { params, returns });
            }
        }
    }

    let funs = FrozenVec::new();

    let mut fun_ptrs_ident_value = HashMap::new();
    let mut fun_ptrs_to_patch = vec![];

    for item in &module.items {
        match item {
            ModuleItem::Fun(fun) => {
                let typed_fun = infer_fun(fun, &env).unwrap();
                println!("{:#?}", typed_fun);
                let (compiled_fun, fun_ptrs) = lower_fun(&typed_fun);
                println!("{:?}", compiled_fun.bytecode);
                let fun_ref = funs.push_get(Box::new(compiled_fun));
                fun_ptrs_to_patch.push((fun_ref, fun_ptrs));
                fun_ptrs_ident_value.insert(fun.name.clone(), Value::fun_ptr(fun_ref));
            }
        }
    }

    for (fun_ref, fun_ptrs) in fun_ptrs_to_patch {
        for (calling_fun_name, index) in fun_ptrs {
            let fun_ptr = fun_ptrs_ident_value.get(&calling_fun_name).unwrap();
            fun_ref.consts[index as usize].set(*fun_ptr);
        }
    }
}

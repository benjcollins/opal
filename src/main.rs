use std::{collections::HashMap, fs};

use ast::{Ident, ModuleItem};
use elsa::FrozenVec;
use infer::infer_fun;
use lower::lower_fun;
use parser::parse_module;
use vm::{VM, Value};

use crate::{
    infer::{FunSig, Type, resolve_type},
    parser::Parser,
};

mod ast;
mod bytecode;
mod infer;
mod instr;
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

    env.insert(Ident::new("debug_int"), FunSig::new_native(vec![Type::Int], Type::Unit));

    for item in &module.items {
        match item {
            ModuleItem::Fun(fun) => {
                let params = fun
                    .params
                    .iter()
                    .map(|(_, ty)| resolve_type(ty).unwrap())
                    .collect::<Vec<_>>();
                let returns = fun
                    .returns
                    .as_ref()
                    .map(|ty| resolve_type(&ty).unwrap())
                    .unwrap_or(Type::Unit);
                env.insert(fun.name.clone(), FunSig::new(params, returns));
            }
        }
    }

    let funs = FrozenVec::new();

    let mut name_to_fun_value = HashMap::new();
    let mut funs_to_patch = vec![];
    let mut main_fun = None;

    name_to_fun_value.insert(Ident::new("debug_int"), Value::native_fun(debug_int));

    for item in &module.items {
        match item {
            ModuleItem::Fun(fun) => {
                let typed_fun = infer_fun(fun, &env).unwrap();
                // println!("{:#?}", typed_fun);
                let (compiled_fun, fun_ptrs) = lower_fun(&typed_fun);
                println!("FUN {}:", fun.name.0);
                for instr in &compiled_fun.bytecode {
                    println!("  {}", instr);
                }
                let fun_ref = funs.push_get(Box::new(compiled_fun));
                funs_to_patch.push((fun_ref, fun_ptrs));
                name_to_fun_value.insert(fun.name.clone(), Value::fun(fun_ref));
                if fun.name == Ident::new("main") {
                    main_fun = Some(fun_ref);
                }
            }
        }
    }

    for (fun_to_patch, fun_ptrs) in funs_to_patch {
        for (target_fun_name, index) in fun_ptrs {
            let fun_value = name_to_fun_value.get(&target_fun_name).unwrap();
            fun_to_patch.consts[index as usize].set(*fun_value);
        }
    }

    let mut vm = VM {
        call_stack: Vec::new(),
        value_stack: vec![Value::unit(); 1024],
        fun: main_fun.unwrap(),
        value_stack_base: 0,
        ip: 0,
    };

    loop {
        vm.execute_next_instr();
        // println!(
        //     "{:?}",
        //     Vec::from_iter(vm.value_stack[0..10].iter().copied().map(|value| value.as_int()))
        // );
    }
}

fn debug_int<'f>(args: &[Value<'f>]) -> Value<'f> {
    println!("{}", args[0].as_int());
    Value::unit()
}

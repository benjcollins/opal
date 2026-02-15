use std::{fs, process::exit};

use runtime::{Heap, Runtime};

mod ast;
mod bytecode;
mod infer;
mod instr;
mod intern;
mod lexer;
mod lower;
mod parser;
mod runtime;
mod scope;
mod token;
mod typed_ast;
mod vm;

fn main() {
    let heap = Heap::new();
    let mut runtime = Runtime::new(&heap);

    runtime.register_native_fun(debug_int);
    runtime.register_native_fun(debug_float);
    runtime.register_native_fun(debug_bool);
    runtime.register_native_fun(assert);

    let source = fs::read_to_string("../examples/example.op").unwrap();
    runtime.compile_module(&source).unwrap();

    runtime.execute_fun("main");
}

#[opal_proc::fun]
fn debug_int(a: i64) {
    println!("{}", a);
}

#[opal_proc::fun]
fn debug_float(value: f64) {
    println!("{}", value)
}

#[opal_proc::fun]
fn debug_bool(value: bool) {
    println!("{}", value)
}

#[opal_proc::fun]
fn assert(value: bool) {
    if !value {
        println!("assertion failed!");
        exit(1);
    }
}

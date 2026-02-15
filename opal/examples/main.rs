use std::fs;

use opal::runtime::{Heap, Runtime};
use opal::vm::RuntimeError;

fn main() {
    let heap = Heap::new();
    let mut runtime = Runtime::new(&heap);

    runtime.register_native_fun(debug_int);
    runtime.register_native_fun(debug_float);
    runtime.register_native_fun(debug_bool);
    runtime.register_native_fun(assert);

    let source = fs::read_to_string("../examples/example.opal").unwrap();
    runtime.compile_module(&source).unwrap();

    runtime.execute_fun("main").unwrap();
}

#[opal_proc::fun]
fn debug_int(a: i64) -> Result<(), RuntimeError> {
    println!("{}", a);
    Ok(())
}

#[opal_proc::fun]
fn debug_float(value: f64) -> Result<(), RuntimeError> {
    println!("{}", value);
    Ok(())
}

#[opal_proc::fun]
fn debug_bool(value: bool) -> Result<(), RuntimeError> {
    println!("{}", value);
    Ok(())
}

#[opal_proc::fun]
fn assert(value: bool) -> Result<(), RuntimeError> {
    if value { Ok(()) } else { Err(RuntimeError) }
}

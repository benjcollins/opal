use std::fs;
use std::path::Path;
use std::process::ExitCode;

use opal::heap::ObjectHeap;
use opal::parser::parse_module;
use opal::runtime::{Heap, Runtime};
use opal::vm::RuntimeError;

fn main() -> ExitCode {
    let heap = Heap::new();
    let object_heap = ObjectHeap::new();
    let mut runtime = Runtime::new(&heap, &object_heap);

    runtime.register_native_fun(print_int);
    runtime.register_native_fun(print_float);
    runtime.register_native_fun(print_bool);

    let path = Path::new("../examples/example.opal");
    let source = fs::read_to_string(path).unwrap();
    let (module, errors) = parse_module(&source, Some(path));
    let module = match module {
        Some(module) if errors.is_empty() => module,
        _ => {
            for error in errors {
                println!("{}", error);
            }
            return ExitCode::FAILURE;
        }
    };
    runtime.compile_module(&module).unwrap();

    runtime.execute_fun("main").unwrap();

    ExitCode::SUCCESS
}

#[opal_proc::fun]
fn print_int(a: i64) -> Result<(), RuntimeError> {
    println!("{}", a);
    Ok(())
}

#[opal_proc::fun]
fn print_float(value: f64) -> Result<(), RuntimeError> {
    println!("{}", value);
    Ok(())
}

#[opal_proc::fun]
fn print_bool(value: bool) -> Result<(), RuntimeError> {
    println!("{}", value);
    Ok(())
}

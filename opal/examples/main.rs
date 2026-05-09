use std::fs;
use std::path::Path;
use std::process::ExitCode;
use std::sync::RwLock;

use opal::heap::Heap;
use opal::parser::parse_module;
use opal::runtime::Runtime;
use opal::value::{List, Str};
use opal::vm::RuntimeError;

fn main() -> ExitCode {
    let heap = RwLock::new(Heap::init().unwrap());
    let mut runtime = Runtime::new(&heap);

    runtime.register_native_fun(len);
    runtime.register_native_fun(assert);
    runtime.register_native_fun(print);
    runtime.register_native_fun(println);

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

    // if false {
    //     let fun = runtime.funs.get(&Ident::new("main")).unwrap();
    //     match fun {
    //         Fun::Native(_) => todo!(),
    //         Fun::Compiled(fun) => {
    //             println!("main:");
    //             for i in 0..fun.bytecode.len() {
    //                 println!("  {}", fun.bytecode[i]);
    //             }
    //             println!();
    //         }
    //     }
    // }

    runtime.execute_fun("main").unwrap();

    ExitCode::SUCCESS
}

#[opal_proc::fun]
fn print<T>(item: T) -> Result<(), RuntimeError> {
    println!("{}", item.0);
    Ok(())
}

#[opal_proc::fun]
fn println<'h>(str: Str) -> Result<(), RuntimeError> {
    println!("{}", &*str);
    Ok(())
}

#[opal_proc::fun]
fn len<T>(list: List<T>) -> Result<i64, RuntimeError> {
    Ok(list.len() as i64)
}

#[opal_proc::fun]
fn assert(value: bool) -> Result<(), RuntimeError> {
    if value { Ok(()) } else { Err(RuntimeError) }
}

use libtest_mimic::{Arguments, Failed, Trial};
use opal::{
    ast::ModuleItem,
    parser::parse_module,
    runtime::{Heap, Runtime},
    vm::RuntimeError,
};
use std::{error::Error, fs, sync::Arc};

#[opal_proc::fun]
fn assert(value: bool) -> Result<(), RuntimeError> {
    if value { Ok(()) } else { Err(RuntimeError) }
}

#[opal_proc::fun]
fn print_int(value: i64) -> Result<(), RuntimeError> {
    println!("{}", value);
    Ok(())
}

#[opal_proc::fun]
fn print_float(value: f64) -> Result<(), RuntimeError> {
    println!("{}", value);
    Ok(())
}

#[opal_proc::fun]
fn fail() -> Result<(), RuntimeError> {
    Err(RuntimeError)
}

fn run_test(name: &str, source: &str) -> Result<(), Failed> {
    let heap = Heap::new();
    let mut runtime = Runtime::new(&heap);
    runtime.register_native_fun(assert);
    runtime.register_native_fun(print_float);
    runtime.register_native_fun(print_int);
    runtime.register_native_fun(fail);
    runtime.compile_module(source).map_err(|_| "could not compile module")?;
    runtime.execute_fun(name).map_err(|_| "test execution failed")?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Arguments::from_args();

    let mut tests = vec![];

    for item in fs::read_dir("tests/tests")? {
        let item = item?;
        let path = item.path();

        if item.file_type()?.is_file() && path.extension().unwrap().to_str().unwrap() == "opal" {
            let source: Arc<str> = fs::read_to_string(path)?.into();
            let module = parse_module(&source).unwrap();

            for item in module.items {
                let ModuleItem::Fun(fun) = item;
                let fun_name = fun.name.0.as_str().to_string();
                if fun_name.starts_with("test_") {
                    let test_name = format!("{}::{}", module.name.0, fun.name.0);
                    let source = source.clone();
                    tests.push(Trial::test(test_name, move || run_test(&fun_name, &source)));
                }
            }
        }
    }

    libtest_mimic::run(&args, tests).exit();
}

use libtest_mimic::{Arguments, Failed, Trial};
use opal::{
    ast::ModuleItem,
    heap::ObjectHeap,
    parser::parse_module,
    runtime::{Heap, Runtime},
    vm::{Fun, RuntimeError},
};
use std::{
    convert::Infallible,
    error::Error,
    fs::{self, File},
    io::Write,
    path::Path,
    sync::Arc,
};

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
fn fail() -> Result<Infallible, RuntimeError> {
    Err(RuntimeError)
}

fn run_test(name: &str, source: &str, path: &Path) -> Result<(), Failed> {
    let (module, errors) = parse_module(source, Some(path));
    assert!(errors.is_empty());
    let module = module.unwrap();

    let heap = Heap::new();
    let object_heap = ObjectHeap::new();
    let mut runtime = Runtime::new(&heap, &object_heap);
    runtime.register_native_fun(assert);
    runtime.register_native_fun(print_float);
    runtime.register_native_fun(print_int);
    runtime.register_native_fun(fail);
    runtime
        .compile_module(&module)
        .map_err(|_| "could not compile module")?;

    fs::create_dir_all("tests/bytecode")?;
    let path = format!("tests/bytecode/{}.bcode", module.name.0);
    if !fs::exists(&path)? {
        let mut file = File::create(&path)?;
        let mut fun_names: Vec<_> = runtime.funs.keys().collect();
        fun_names.sort_by_key(|name| name.0.as_str());
        for name in fun_names {
            let fun = runtime.funs.get(name).unwrap();
            if let Fun::Compiled(fun) = fun {
                writeln!(file, "{}:", name.0)?;
                for instr in &fun.bytecode {
                    writeln!(file, "  {}", instr)?;
                }
                writeln!(file)?;
            }
        }
    }

    runtime.execute_fun(name).map_err(|_| "test execution failed")?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Arguments::from_args();

    let mut tests = vec![];

    fs::remove_dir_all("tests/bytecode")?;

    for item in fs::read_dir("tests/tests")? {
        let item = item?;
        let path = item.path();

        if item.file_type()?.is_file() && path.extension().unwrap().to_str().unwrap() == "opal" {
            let source = Arc::new(fs::read_to_string(&path)?);
            let (module, errors) = parse_module(&source, Some(&path));
            let module = match module {
                Some(module) if errors.is_empty() => module,
                _ => {
                    for error in errors {
                        println!("{}", error);
                    }
                    panic!("failed to parse module");
                }
            };

            for item in module.items {
                let ModuleItem::Fun(fun) = item;
                let fun_name = fun.name.0.as_str().to_string();
                if fun_name.starts_with("test_") {
                    let test_name = format!("{}::{}", module.name.0, fun.name.0);
                    let source = source.clone();
                    let path = path.clone();
                    tests.push(Trial::test(test_name, move || run_test(&fun_name, &source, &path)));
                }
            }
        }
    }

    libtest_mimic::run(&args, tests).exit();
}

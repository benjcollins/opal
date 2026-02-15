use std::collections::HashMap;

use elsa::FrozenVec;

use crate::{
    ast::{Ident, ModuleItem},
    infer::{FunSig, Type, infer_fun, resolve_type},
    lower::{CompiledFun, lower_fun},
    parser::{Parser, parse_module},
    vm::{Fun, VM, Value},
};

#[derive(Debug, Clone, Copy)]
pub struct NativeFun {
    pub name: &'static str,
    pub params: &'static [Type],
    pub returns: Type,
    pub fun: for<'h> fn(&[Value<'h>]) -> Value<'h>,
}

impl NativeFun {
    pub fn sig(&self) -> FunSig {
        FunSig {
            params: self.params.to_vec(),
            returns: self.returns,
        }
    }
}

pub struct Heap<'h> {
    funs: FrozenVec<Box<CompiledFun<'h>>>,
}

pub struct Runtime<'h> {
    heap: &'h Heap<'h>,
    env: HashMap<Ident, FunSig>,
    env2: HashMap<Ident, Fun<'h>>,
}

impl<'h> Heap<'h> {
    pub fn new() -> Heap<'h> {
        Heap { funs: FrozenVec::new() }
    }
}

impl<'h> Runtime<'h> {
    pub fn new(heap: &'h Heap<'h>) -> Runtime<'h> {
        Runtime {
            heap,
            env: HashMap::new(),
            env2: HashMap::new(),
        }
    }
    pub fn register_native_fun(&mut self, fun: NativeFun) {
        self.env.insert(Ident::new(fun.name), fun.sig());
        self.env2.insert(Ident::new(fun.name), Fun::Native(fun.fun));
    }
    pub fn compile_module(&mut self, source: &str) -> Result<(), ()> {
        let mut parser = Parser::new(&source);
        let module = parse_module(&mut parser)?;

        for item in &module.items {
            match item {
                ModuleItem::Fun(fun) => {
                    let params = fun.params.iter().map(|(_, ty)| resolve_type(ty).unwrap()).collect::<Vec<_>>();
                    let returns = fun.returns.as_ref().map(|ty| resolve_type(ty).unwrap()).unwrap_or(Type::Unit);
                    self.env.insert(fun.name.clone(), FunSig::new(params, returns));
                }
            }
        }

        let mut funs_to_patch = vec![];

        for item in &module.items {
            match item {
                ModuleItem::Fun(fun) => {
                    let typed_fun = infer_fun(fun, &self.env).unwrap();
                    let (compiled_fun, fun_ptrs) = lower_fun(&typed_fun);
                    let fun_ref = self.heap.funs.push_get(Box::new(compiled_fun));
                    funs_to_patch.push((fun_ref, fun_ptrs));
                    self.env2.insert(fun.name.clone(), Fun::Compiled(fun_ref));
                }
            }
        }

        for (fun_to_patch, fun_ptrs) in funs_to_patch {
            for (target_fun_name, index) in fun_ptrs {
                let fun = self.env2.get(&target_fun_name).unwrap();
                fun_to_patch.consts[index as usize].set(Value::from_fun(*fun));
            }
        }

        Ok(())
    }
    pub fn execute_fun(&mut self, name: &str) {
        let Fun::Compiled(fun) = self.env2.get(&Ident::new(name)).unwrap() else {
            panic!()
        };

        let mut vm = VM {
            call_stack: Vec::new(),
            value_stack: vec![Value::from_unit(()); 1024],
            fun,
            value_stack_base: 0,
            ip: 0,
            running: true,
        };

        println!("--- RUNNING ---");

        while vm.running {
            vm.execute_next_instr();
        }
    }
}

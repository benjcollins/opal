use std::{collections::HashMap, sync::RwLock};

use crate::{
    ast::{Ident, Module, ModuleItem},
    heap::{function::Function, handle::Handle, stack::StackGuard, Heap},
    infer::infer_fun,
    lower::lower_fun,
    ty::{BorrowedType, FunSig, Type},
    value::Value,
    vm::{ControlFlow, RuntimeError, VM},
};

pub type HostFun = for<'h> fn(value_stack: &StackGuard<'h>) -> Result<Value<'h>, RuntimeError>;

#[derive(Debug, Clone, Copy)]
pub struct TypedHostFun {
    pub name: &'static str,
    pub generics: &'static [&'static str],
    pub params: &'static [BorrowedType<'static>],
    pub returns: BorrowedType<'static>,
    pub fun: HostFun,
}

impl TypedHostFun {
    pub fn sig(&self) -> FunSig {
        FunSig {
            generics: self.generics.iter().map(|name| Ident::new(name)).collect(),
            params: Vec::from_iter(self.params.iter().map(|ty| ty.into())),
            returns: Box::new((&self.returns).into()),
        }
    }
}

pub struct Runtime<'h> {
    heap: &'h RwLock<Heap>,
    fun_sigs: HashMap<Ident, FunSig>,
    fun_handles: HashMap<Ident, Fun>,
}

#[derive(Debug, Clone)]
pub enum Fun {
    Host(HostFun),
    VM(Handle<Function>),
}

impl<'h> Runtime<'h> {
    pub fn new(heap: &'h RwLock<Heap>) -> Runtime<'h> {
        Runtime {
            heap,
            fun_sigs: HashMap::new(),
            fun_handles: HashMap::new(),
        }
    }
    pub fn register_native_fun(&mut self, fun: TypedHostFun) {
        self.fun_sigs.insert(Ident::new(fun.name), fun.sig());
        self.fun_handles.insert(Ident::new(fun.name), Fun::Host(fun.fun));
    }
    pub fn compile_module(&mut self, module: &Module) -> Result<(), ()> {
        for item in &module.items {
            match item {
                ModuleItem::Fun(fun) => {
                    let params = fun
                        .params
                        .iter()
                        .map(|(_, ty)| ty.try_into().unwrap())
                        .collect::<Vec<_>>();
                    let returns = fun
                        .returns
                        .as_ref()
                        .map(|ty| ty.try_into().unwrap())
                        .unwrap_or(Type::Unit);
                    self.fun_sigs
                        .insert(fun.name.clone(), FunSig::new(vec![], params, returns));
                }
            }
        }

        let mut funs_to_patch = vec![];
        let heap = self.heap.read().unwrap();

        for item in &module.items {
            match item {
                ModuleItem::Fun(fun) => {
                    let typed_fun = infer_fun(fun, &self.fun_sigs).unwrap();
                    let (fun_handle, fun_ptrs) = lower_fun(&typed_fun, &heap);
                    self.fun_handles.insert(fun.name.clone(), Fun::VM(fun_handle.clone()));
                    funs_to_patch.push((fun_handle, fun_ptrs));
                }
            }
        }

        for (fun_to_patch, fun_ptrs) in funs_to_patch {
            for (target_fun_name, index) in fun_ptrs {
                let fun = self.fun_handles.get(&target_fun_name).unwrap();
                let value = match fun {
                    Fun::Host(fun) => Value::HostFun(*fun),
                    Fun::VM(fun) => Value::VMFun(fun.to_object(&heap)),
                };
                fun_to_patch.to_object(&heap).set_constant(index as usize, value);
            }
        }

        Ok(())
    }
    pub fn execute_fun(&self, name: &str) -> Result<(), RuntimeError> {
        let Fun::VM(fun_handle) = self.fun_handles.get(&Ident::new(name)).cloned().unwrap() else {
            panic!()
        };

        let stack = {
            let heap = self.heap.read().unwrap();
            let fun_object = fun_handle.to_object(&heap);
            heap.alloc_stack(fun_object).to_handle()
        };

        let mut cf = ControlFlow::Continue;
        'outer: while cf.is_continue() {
            let mut heap = self.heap.write().unwrap();
            let mut vm = VM {
                stack: stack.to_object(&heap).lock(),
                heap: &heap,
            };

            for _ in 0..1000 {
                cf = vm.execute_next_instr()?;
                if cf.is_break() {
                    break 'outer;
                }
            }

            drop(vm);

            heap.collect();
        }

        Ok(())
    }
}

use std::{cell::RefCell, collections::HashMap};

use elsa::FrozenVec;

use crate::{
    ast::{Ident, Module, ModuleItem},
    heap::{Array, Heap, Object},
    infer::infer_fun,
    lower::{CompiledFun, lower_fun},
    ty::{BorrowedType, FunSig, Type},
    value::{StaticValue, Value},
    vm::{ControlFlow, RuntimeError, VM},
};

pub type NativeFun =
    for<'m, 's> fn(value_stack: Object<'m, Array<'s>>, value_stack_base: usize) -> Result<Value<'m, 's>, RuntimeError>;

#[derive(Debug, Clone, Copy)]
pub struct TypedNativeFun {
    pub name: &'static str,
    pub generics: &'static [&'static str],
    pub params: &'static [BorrowedType<'static>],
    pub returns: BorrowedType<'static>,
    pub fun: NativeFun,
}

impl TypedNativeFun {
    pub fn sig(&self) -> FunSig {
        FunSig {
            generics: self.generics.iter().map(|name| Ident::new(name)).collect(),
            params: Vec::from_iter(self.params.iter().map(|ty| ty.into())),
            returns: Box::new((&self.returns).into()),
        }
    }
}

pub struct Runtime<'s> {
    compiled_funs: FrozenVec<Box<CompiledFun<'s>>>,
    heap: Heap,
    fun_sigs: RefCell<HashMap<Ident, FunSig>>,
    pub funs: RefCell<HashMap<Ident, Fun<'s>>>,
}

#[derive(Debug, Clone, Copy)]
pub enum Fun<'s> {
    Native(NativeFun),
    Compiled(&'s CompiledFun<'s>),
}

impl<'s> Runtime<'s> {
    pub fn new(heap: Heap) -> Runtime<'s> {
        Runtime {
            heap,
            fun_sigs: RefCell::new(HashMap::new()),
            funs: RefCell::new(HashMap::new()),
            compiled_funs: FrozenVec::new(),
        }
    }
    pub fn register_native_fun(&mut self, fun: TypedNativeFun) {
        self.fun_sigs.borrow_mut().insert(Ident::new(fun.name), fun.sig());
        self.funs
            .borrow_mut()
            .insert(Ident::new(fun.name), Fun::Native(fun.fun));
    }
    pub fn compile_module(&'s self, module: &Module) -> Result<(), ()> {
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
                        .borrow_mut()
                        .insert(fun.name.clone(), FunSig::new(vec![], params, returns));
                }
            }
        }

        let mut funs_to_patch = vec![];

        for item in &module.items {
            match item {
                ModuleItem::Fun(fun) => {
                    let typed_fun = infer_fun(fun, &self.fun_sigs.borrow()).unwrap();
                    let (compiled_fun, fun_ptrs) = lower_fun(&typed_fun);

                    let compiled_fun = self.compiled_funs.push_get(Box::new(compiled_fun));

                    funs_to_patch.push((compiled_fun, fun_ptrs));
                    self.funs
                        .borrow_mut()
                        .insert(fun.name.clone(), Fun::Compiled(compiled_fun));
                }
            }
        }

        for (fun_to_patch, fun_ptrs) in funs_to_patch {
            for (target_fun_name, index) in fun_ptrs {
                let funs = self.funs.borrow();
                let fun = funs.get(&target_fun_name).unwrap();
                let value = match *fun {
                    Fun::Native(fun) => StaticValue::NativeFun(fun),
                    Fun::Compiled(fun) => StaticValue::CompiledFun(fun),
                };
                fun_to_patch.consts[index as usize].set(value);
            }
        }

        Ok(())
    }
    pub fn execute_fun(&'s self, name: &str) -> Result<(), RuntimeError> {
        let fun = {
            let funs = self.funs.borrow();
            funs.get(&Ident::new(name)).copied()
        };

        let Fun::Compiled(fun) = fun.unwrap() else { panic!() };

        let mutator = self.heap.mutator();
        let value_stack = mutator.alloc_array(1024);

        let mut vm = VM {
            call_stack: Vec::with_capacity(256),
            value_stack,
            mutator: &mutator,
            ip: 0,
            value_stack_frame: 0,
            fun,
        };

        let mut cf = ControlFlow::Continue;
        while cf.is_continue() {
            cf = vm.execute_next_instr()?;
        }

        Ok(())
    }
}

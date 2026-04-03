use std::{cell::RefCell, collections::HashMap};

use elsa::FrozenVec;

use crate::{
    ast::{Ident, Module, ModuleItem},
    heap::{Heap, mutator::Mutator, stack::Stack},
    infer::infer_fun,
    lower::{CompiledFun, lower_fun},
    ty::{BorrowedType, FunSig, Type},
    value::Value,
    vm::{ControlFlow, RuntimeError, VM},
};

pub type NativeFun = for<'m, 's, 'h> fn(
    value_stack: &Stack<'h, 's>,
    mutator: &'m Mutator<'h>,
    value_stack_base: usize,
) -> Result<Value<'m, 's>, RuntimeError>;

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

pub struct Runtime<'s, 'h> {
    compiled_funs: FrozenVec<Box<CompiledFun<'s>>>,
    heap: &'h Heap,
    fun_sigs: RefCell<HashMap<Ident, FunSig>>,
    pub funs: RefCell<HashMap<Ident, Fun<'s>>>,
}

#[derive(Debug, Clone, Copy)]
pub enum Fun<'s> {
    Native(NativeFun),
    Compiled(&'s CompiledFun<'s>),
}

impl<'s, 'h> Runtime<'s, 'h> {
    pub fn new(heap: &'h Heap) -> Runtime<'s, 'h> {
        Runtime {
            heap,
            fun_sigs: RefCell::new(HashMap::new()),
            funs: RefCell::new(HashMap::new()),
            compiled_funs: FrozenVec::new(),
        }
    }
    pub fn register_native_fun(&self, fun: TypedNativeFun) {
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
                    Fun::Native(fun) => Value::host_fun(fun),
                    Fun::Compiled(fun) => Value::fun(fun),
                };
                fun_to_patch.consts[index as usize].set(value.try_into().unwrap());
            }
        }

        Ok(())
    }
    pub fn execute_fun(&'s self, name: &str) -> Result<(), RuntimeError> {
        let fun = {
            let funs = self.funs.borrow();
            funs.get(&Ident::new(name)).copied()
        };

        let Fun::Compiled(mut fun) = fun.unwrap() else { panic!() };

        let mut call_stack = vec![];
        let mut value_stack = self.heap.new_stack();
        let mut ip = 0;
        let mut value_stack_frame = 0;

        let mut cf = ControlFlow::Continue;
        while cf.is_continue() {
            let mutator = self.heap.new_mutator();

            let mut vm = VM {
                call_stack,
                value_stack,
                mutator: &mutator,
                ip,
                value_stack_frame,
                fun,
            };

            for _ in 0..1000 {
                if cf.is_break() {
                    break;
                }
                cf = vm.execute_next_instr()?;
            }

            call_stack = vm.call_stack;
            value_stack = vm.value_stack;
            ip = vm.ip;
            value_stack_frame = vm.value_stack_frame;
            fun = vm.fun;

            mutator.finish();
            self.heap.collect_garabge();
        }

        Ok(())
    }
}

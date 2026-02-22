use std::collections::HashMap;

use elsa::FrozenVec;

use crate::{
    ast::{Ident, Module, ModuleItem},
    heap::ObjectHeap,
    infer::infer_fun,
    lower::{CompiledFun, lower_fun},
    ty::{BorrowedType, FunSig, Type},
    value::Value,
    vm::{ControlFlow, Fun, RuntimeError, VM},
};

pub type NativeFunSig = for<'h> fn(&[Value<'h>]) -> Result<Value<'h>, RuntimeError>;

#[derive(Debug, Clone, Copy)]
pub struct NativeFun {
    pub name: &'static str,
    pub params: &'static [BorrowedType<'static>],
    pub returns: BorrowedType<'static>,
    pub fun: NativeFunSig,
}

impl NativeFun {
    pub fn sig(&self) -> FunSig {
        FunSig {
            params: Vec::from_iter(self.params.into_iter().map(|ty| ty.into())),
            returns: Box::new((&self.returns).into()),
        }
    }
}

pub struct Heap<'h> {
    funs: FrozenVec<Box<CompiledFun<'h>>>,
}

pub struct Runtime<'h> {
    fun_heap: &'h Heap<'h>,
    object_heap: &'h ObjectHeap,
    fun_sigs: HashMap<Ident, FunSig>,
    pub funs: HashMap<Ident, Fun<'h>>,
}

impl Default for Heap<'_> {
    fn default() -> Self {
        Heap::new()
    }
}

impl<'h> Heap<'h> {
    pub fn new() -> Heap<'h> {
        Heap { funs: FrozenVec::new() }
    }
}

impl<'h> Runtime<'h> {
    pub fn new(heap: &'h Heap<'h>, object_heap: &'h ObjectHeap) -> Runtime<'h> {
        Runtime {
            fun_heap: heap,
            object_heap,
            fun_sigs: HashMap::new(),
            funs: HashMap::new(),
        }
    }
    pub fn register_native_fun(&mut self, fun: NativeFun) {
        self.fun_sigs.insert(Ident::new(fun.name), fun.sig());
        self.funs.insert(Ident::new(fun.name), Fun::Native(fun.fun));
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
                    self.fun_sigs.insert(fun.name.clone(), FunSig::new(params, returns));
                }
            }
        }

        let mut funs_to_patch = vec![];

        for item in &module.items {
            match item {
                ModuleItem::Fun(fun) => {
                    let typed_fun = infer_fun(fun, &self.fun_sigs).unwrap();
                    let (compiled_fun, fun_ptrs) = lower_fun(&typed_fun);
                    let fun_ref = self.fun_heap.funs.push_get(Box::new(compiled_fun));
                    funs_to_patch.push((fun_ref, fun_ptrs));
                    self.funs.insert(fun.name.clone(), Fun::Compiled(fun_ref));
                }
            }
        }

        for (fun_to_patch, fun_ptrs) in funs_to_patch {
            for (target_fun_name, index) in fun_ptrs {
                let fun = self.funs.get(&target_fun_name).unwrap();
                fun_to_patch.consts[index as usize].set(Value::from_fun(*fun));
            }
        }

        Ok(())
    }
    pub fn execute_fun(&mut self, name: &str) -> Result<(), RuntimeError> {
        let Fun::Compiled(fun) = self.funs.get(&Ident::new(name)).unwrap() else {
            panic!()
        };

        let mut vm = VM {
            call_stack: Vec::new(),
            value_stack: vec![Value::from_unit(()); 1024],
            fun,
            value_stack_base: 0,
            ip: 0,
            heap: self.object_heap,
        };

        let mut cf = ControlFlow::Continue;
        while cf.is_continue() {
            cf = vm.execute_next_instr()?;
        }

        Ok(())
    }
}

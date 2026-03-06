use std::collections::HashMap;

use crate::{
    ast::{Ident, Module, ModuleItem},
    heap::{Bytecode, Heap, Object, RootedObject, Values},
    infer::infer_fun,
    lower::lower_fun,
    ty::{BorrowedType, FunSig, Type},
    value::Value,
    vm::{ControlFlow, RuntimeError, VM},
};

pub type NativeFun =
    for<'a> fn(value_stack: Object<'a, Values>, value_stack_base: usize) -> Result<Value<'a>, RuntimeError>;

#[derive(Debug, Clone, Copy)]
pub struct TypedNativeFun {
    pub name: &'static str,
    pub params: &'static [BorrowedType<'static>],
    pub returns: BorrowedType<'static>,
    pub fun: NativeFun,
}

impl TypedNativeFun {
    pub fn sig(&self) -> FunSig {
        FunSig {
            params: Vec::from_iter(self.params.iter().map(|ty| ty.into())),
            returns: Box::new((&self.returns).into()),
        }
    }
}

pub struct Runtime<'h> {
    heap: &'h Heap,
    fun_sigs: HashMap<Ident, FunSig>,
    pub funs: HashMap<Ident, Fun<'h>>,
}

pub enum Fun<'h> {
    Native(NativeFun),
    Compiled(RootedObject<'h, Values>),
}

impl<'h> Runtime<'h> {
    pub fn new(heap: &'h Heap) -> Runtime<'h> {
        Runtime {
            heap,
            fun_sigs: HashMap::new(),
            funs: HashMap::new(),
        }
    }
    pub fn register_native_fun(&mut self, fun: TypedNativeFun) {
        self.fun_sigs.insert(Ident::new(fun.name), fun.sig());
        self.funs.insert(Ident::new(fun.name), Fun::Native(fun.fun));
    }
    pub fn compile_module(&mut self, module: &Module) -> Result<(), ()> {
        let heap_lock = self.heap.lock();

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

        let mut consts_to_patch = vec![];

        for item in &module.items {
            match item {
                ModuleItem::Fun(fun) => {
                    let typed_fun = infer_fun(fun, &self.fun_sigs).unwrap();
                    let (compiled_fun, fun_ptrs) = lower_fun(&typed_fun);

                    let bytecode = heap_lock.alloc::<Bytecode>(compiled_fun.bytecode.len());
                    for i in 0..compiled_fun.bytecode.len() {
                        bytecode.set(i, compiled_fun.bytecode[i]);
                    }

                    let consts = heap_lock.alloc::<Values>(compiled_fun.consts.len());
                    for i in 0..compiled_fun.consts.len() {
                        consts.set(i, compiled_fun.consts[i]);
                    }

                    let fun_object = heap_lock.alloc::<Values>(2);
                    fun_object.set(0, Value::from_object(bytecode));
                    fun_object.set(1, Value::from_object(consts));
                    let rooted_fun_object = heap_lock.root(fun_object);

                    consts_to_patch.push((consts, fun_ptrs));
                    self.funs.insert(fun.name.clone(), Fun::Compiled(rooted_fun_object));
                }
            }
        }

        for (consts_to_patch, fun_ptrs) in consts_to_patch {
            for (target_fun_name, index) in fun_ptrs {
                let fun = self.funs.get(&target_fun_name).unwrap();
                let value = match fun {
                    Fun::Native(fun) => Value::from_native_fun(*fun),
                    Fun::Compiled(fun) => Value::from_object(heap_lock.get_ref_from_root(fun)),
                };
                consts_to_patch.set(index as usize, value);
            }
        }

        // print bytecode
        // let mut fun_names: Vec<_> = self.funs.keys().collect();
        // fun_names.sort_by_key(|name| name.0.as_str());
        // for name in fun_names {
        //     let fun = self.funs.get(name).unwrap();
        //     if let Fun::Compiled(fun) = fun {
        //         let bytecode = unsafe { heap_lock.get_ref_from_root(fun).get(0).as_object::<Bytecode>() };
        //         println!("{}:", name.0);
        //         for i in 0..bytecode.len() {
        //             let instr = bytecode.get(i);
        //             println!("  {}", instr);
        //         }
        //         println!();
        //     }
        // }

        Ok(())
    }
    pub fn execute_fun(&mut self, name: &str) -> Result<(), RuntimeError> {
        let Fun::Compiled(fun) = self.funs.get(&Ident::new(name)).unwrap() else {
            panic!()
        };

        let heap_lock = self.heap.lock();

        let fun_object = heap_lock.get_ref_from_root(fun);

        let mut vm = VM {
            call_stack: heap_lock.alloc(256),
            value_stack: heap_lock.alloc(1024),
            bytecode: unsafe { fun_object.get(0).as_object::<Bytecode>() },
            consts: unsafe { fun_object.get(1).as_object::<Values>() },
            heap: &heap_lock,
            ip: 0,
            value_stack_frame: 0,
            call_stack_top: 0,
        };

        let mut cf = ControlFlow::Continue;
        while cf.is_continue() {
            cf = vm.execute_next_instr()?;
        }

        Ok(())
    }
}

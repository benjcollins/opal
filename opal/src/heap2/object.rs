use std::{
    iter::repeat_n,
    ops::{Deref, DerefMut},
    ptr,
    sync::{
        Mutex, MutexGuard,
        atomic::{AtomicBool, AtomicPtr, AtomicU32, Ordering},
    },
};

use crate::{
    heap2::handle::Handle,
    instr::Instr,
    value::{Value, ValueTag},
};

#[derive(Debug)]
pub struct ObjectHeader {
    pub next: AtomicPtr<ObjectHeader>,
    pub handle_count: AtomicU32,
    pub tag: ObjectTag,
    pub marked: AtomicBool,
}

#[derive(Debug)]
#[repr(C)]
pub struct Object<T> {
    pub header: ObjectHeader,
    pub body: T,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectTag {
    Function,
    Stack,
    Array,
}

#[derive(Debug)]
pub struct Function {
    pub bytecode: Box<[Instr]>,
    pub constants_tag: Box<[ValueTag]>,
    pub constants_data: Box<[AtomicPtr<()>]>,
    pub frame_size: u8,
}

pub struct Stack(Mutex<StackInner>);

pub struct StackInner {
    pub call_stack: Vec<CallFrame>,
    pub value_stack_tag: Vec<ValueTag>,
    pub value_stack_data: Vec<*mut ()>,
    pub function: *mut Object<Function>,
    pub base_ptr: usize,
    pub instr_ptr: usize,
}

pub struct StackGuard<'h>(MutexGuard<'h, StackInner>);

#[derive(Debug)]
pub struct List {
    pub value_tag: ValueTag,
    pub elements: Vec<AtomicPtr<()>>,
}

pub struct CallFrame {
    pub instr_ptr: usize,
    pub function: *mut Object<Function>,
}

impl<T> Deref for Object<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.body
    }
}

impl<T> DerefMut for Object<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.body
    }
}

impl<T> Object<T> {
    pub fn to_handle(&self) -> Handle<T> {
        Handle::new(self)
    }
}

impl List {
    pub fn len(&self) -> usize {
        self.elements.len()
    }
    pub fn get_element(&self, index: usize) -> Value<'_> {
        unsafe { Value::from_raw_parts(self.value_tag, self.elements[index].load(Ordering::Relaxed)) }
    }
    pub fn set_element(&self, index: usize, value: Value<'_>) {
        let (tag, data) = value.to_raw_parts();
        if tag != self.value_tag {
            panic!("cannot set invalid type");
        }
        self.elements[index].store(data, Ordering::Relaxed);
    }
}

impl Function {
    pub fn get_constant<'s>(&self, index: usize) -> Value<'_> {
        unsafe {
            Value::from_raw_parts(
                self.constants_tag[index],
                self.constants_data[index].load(Ordering::Relaxed),
            )
        }
    }
    pub fn set_constant(&self, index: usize, value: Value) {
        let (tag, data) = value.to_raw_parts();
        if tag != self.constants_tag[index] {
            panic!("cannot set invalid type");
        }
        self.constants_data[index].store(data, Ordering::Relaxed);
    }
}

impl Stack {
    pub fn lock(&self) -> StackGuard<'_> {
        StackGuard(self.0.lock().unwrap())
    }
}

impl<'h> StackGuard<'h> {
    pub fn get_stack_value(&self, index: usize) -> Value<'h> {
        unsafe { Value::from_raw_parts(self.0.value_stack_tag[index], self.0.value_stack_data[index]) }
    }
    pub fn set_stack_value(&mut self, index: usize, value: Value<'h>) {
        let (tag, data) = value.to_raw_parts();
        self.0.value_stack_tag[index] = tag;
        self.0.value_stack_data[index] = data;
    }
    pub fn grow_stack(&mut self, amount: usize, default: Value<'h>) {
        let (tag, data) = default.to_raw_parts();
        self.0.value_stack_tag.extend(repeat_n(tag, amount));
        self.0.value_stack_data.extend(repeat_n(data, amount));
    }
    pub fn shrink_stack(&mut self, amount: usize) {
        let new_size = self.0.value_stack_data.len() - amount;
        self.0.value_stack_data.truncate(new_size);
        self.0.value_stack_tag.truncate(new_size);
    }
    pub fn push_call_frame(&mut self, function: &Object<Function>) {
        let call_frame = CallFrame {
            function: self.function,
            instr_ptr: self.instr_ptr,
        };
        self.0.call_stack.push(call_frame);
        self.function = ptr::from_ref(function).cast_mut();
        self.instr_ptr = 0;
    }
    pub fn pop_call_frame(&mut self) -> bool {
        let Some(frame) = self.0.call_stack.pop() else {
            return false;
        };
        self.function = frame.function;
        self.instr_ptr = frame.instr_ptr;
        true
    }
    pub fn get_function(&self) -> &'h Object<Function> {
        unsafe { self.0.function.as_ref_unchecked() }
    }
}

impl<'h> DerefMut for StackGuard<'h> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'h> Deref for StackGuard<'h> {
    type Target = StackInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

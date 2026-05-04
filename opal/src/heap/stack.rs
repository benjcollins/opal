use std::{
    ops::{Deref, DerefMut},
    ptr,
    sync::{Mutex, MutexGuard},
};

use crate::{
    heap::{function::Function, object::Object},
    value::{Value, ValueTag},
};

#[derive(Debug)]
pub struct Stack(pub Mutex<StackInner>);

#[derive(Debug)]
pub struct StackInner {
    pub(super) call_stack: Vec<CallFrame>,
    pub(super) value_stack_tag: Vec<ValueTag>,
    pub(super) value_stack_data: Vec<*mut ()>,
    pub(super) function: *mut Object<Function>,
    pub base_ptr: usize,
    pub instr_ptr: usize,
}

pub struct StackGuard<'h>(MutexGuard<'h, StackInner>);

#[derive(Debug)]
pub struct CallFrame {
    pub instr_ptr: usize,
    pub function: *mut Object<Function>,
}

pub struct Values<'h> {
    index: usize,
    tags: &'h [ValueTag],
    data: &'h [*mut ()],
}

impl<'h> Iterator for Values<'h> {
    type Item = Value<'h>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.tags.len() {
            return None;
        }
        let tag = self.tags[self.index];
        let data = self.data[self.index];
        self.index += 1;
        unsafe { Some(Value::from_raw_parts(tag, data)) }
    }
}

impl Stack {
    pub fn lock(&self) -> StackGuard<'_> {
        StackGuard(self.0.lock().unwrap())
    }
}

impl<'h> StackGuard<'h> {
    pub fn values(&self) -> Values<'_> {
        Values {
            index: 0,
            tags: &self.value_stack_tag,
            data: &self.value_stack_data,
        }
    }
    pub fn get_stack_value(&self, index: usize) -> Value<'h> {
        unsafe { Value::from_raw_parts(self.0.value_stack_tag[index], self.0.value_stack_data[index]) }
    }
    pub fn set_stack_value(&mut self, index: usize, value: Value<'h>) {
        let (tag, data) = value.to_raw_parts();
        self.0.value_stack_tag[index] = tag;
        self.0.value_stack_data[index] = data;
    }
    // pub fn grow_stack(&mut self, amount: usize, default: Value<'h>) {
    //     let (tag, data) = default.to_raw_parts();
    //     self.0.value_stack_tag.extend(repeat_n(tag, amount));
    //     self.0.value_stack_data.extend(repeat_n(data, amount));
    // }
    // pub fn shrink_stack(&mut self, amount: usize) {
    //     let new_size = self.0.value_stack_data.len() - amount;
    //     self.0.value_stack_data.truncate(new_size);
    //     self.0.value_stack_tag.truncate(new_size);
    // }
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
    pub fn function(&self) -> &'h Object<Function> {
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

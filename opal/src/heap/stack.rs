use std::{
    alloc::{Layout, dealloc},
    marker::PhantomData,
};

use crate::{
    heap::{Heap, mutator::Mutator},
    lower::CompiledFun,
    value::{Value, ValueTag},
};

pub struct Stack<'h, 's> {
    inner: *mut StackInner,
    _phantom: PhantomData<(&'h Heap, &'s CompiledFun<'s>)>,
}

pub(super) struct StackInner {
    pub(super) heap: *const Heap,
    pub(super) next: *mut StackInner,
    pub(super) prev: *mut StackInner,
    pub(super) value_data: Vec<*mut ()>,
    pub(super) value_tag: Vec<ValueTag>,
}

impl<'h, 's> Stack<'h, 's> {
    pub(super) fn new(inner: *mut StackInner) -> Stack<'h, 's> {
        Stack {
            inner,
            _phantom: PhantomData,
        }
    }
    fn inner_mut<'m>(&mut self, _mutator: &'m Mutator<'h>) -> &mut StackInner {
        unsafe { self.inner.as_mut_unchecked() }
    }
    fn inner<'m>(&self, _mutator: &'m Mutator<'h>) -> &StackInner {
        unsafe { self.inner.as_ref_unchecked() }
    }
    pub fn get<'m>(&self, index: usize, mutator: &'m Mutator<'h>) -> Value<'m, 's> {
        let inner = self.inner(mutator);
        let data = inner.value_data[index];
        let tag = inner.value_tag[index];
        unsafe { Value::new(tag, data) }
    }
    pub fn set<'m>(&mut self, index: usize, value: Value, mutator: &'m Mutator<'h>) {
        let inner = self.inner_mut(mutator);
        inner.value_data[index] = value.data();
        inner.value_tag[index] = value.tag();
    }
}

impl<'h, 's> Drop for Stack<'h, 's> {
    fn drop(&mut self) {
        unsafe {
            let heap = (*self.inner).heap.as_ref_unchecked();
            let mut stacks_head = heap.stacks_head.lock().unwrap();
            let prev = (*self.inner).prev;
            let next = (*self.inner).next;
            if prev.is_null() {
                *stacks_head = next;
            } else {
                (*prev).next = next;
            }
            if !next.is_null() {
                (*next).prev = prev;
            }
            dealloc(self.inner.cast(), Layout::new::<StackInner>());
        }
    }
}

use std::{
    alloc::{Layout, dealloc},
    marker::PhantomData,
    ptr::drop_in_place,
    sync::atomic::{AtomicPtr, AtomicU8, Ordering},
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
    pub(super) value_data: Vec<AtomicPtr<()>>,
    pub(super) value_tag: Vec<AtomicU8>,
}

impl<'h, 's> Stack<'h, 's> {
    pub(super) fn new(inner: *mut StackInner) -> Stack<'h, 's> {
        Stack {
            inner,
            _phantom: PhantomData,
        }
    }
    pub fn get<'m>(&self, index: usize, _mutator: &'m Mutator<'h>) -> Value<'m, 's> {
        unsafe {
            let data = (&(*self.inner).value_data)[index].load(Ordering::Relaxed);
            let tag = ValueTag::from_repr((&(*self.inner).value_tag)[index].load(Ordering::Relaxed)).unwrap();
            Value::new(tag, data)
        }
    }
    pub fn set<'m>(&self, index: usize, value: Value, _mutator: &'m Mutator<'h>) {
        unsafe {
            (&(*self.inner).value_data)[index].store(value.data(), Ordering::Relaxed);
            (&(*self.inner).value_tag)[index].store(value.tag() as u8, Ordering::Relaxed);
        }
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
            drop_in_place(self.inner);
            dealloc(self.inner.cast(), Layout::new::<StackInner>());
        }
    }
}

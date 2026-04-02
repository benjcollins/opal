use std::{ptr, sync::atomic::Ordering};

use crate::{
    heap::{
        Heap,
        object::{Array, ArrayHeader, Object, Tag},
    },
    value::ValueTag,
};

pub struct Mutator<'h> {
    pub(super) heap: &'h Heap,
}

impl<'h> Mutator<'h> {
    pub fn alloc_array<'s>(&self, size: usize) -> Object<'_, Array<'s>> {
        unsafe {
            let ptr = self
                .heap
                .alloc_raw(size_of::<ArrayHeader>() + size_of::<*mut ()>() * size, Tag::Array);
            let header = ptr.add(1).cast::<ArrayHeader>();
            (*header).len = size;
            let array = header.add(1).cast::<*mut ()>();
            for index in 0..size {
                *array.add(index) = ptr::null_mut();
            }
            (*header).tag.store(ValueTag::Unit as u8, Ordering::Relaxed);
            Object::from_ptr(ptr.cast())
        }
    }
}

impl<'h> Drop for Mutator<'h> {
    fn drop(&mut self) {
        self.heap.mutators.fetch_sub(1, Ordering::Release);
    }
}

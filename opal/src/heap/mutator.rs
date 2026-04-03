use std::{
    ptr,
    sync::{RwLockReadGuard, atomic::Ordering},
};

use crate::{
    heap::{
        Objects,
        object::{Array, ArrayHeader, Object, Tag},
    },
    value::ValueTag,
};

pub struct Mutator<'h> {
    pub(super) heap_guard: RwLockReadGuard<'h, Objects>,
}

impl<'h> Mutator<'h> {
    pub fn alloc_array<'s>(&self, size: usize) -> Object<'_, Array<'s>> {
        unsafe {
            let ptr = self
                .heap_guard
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
    pub fn finish(self) {}
}

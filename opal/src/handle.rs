use std::{
    alloc::{Layout, dealloc},
    marker::PhantomData,
};

use crate::heap::ObjectHeader;

pub struct HandleInner {
    pub prev: *mut HandleInner,
    pub object: *mut ObjectHeader,
    pub next: *mut HandleInner,
}

pub struct Handle<'h, T> {
    pub inner: *mut HandleInner,
    pub _phantom: PhantomData<&'h T>,
}

impl<'h, T> Handle<'h, T> {
    pub fn object_ptr(&self) -> *mut ObjectHeader {
        unsafe { (*self.inner).object }
    }
}

impl<'h, T> Drop for Handle<'h, T> {
    fn drop(&mut self) {
        unsafe {
            let next = (*self.inner).next;
            let prev = (*self.inner).prev;
            if !next.is_null() {
                (*next).prev = prev;
            }
            if !prev.is_null() {
                (*prev).next = next;
            }

            dealloc(self.inner.cast(), Layout::new::<HandleInner>());
        }
    }
}

use std::{marker::PhantomData, ptr, sync::atomic::Ordering};

use crate::heap::{Heap, object::Object};

#[derive(Debug)]
pub struct Handle<T> {
    object: *mut Object<T>,
    _phantom: PhantomData<T>,
}

impl<T> Handle<T> {
    pub fn new(object: &Object<T>) -> Handle<T> {
        (*object).header.handle_count.fetch_add(1, Ordering::Relaxed);
        Handle {
            object: ptr::from_ref(object).cast_mut(),
            _phantom: PhantomData,
        }
    }
    pub fn to_object<'h>(&self, _heap: &'h Heap) -> &'h Object<T> {
        unsafe { self.object.as_ref_unchecked() }
    }
}

impl<T> Drop for Handle<T> {
    fn drop(&mut self) {
        unsafe {
            (*self.object).header.handle_count.fetch_sub(1, Ordering::Relaxed);
        }
    }
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        unsafe {
            (*self.object).header.handle_count.fetch_add(1, Ordering::Relaxed);
            Handle {
                object: self.object,
                _phantom: PhantomData,
            }
        }
    }
}

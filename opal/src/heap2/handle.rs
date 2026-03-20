use std::{
    alloc::{Layout, alloc, dealloc},
    marker::PhantomData,
    ptr::null_mut,
    sync::Mutex,
};

use crate::heap2::{Object, ObjectHeader};

struct Handle {
    prev: *mut Handle,
    object: *mut ObjectHeader,
    next: *mut Handle,
}

pub struct ObjectHandle<T> {
    inner: *mut Handle,
    _phantom: PhantomData<T>,
}

impl<T> ObjectHandle<T> {
    pub fn object_ptr(&self) -> *mut ObjectHeader {
        unsafe { (*self.inner).object }
    }
}

pub struct Handles {
    first: *mut Handle,
}

unsafe impl Send for Handles {}

pub static GLOBAL_HANDLES: Mutex<Handles> = Mutex::new(Handles { first: null_mut() });

impl Handles {
    pub fn add<T>(&mut self, object: Object<'_, T>) -> ObjectHandle<T> {
        unsafe {
            let handle = alloc(Layout::new::<Handle>()).cast::<Handle>();
            (*handle).object = object.as_ptr();
            (*handle).next = self.first;
            (*handle).prev = null_mut();
            if !self.first.is_null() {
                (*self.first).prev = handle;
            }
            self.first = handle;

            ObjectHandle {
                inner: handle,
                _phantom: PhantomData,
            }
        }
    }
}

impl<'h> IntoIterator for &'h Handles {
    type Item = *mut ObjectHeader;

    type IntoIter = HandlesIter<'h>;

    fn into_iter(self) -> Self::IntoIter {
        HandlesIter {
            current: self.first,
            _phantom: PhantomData,
        }
    }
}

pub struct HandlesIter<'h> {
    current: *mut Handle,
    _phantom: PhantomData<&'h ()>,
}

impl<'h> Iterator for HandlesIter<'h> {
    type Item = *mut ObjectHeader;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.current.is_null() {
                None
            } else {
                let current = self.current;
                self.current = (*current).next;
                Some((*current).object)
            }
        }
    }
}

impl<T> Drop for ObjectHandle<T> {
    fn drop(&mut self) {
        let _handles = GLOBAL_HANDLES.lock().unwrap();

        unsafe {
            let next = (*self.inner).next;
            let prev = (*self.inner).prev;
            if !next.is_null() {
                (*next).prev = prev;
            }
            if !prev.is_null() {
                (*prev).next = next;
            }

            dealloc(self.inner.cast(), Layout::new::<Handle>());
        }
    }
}

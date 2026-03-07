use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    marker::PhantomData,
};

struct Heap {
    objects: *mut u8,
    mutators: Cell<u32>,
    handles: RefCell<HashMap<*mut u8, u32>>,
}

struct ObjectHeader {
    next: *mut u8,
}

impl Heap {
    pub fn new() -> Heap {
        todo!()
    }
    pub fn alloc<T>(&self) -> ObjectMutator<'_, T> {
        todo!()
    }
}

impl Drop for Heap {
    fn drop(&mut self) {
        todo!()
    }
}

struct ObjectHandle<'h, T> {
    ptr: *mut u8,
    heap: &'h Heap,
    _phantom: PhantomData<T>,
}

impl<'h, T> ObjectHandle<'h, T> {
    fn mutator(&self) -> ObjectMutator<'h, T> {
        todo!()
    }
}

impl<'h, T> Clone for ObjectHandle<'h, T> {
    fn clone(&self) -> Self {
        let mut handles = self.heap.handles.borrow_mut();
        let handle_count = handles.get_mut(&self.ptr).unwrap();
        *handle_count += 1;
        ObjectHandle {
            ptr: self.ptr,
            heap: self.heap,
            _phantom: PhantomData,
        }
    }
}

impl<'h, T> Drop for ObjectHandle<'h, T> {
    fn drop(&mut self) {
        let mut handles = self.heap.handles.borrow_mut();
        let handle_count = handles.get_mut(&self.ptr).unwrap();
        *handle_count -= 1;
        if *handle_count == 0 {
            handles.remove(&self.ptr);
        }
    }
}

struct ObjectMutator<'h, T> {
    ptr: *mut u8,
    heap: &'h Heap,
    _phantom: PhantomData<&'h T>,
}

impl<'h, T> Drop for ObjectMutator<'h, T> {
    fn drop(&mut self) {
        self.heap.mutators.set(self.heap.mutators.get() - 1);
    }
}

struct Object<'h, 'm, T> {
    ptr: *mut u8,
    _phantom: PhantomData<(&'h (), &'m (), T)>,
}

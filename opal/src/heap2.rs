use std::marker::PhantomData;

struct Heap {}

struct Object<'h, T = ()> {
    ptr: *mut u8,
    _phantom: PhantomData<&'h T>,
}

struct ObjectHandle<T = ()> {
    inner: *mut ObjectHandleInner,
    _phantom: PhantomData<T>,
}

struct ObjectHandleInner {
    prev: *mut ObjectHandleInner,
    object: *mut u8,
    next: *mut ObjectHandleInner,
}

enum Tag {
    Fun,
}

impl Heap {
    pub fn init() -> Option<Heap> {
        todo!()
    }
    pub fn create_handle<'h>(&'h self, object: Object<'h>) -> ObjectHandle {
        todo!()
    }
    pub unsafe fn collect(&mut self, mark_roots: impl FnOnce()) {
        todo!()
    }
}

impl<'h> Object<'h> {}

// heap object types
// function: instrs, values
// call stack
// value stack
// record
// array

// what do heap objects need to be able to do
// determine the size of the object so that it can be deallocated
// trace pointers through the object

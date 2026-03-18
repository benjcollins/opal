pub mod fun_object;

use std::{marker::PhantomData, ptr::null_mut, sync::Once};

use crate::heap2::fun_object::FunObjectRef;

pub struct Heap {
    objects: *mut ObjectHeader,
    handles: *mut Handle,
}

struct ObjectHeader {
    next: *mut ObjectHeader,
    tag: ObjectTag,
    marked: bool,
}

pub struct ObjectRef<'h> {
    ptr: *mut ObjectHeader,
    _phantom: PhantomData<&'h ()>,
}

pub struct ObjectHandle {
    inner: *mut Handle,
}

struct Handle {
    prev: *mut Handle,
    object: *mut ObjectHeader,
    next: *mut Handle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectTag {
    Fun,
}

impl<'h> Clone for ObjectRef<'h> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'h> Copy for ObjectRef<'h> {}

pub trait HeapObject<'h> {
    fn size(&self) -> usize;
    fn trace(&self);
}

// pub trait HeapObject<'h> {
//     type Init;

//     const TAG: ObjectTag;

//     fn upcast(&self) -> ObjectRef<'h>;
//     fn downcast(object: ObjectRef<'h>) -> Self;
//     fn size(&self) -> usize;
//     fn trace(&self);
// }

impl Heap {
    pub fn init() -> Option<Heap> {
        static INIT: Once = Once::new();
        let mut heap = None;
        INIT.call_once(|| {
            heap = Some(Heap {
                objects: null_mut(),
                handles: null_mut(),
            })
        });
        heap
    }
    // pub fn alloc<'h, T: HeapObject<'h>>(&'h self, init: T::Init) -> T {
    //     todo!()
    // }
    pub fn create_handle<'h, O>(&'h self, object: ObjectRef<'h>) -> ObjectHandle {
        todo!()
    }
}

impl<'h> ObjectRef<'h> {
    unsafe fn from_ptr(ptr: *mut ObjectHeader) -> ObjectRef<'h> {
        ObjectRef {
            ptr,
            _phantom: PhantomData,
        }
    }
    fn weird(&self) -> Box<dyn HeapObject<'h> + 'h> {
        unsafe {
            match (*self.ptr).tag {
                ObjectTag::Fun => Box::new(self.downcast::<FunObjectRef>().unwrap()),
            }
        }
    }
    pub fn downcast<T: HeapObject<'h>>(self) -> Option<T> {
        unsafe {
            if (*self.ptr).tag == T::TAG {
                Some(T::downcast(self))
            } else {
                None
            }
        }
    }
}

// heap object types
// function: instrs, values
// call stack
// value stack
// record
// array

// what do heap objects need to be able to do
// determine the size of the object so that it can be deallocated
// trace pointers through the object

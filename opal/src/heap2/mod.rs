pub mod handle;

use std::{
    alloc::{Layout, alloc, dealloc},
    marker::PhantomData,
    mem,
    ops::Deref,
    ptr::{self, replace},
    slice,
    sync::{
        Once,
        atomic::{AtomicPtr, Ordering},
    },
};

use crate::heap2::handle::{GLOBAL_HANDLES, ObjectHandle};

pub struct Heap {
    objects: AtomicPtr<ObjectHeader>,
}

pub struct ObjectHeader {
    next: *mut ObjectHeader,
    tag: Tag,
    marked: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TypeTag {
    Int,
    Float,
    Bool,
}

pub struct Object<'h, T> {
    ptr: *mut ObjectHeader,
    _phantom: PhantomData<&'h T>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tag {
    Array { len: usize, cap: usize, tag: TypeTag },
    Bytes { len: usize, cap: usize },
    Native { size: usize, drop: fn(*mut ()) },
}

impl<'h, T> Clone for Object<'h, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'h, T> Copy for Object<'h, T> {}

const HEAP_ALIGN: usize = align_of::<*mut ()>();

pub struct Array;
pub struct Bytes;
pub struct Native<T>(PhantomData<T>);

struct NativeHeader {
    drop: fn(*mut ()),
    size: usize,
}

struct ArrayHeader {
    len: usize,
    cap: usize,
    tag: TypeTag,
}

impl Heap {
    pub fn init() -> Option<Heap> {
        static INIT: Once = Once::new();
        let mut heap = None;
        INIT.call_once(|| {
            heap = Some(Heap {
                objects: AtomicPtr::new(ptr::null_mut()),
            })
        });
        heap
    }
    fn alloc_raw(&self, tag: Tag, size: usize) -> *mut ObjectHeader {
        unsafe {
            let size = size_of::<ObjectHeader>() + size;
            let layout = Layout::from_size_align_unchecked(size, HEAP_ALIGN);
            let ptr = alloc(layout).cast::<ObjectHeader>();
            (*ptr).tag = tag;
            (*ptr).next = self.objects.swap(ptr, Ordering::Relaxed);
            ptr
        }
    }
    pub fn alloc_bytes(&self, bytes: &[u8]) -> Object<'_, Bytes> {
        unsafe {
            let object_ptr = self.alloc_raw(Tag::Bytes, size_of::<usize>() + bytes.len());
            let len_ptr = object_ptr.add(1).cast::<usize>();
            *len_ptr = bytes.len();
            let bytes_ptr = len_ptr.add(1).cast::<u8>();
            ptr::copy(bytes.as_ptr(), bytes_ptr, bytes.len());
            Object::from_ptr(object_ptr)
        }
    }
    pub fn alloc_array(&self, size: usize) -> Object<'_, Array> {
        unsafe {
            let object_ptr = self.alloc_raw(Tag::Array, size_of::<usize>() + size_of::<Value>() * size);
            let len_ptr = object_ptr.add(1).cast::<usize>();
            *len_ptr = todo!();
        }
    }
    pub fn alloc_native<T>(&self, value: T) -> Object<'_, Native<T>> {
        unsafe {
            let object_ptr = self.alloc_raw(Tag::Native, size_of::<NativeHeader>() + size_of::<T>());
            let native_header_ptr = object_ptr.add(1).cast::<NativeHeader>();
            (*native_header_ptr).size = size_of::<T>();
            (*native_header_ptr).drop = |value_ptr| ptr::drop_in_place(value_ptr.cast::<T>());
            let value_ptr = native_header_ptr.add(1).cast::<T>();
            mem::forget(replace(value_ptr, value));
            Object::from_ptr(object_ptr)
        }
    }
    pub fn object_from_handle<'h, T>(&'h self, handle: &ObjectHandle<T>) -> Object<'h, T> {
        unsafe { Object::from_ptr(handle.object_ptr()) }
    }
    pub fn collect_garbage(&mut self) {
        unsafe {
            let handles = GLOBAL_HANDLES.lock().unwrap();
            for object_ptr in &*handles {
                (*object_ptr).marked = true;
            }

            // TODO: tracing

            let mut previous: *mut ObjectHeader = ptr::null_mut();
            let mut current = self.objects.load(Ordering::Relaxed);
            while !current.is_null() {
                if (*current).marked {
                    previous = current;
                } else {
                    println!("freeing!");
                    if !previous.is_null() {
                        (*previous).next = (*current).next;
                    }
                    if (*current).tag == Tag::Native {
                        let native_header_ptr = current.add(1).cast::<NativeHeader>();
                        let value_ptr = native_header_ptr.add(1);
                        ((*native_header_ptr).drop)(value_ptr.cast())
                    }
                    let size = size_of::<ObjectHeader>() + object_size(current);
                    dealloc(current.cast(), Layout::from_size_align_unchecked(size, HEAP_ALIGN));
                }
                current = (*current).next;
            }
        }
    }
}

fn object_size(object_ptr: *mut ObjectHeader) -> usize {
    unsafe {
        match (*object_ptr).tag {
            Tag::Bytes => {
                let len_ptr = object_ptr.add(1).cast::<usize>();
                size_of::<usize>() + *len_ptr
            }
            Tag::Native => {
                let native_header_ptr = object_ptr.add(1).cast::<NativeHeader>();
                (*native_header_ptr).size
            }
            Tag::Array => todo!(),
        }
    }
}

impl<'h, T> Object<'h, T> {
    unsafe fn from_ptr(ptr: *mut ObjectHeader) -> Object<'h, T> {
        Object {
            ptr,
            _phantom: PhantomData,
        }
    }
    fn as_ptr(self) -> *mut ObjectHeader {
        self.ptr
    }
    pub fn into_handle(self) -> ObjectHandle<T> {
        GLOBAL_HANDLES.lock().unwrap().add(self)
    }
}

impl<'h> Deref for Object<'h, Bytes> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe {
            let len_ptr = self.ptr.add(1).cast::<usize>();
            let bytes_ptr = len_ptr.add(1).cast::<u8>();
            slice::from_raw_parts(bytes_ptr, *len_ptr)
        }
    }
}

impl<'h, T> Deref for Object<'h, Native<T>> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            let native_header_ptr = self.ptr.add(1).cast::<NativeHeader>();
            let value_ptr = native_header_ptr.add(1).cast::<T>();
            value_ptr.as_ref().unwrap()
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

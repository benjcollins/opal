use std::{
    alloc::{Layout, alloc, dealloc},
    cell::Cell,
    marker::PhantomData,
    mem,
    ops::Deref,
    ptr::{self, replace},
    slice,
};

use crate::{
    handle::{Handle, HandleInner},
    instr::Val,
    value::Value,
};

pub struct Heap {
    objects: Cell<*mut ObjectHeader>,
    handles: Cell<*mut HandleInner>,
    mutators: Cell<u32>,
}

#[derive(Clone, Copy)]
pub struct ObjectHeader {
    next: *mut ObjectHeader,
    metadata: Metadata,
    marked: bool,
    tag: Tag,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tag {
    Array,
    Bytes,
    Native,
}

#[derive(Clone, Copy)]
union Metadata {
    array: usize,
    bytes: usize,
    native: (usize, fn(*mut ())),
}

#[derive(Debug)]
pub struct Object<'m, T> {
    ptr: *mut ObjectHeader,
    _phantom: PhantomData<&'m T>,
}

impl<'h, T> Clone for Object<'h, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'m, T> Copy for Object<'m, T> {}

const HEAP_ALIGN: usize = align_of::<*mut ()>();

#[derive(Debug)]
pub struct Array<'s>(PhantomData<&'s ()>);

#[derive(Debug)]
pub struct Bytes;

#[derive(Debug)]
pub struct Native<T>(PhantomData<T>);

fn size(tag: Tag, metadata: Metadata) -> usize {
    unsafe {
        match tag {
            Tag::Array => metadata.array * size_of::<Value>(),
            Tag::Bytes => metadata.bytes,
            Tag::Native => {
                let (size, _) = metadata.native;
                size
            }
        }
    }
}

thread_local! {
    static HEAP_EXISTS: Cell<bool> = Cell::new(false);
}

impl Drop for Heap {
    fn drop(&mut self) {
        // self.collect_garbage();
        // assert heap is empty!
        HEAP_EXISTS.set(false);
    }
}

pub struct Mutator<'h> {
    heap: &'h Heap,
}

impl<'h> Mutator<'h> {
    pub fn alloc_bytes(&self, bytes: &[u8]) -> Object<'_, Bytes> {
        unsafe {
            let object_ptr = self.heap.alloc_raw(Tag::Bytes, Metadata { bytes: bytes.len() });
            let bytes_ptr = object_ptr.add(1).cast::<u8>();
            ptr::copy(bytes.as_ptr(), bytes_ptr, bytes.len());
            Object::from_ptr(object_ptr)
        }
    }
    pub fn alloc_array<'s>(&self, len: usize) -> Object<'_, Array<'s>> {
        unsafe {
            let object_ptr = self.heap.alloc_raw(Tag::Array, Metadata { array: len });
            let data_ptr = object_ptr.add(1).cast::<Value>();
            for i in 0..len {
                *data_ptr.add(i) = Value::Unit;
            }
            Object::from_ptr(object_ptr)
        }
    }
    pub fn alloc_native<T>(&self, value: T) -> Object<'_, Native<T>> {
        unsafe {
            let object_ptr = self.heap.alloc_raw(
                Tag::Native,
                Metadata {
                    native: (size_of::<T>(), |value_ptr| ptr::drop_in_place(value_ptr.cast::<T>())),
                },
            );
            let value_ptr = object_ptr.add(1).cast::<T>();
            mem::forget(replace(value_ptr, value));
            Object::from_ptr(object_ptr)
        }
    }
    pub fn object_from_handle<T>(&self, handle: &Handle<'h, T>) -> Object<'_, T> {
        unsafe { Object::from_ptr(handle.object_ptr()) }
    }
}

impl<'h> Drop for Mutator<'h> {
    fn drop(&mut self) {
        self.heap.mutators.set(self.heap.mutators.get() - 1);
    }
}

impl Heap {
    pub fn new() -> Option<Heap> {
        if !HEAP_EXISTS.get() {
            HEAP_EXISTS.set(false);
            Some(Heap {
                objects: Cell::new(ptr::null_mut()),
                handles: Cell::new(ptr::null_mut()),
                mutators: Cell::new(0),
            })
        } else {
            None
        }
    }
    pub fn mutator(&self) -> Mutator<'_> {
        self.mutators.set(self.mutators.get() + 1);
        Mutator { heap: self }
    }
    unsafe fn alloc_raw(&self, tag: Tag, metadata: Metadata) -> *mut ObjectHeader {
        unsafe {
            let size = size_of::<ObjectHeader>() + size(tag, metadata);
            let layout = Layout::from_size_align_unchecked(size, HEAP_ALIGN);
            let ptr = alloc(layout).cast::<ObjectHeader>();
            (*ptr).metadata = metadata;
            (*ptr).next = self.objects.get();
            self.objects.set(ptr);
            ptr
        }
    }
    pub fn create_handle<T>(&self, object: Object<'_, T>) -> Handle<'_, T> {
        unsafe {
            let handle = alloc(Layout::new::<HandleInner>()).cast::<HandleInner>();
            (*handle).object = object.as_ptr();
            (*handle).next = self.handles.get();
            (*handle).prev = ptr::null_mut();
            if !self.handles.get().is_null() {
                (*self.handles.get()).prev = handle;
            }
            self.handles.set(handle);

            Handle {
                inner: handle,
                _phantom: PhantomData,
            }
        }
    }
    fn mark_roots(&mut self) {
        unsafe {
            let mut current = self.handles.get();
            while !current.is_null() {
                (*(*current).object).marked = true;
                current = (*current).next;
            }
        }
    }
    fn sweep_unmarked(&mut self) {
        unsafe {
            let mut previous: *mut ObjectHeader = ptr::null_mut();
            let mut current = self.objects.get();
            while !current.is_null() {
                if (*current).marked {
                    previous = current;
                } else {
                    println!("freeing!");
                    if !previous.is_null() {
                        (*previous).next = (*current).next;
                    }
                    if (*current).tag == Tag::Native {
                        let (_, drop) = (*current).metadata.native;
                        (drop)(current.add(1).cast())
                    }
                    let size = size_of::<ObjectHeader>() + size((*current).tag, (*current).metadata);
                    dealloc(current.cast(), Layout::from_size_align_unchecked(size, HEAP_ALIGN));
                }
                current = (*current).next;
            }
        }
    }
    pub fn collect_garbage(&mut self) {
        if self.mutators.get() > 0 {
            panic!()
        }
        self.mark_roots();
        // TODO: tracing
        self.sweep_unmarked();
    }
}

impl<'m, T> Object<'m, T> {
    unsafe fn from_ptr(ptr: *mut ObjectHeader) -> Object<'m, T> {
        Object {
            ptr,
            _phantom: PhantomData,
        }
    }
    pub fn as_ptr(self) -> *mut ObjectHeader {
        self.ptr
    }
}

impl<'m> Deref for Object<'m, Bytes> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe {
            let data_ptr = self.ptr.add(1).cast::<u8>();
            slice::from_raw_parts(data_ptr, (*self.ptr).metadata.bytes)
        }
    }
}

impl<'m, T> Deref for Object<'m, Native<T>> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            let value_ptr = self.ptr.add(1).cast::<T>();
            value_ptr.as_ref().unwrap()
        }
    }
}

impl<'m, 's> Object<'m, Array<'s>> {
    pub fn len(&self) -> usize {
        unsafe { (*self.ptr).metadata.array }
    }
    pub fn set(&self, index: usize, value: Value<'m, 's>) {
        unsafe {
            if index >= self.len() {
                panic!()
            }
            let data_ptr = self.ptr.add(1).cast::<Value>();
            *data_ptr.add(index) = value;
        }
    }
    pub fn get(&self, index: usize) -> Value<'m, 's> {
        unsafe {
            if index >= self.len() {
                panic!()
            }
            let data_ptr = self.ptr.add(1).cast::<Value>();
            *data_ptr.add(index)
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

use std::{
    marker::PhantomData,
    slice,
    sync::atomic::{AtomicPtr, AtomicU8, Ordering},
};

use crate::{
    lower::CompiledFun,
    value::{Value, ValueTag},
};

pub struct Object<'m, T> {
    ptr: *mut ObjectHeader,
    _phantom: PhantomData<&'m T>,
}

pub(super) struct ObjectHeader {
    pub(super) next: AtomicPtr<ObjectHeader>,
    pub(super) tag: Tag,
    pub(super) marked: bool,
}

impl<'m, T> Clone for Object<'m, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'m, T> Copy for Object<'m, T> {}
unsafe impl<'m, T> Send for Object<'m, T> {}
unsafe impl<'m, T> Sync for Object<'m, T> {}

pub(super) enum Tag {
    Array,
}

impl<'m, T> Object<'m, T> {
    pub unsafe fn from_ptr(ptr: *mut ()) -> Object<'m, T> {
        assert!(!ptr.is_null());
        Object {
            ptr: ptr.cast(),
            _phantom: PhantomData,
        }
    }
    pub fn as_ptr(&self) -> *mut () {
        self.ptr.cast()
    }
}

pub(super) struct ArrayHeader {
    pub(super) len: usize,
    pub(super) tag: AtomicU8,
}

pub struct Array<'s>(PhantomData<&'s CompiledFun<'s>>);

impl<'m, 's> Object<'m, Array<'s>> {
    unsafe fn header(&self) -> *mut ArrayHeader {
        unsafe { self.ptr.add(1).cast() }
    }
    fn slice(&self) -> &[AtomicPtr<()>] {
        unsafe {
            let header = self.header();
            let ptr = header.add(1).cast::<AtomicPtr<()>>();
            slice::from_raw_parts(ptr, (*header).len)
        }
    }
    pub fn len(&self) -> usize {
        unsafe { (*self.header()).len }
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn get(&self, index: usize) -> Value<'m, 's> {
        unsafe {
            let tag = (*self.header()).tag.load(Ordering::Relaxed);
            let tag = ValueTag::from_repr(tag).expect("invalid tag");
            let data = self.slice()[index].load(Ordering::Relaxed);
            Value::new(tag, data)
        }
    }
    pub fn set(&self, index: usize, value: Value<'m, 's>) {
        unsafe {
            let tag_atomic = &(*self.header()).tag;
            let tag = ValueTag::from_repr(tag_atomic.load(Ordering::Relaxed)).unwrap();
            if tag == value.tag() {
                self.slice()[index].store(value.data(), Ordering::Relaxed);
                return;
            }
            if tag == ValueTag::Unit {
                match tag_atomic.compare_exchange(
                    ValueTag::Unit as u8,
                    value.tag() as u8,
                    Ordering::AcqRel,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => {
                        self.slice()[index].store(value.data(), Ordering::Relaxed);
                    }
                    Err(_) => panic!("array tag is no longer undefined"),
                }
            }
        }
    }
}

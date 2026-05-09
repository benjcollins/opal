use std::{
    ops::{Deref, DerefMut},
    ptr, slice,
    sync::atomic::{AtomicBool, AtomicPtr, AtomicU32},
};

use crate::heap::{handle::Handle, object_layout};

pub trait ObjectTrait {
    type Item: Copy;
    fn size(&self) -> usize {
        0
    }
}

#[derive(Debug)]
pub struct ObjectHeader {
    pub next: AtomicPtr<ObjectHeader>,
    pub handle_count: AtomicU32,
    pub tag: ObjectTag,
    pub marked: AtomicBool,
}

#[derive(Debug)]
#[repr(C)]
pub struct Object<T> {
    pub header: ObjectHeader,
    pub body: T,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectTag {
    Function,
    Stack,
    List,
    Bytes,
}

impl<T> Deref for Object<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.body
    }
}

impl<T> DerefMut for Object<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.body
    }
}

impl<T> Object<T> {
    pub fn to_handle(&self) -> Handle<T> {
        Handle::new(self)
    }
}

impl<T: ObjectTrait> Object<T> {
    pub fn extended(&self) -> &[T::Item] {
        let (_, offset) = object_layout(&self.body);
        unsafe { slice::from_raw_parts(ptr::from_ref(self).byte_add(offset).cast(), self.size()) }
    }
}

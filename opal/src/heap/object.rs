use std::{
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, AtomicPtr, AtomicU32},
};

use crate::heap::handle::Handle;

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

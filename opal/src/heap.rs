use std::{
    alloc::{GlobalAlloc, Layout, alloc, dealloc},
    cell::Cell,
    marker::PhantomData,
};

use strum::{EnumIs, FromRepr};

use crate::value::Value;

pub struct ObjectHeap {
    ptr: *mut u8,
    offset: Cell<usize>,
}

#[derive(Debug, Clone, Copy)]
pub struct HeapObject<'h> {
    pub ptr: *mut ObjectHeader,
    pub phantom: PhantomData<&'h ()>,
}

#[derive(Debug, Clone, Copy)]
pub struct ObjectHeader(u64);

#[derive(Debug, Clone, Copy)]
pub struct ArrayObject<'h>(HeapObject<'h>);

#[derive(FromRepr, EnumIs)]
#[repr(u8)]
enum Tag {
    Bytes,
    Record,
    Array,
}

const HEAP_SIZE: usize = 1024 * 1024 * 1024;

impl<'h> HeapObject<'h> {
    fn header(&self) -> ObjectHeader {
        unsafe { self.ptr.read() }
    }
    fn data_ptr(self) -> *mut u8 {
        unsafe { self.ptr.add(1) }.cast()
    }
    pub fn as_array(&self) -> Option<ArrayObject<'h>> {
        if self.header().tag().is_array() {
            Some(ArrayObject(*self))
        } else {
            None
        }
    }
}

impl<'h> ArrayObject<'h> {
    fn len(self) -> u64 {
        self.0.header().payload()
    }
    pub fn get(self, index: u64) -> Value<'h> {
        let ptr: *mut Value<'h> = self.0.data_ptr().cast();
        if index >= self.len() {
            panic!()
        }
        unsafe { ptr.add(index as usize).read() }
    }
    pub fn set(self, index: u64, value: Value<'h>) {
        let ptr: *mut Value<'h> = self.0.data_ptr().cast();
        if index >= self.len() {
            panic!()
        }
        unsafe { ptr.add(index as usize).write(value) }
    }
    pub fn heap_object(self) -> HeapObject<'h> {
        self.0
    }
}

impl ObjectHeader {
    fn new(tag: Tag, payload: u64) -> ObjectHeader {
        ObjectHeader(tag as u64 | payload << 4)
    }
    fn tag(self) -> Tag {
        Tag::from_repr((self.0 & 0x3) as u8).unwrap()
    }
    fn payload(self) -> u64 {
        self.0 >> 4
    }
}

fn align_down(ptr: usize, align: usize) -> usize {
    ptr & !(align - 1)
}

fn align_up(ptr: usize, align: usize) -> usize {
    align_down(ptr + align - 1, align)
}

impl ObjectHeap {
    pub fn new() -> ObjectHeap {
        unsafe {
            let ptr = alloc(Layout::array::<u64>(HEAP_SIZE).unwrap());
            ObjectHeap {
                ptr,
                offset: Cell::new(0),
            }
        }
    }
    fn alloc(&self, size: usize) -> *mut u8 {
        let offset = self.offset.get();
        self.offset.set(align_up(offset + size, 8));
        unsafe { self.ptr.add(offset) }
    }
    pub fn alloc_array(&self, len: u64) -> ArrayObject<'_> {
        let size = (1 + len as usize) * size_of::<u64>();
        let ptr: *mut ObjectHeader = self.alloc(size).cast();
        unsafe { ptr.write(ObjectHeader::new(Tag::Array, len)) };
        let heap_object = HeapObject {
            ptr,
            phantom: PhantomData,
        };
        ArrayObject(heap_object)
    }
}

impl Drop for ObjectHeap {
    fn drop(&mut self) {
        unsafe { dealloc(self.ptr, Layout::array::<u64>(HEAP_SIZE).unwrap()) }
    }
}

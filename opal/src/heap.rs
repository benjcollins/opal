use std::{
    cell::RefCell,
    marker::PhantomData,
    ptr::{self, null_mut},
    sync::atomic::AtomicU32,
};

use elsa::FrozenMap;
use libc::{MAP_ANONYMOUS, PROT_READ, PROT_WRITE, mmap};

use crate::{instr::Instr, value::Value};

enum Tag {
    Fun,
    Array,
}

static HEAP_SIZE: usize = 64 * 1024 * 1024; // 64 MB

pub struct Heap {
    base: *mut u8,
    offset: usize,
    roots: FrozenMap<ObjectPtr, Box<AtomicU32>>,
}

pub struct ObjectHeader(usize);

pub struct ObjectHandle<'h, T> {
    object: ObjectPtr,
    ref_count: &'h AtomicU32,
    phantom: PhantomData<T>,
}

pub struct ObjectRef<'a, T> {
    ptr: ObjectPtr,
    phantom: PhantomData<&'a T>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectPtr(*mut u8);

pub struct ArrayObject;
pub struct FunObject;

trait HeapObject {
    type Init<'a>;
    const TAG: Tag;
    fn size(init: &Self::Init<'_>) -> usize;
    fn init(ptr: *mut u8, init: &Self::Init<'_>);
}

#[repr(align(8))]
struct FunObjectHeader {
    instr_len: usize,
    consts_len: usize,
}

impl HeapObject for FunObject {
    type Init<'a> = (&'a [Instr], &'a [Value]);
    const TAG: Tag = Tag::Fun;

    fn size((instrs, consts): &Self::Init<'_>) -> usize {
        size_of::<FunObjectHeader>() + instrs.len() * size_of::<Instr>() + consts.len() * size_of::<Value>()
    }
    fn init(mut ptr: *mut u8, (instrs, consts): &Self::Init<'_>) {
        unsafe {
            let fun_object_header = ptr.cast::<FunObjectHeader>();
            fun_object_header.write(FunObjectHeader {
                instr_len: instrs.len(),
                consts_len: consts.len(),
            });
            ptr = ptr.add(size_of::<FunObjectHeader>());
            ptr::copy(instrs.as_ptr() as *mut u8, ptr, size_of::<Instr>() * instrs.len());
            ptr = ptr.add(size_of::<Instr>());
            ptr::copy(instrs.as_ptr() as *mut u8, ptr, size_of::<Instr>() * instrs.len());
        }
    }
}

impl<'a> ObjectRef<'a, ArrayObject> {
    pub fn get(&self, index: usize) -> Value {
        todo!()
    }
}

impl<'a> ObjectRef<'a, FunObject> {
    pub fn instrs(self) -> &'a [Instr] {
        todo!()
    }
    pub fn consts(self) -> &'a [Value] {
        todo!()
    }
}

impl Heap {
    pub fn new() -> Heap {
        let base = unsafe { mmap(null_mut(), HEAP_SIZE, PROT_READ | PROT_WRITE, MAP_ANONYMOUS, -1, 0) } as *mut u8;
        Heap {
            base,
            offset: 0,
            roots: FrozenMap::new(),
        }
    }
    pub fn alloc_raw<T: HeapObject>(&self) -> ObjectPtr {
        todo!()
    }
    pub fn alloc<T>(&self) -> ObjectHandle<'_, T> {
        todo!()
    }
}

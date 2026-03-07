use std::{
    cell::Cell,
    io::Error,
    marker::PhantomData,
    ptr::null_mut,
    sync::{RwLock, RwLockReadGuard, atomic::AtomicU32},
};

use elsa::FrozenMap;
use libc::{MAP_ANONYMOUS, MAP_FAILED, MAP_PRIVATE, PROT_READ, PROT_WRITE, mmap};

use crate::{instr::Instr, value::Value};

static HEAP_SIZE: usize = 64 * 1024 * 1024; // 64 MB

pub struct Heap(RwLock<HeapInner>);

pub struct HeapLock<'h> {
    heap: &'h Heap,
    guard: RwLockReadGuard<'h, HeapInner>,
}

struct HeapInner {
    base: *mut u8,
    offset: Cell<usize>,
    roots: FrozenMap<ObjectPtr, Box<AtomicU32>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectPtr(*mut u8);

pub struct RootedObject<'h, T> {
    ptr: ObjectPtr,
    heap: &'h Heap,
    _phantom: PhantomData<T>,
}

pub struct RootedObjectLock<'h, T> {
    ptr: ObjectPtr,
    _heap_lock: HeapLock<'h>,
    _phantom: PhantomData<T>,
}

#[derive(Debug, Clone, Copy)]
pub struct Object<'a, T> {
    ptr: ObjectPtr,
    phantom: PhantomData<&'a T>,
}

#[derive(Debug, Clone, Copy)]
pub struct Bytecode;

#[derive(Debug, Clone, Copy)]
pub struct Values;

pub trait ObjectType {
    type Element<'a>;
}

impl ObjectType for Bytecode {
    type Element<'a> = Instr;
}

impl ObjectType for Values {
    type Element<'a> = Value<'a>;
}

impl<'a, T: ObjectType> Object<'a, T> {
    pub fn get(&self, index: usize) -> T::Element<'a> {
        unsafe { self.data_base().cast::<T::Element<'a>>().add(index).read() }
    }
    pub fn set(&self, index: usize, value: T::Element<'a>) {
        unsafe {
            self.data_base().cast::<T::Element<'a>>().add(index).write(value);
        }
    }
    pub fn data_base(&self) -> *mut u8 {
        unsafe { self.as_ptr().cast::<u64>().add(1) as *mut u8 }
    }
    pub fn len(&self) -> usize {
        (unsafe { self.as_ptr().cast::<u64>().read() }) as usize
    }
    pub fn as_ptr(&self) -> *mut u8 {
        self.ptr.0
    }
    pub unsafe fn from_ptr(ptr: *mut u8) -> Object<'a, T> {
        Object {
            ptr: ObjectPtr(ptr),
            phantom: PhantomData,
        }
    }
}

impl Heap {
    pub fn new() -> Heap {
        let base = unsafe {
            mmap(
                null_mut(),
                HEAP_SIZE,
                PROT_READ | PROT_WRITE,
                MAP_ANONYMOUS | MAP_PRIVATE,
                -1,
                0,
            )
        } as *mut u8;
        if base == MAP_FAILED as *mut u8 {
            let error = Error::last_os_error();
            panic!("could not memory map heap: {}", error);
        }
        let inner = HeapInner {
            base,
            offset: Cell::new(0),
            roots: FrozenMap::new(),
        };
        Heap(RwLock::new(inner))
    }
    pub fn lock(&self) -> HeapLock<'_> {
        HeapLock {
            heap: self,
            guard: self.0.read().unwrap(),
        }
    }
}

impl<'h> HeapLock<'h> {
    fn alloc_raw(&self, mut size: usize) -> ObjectPtr {
        size += size % 8;
        if self.guard.offset.get() + size > HEAP_SIZE {
            panic!("out of memory!");
        }
        let ptr = unsafe { self.guard.base.add(self.guard.offset.get()) };
        self.guard.offset.set(self.guard.offset.get() + size);
        ObjectPtr(ptr)
    }
    pub fn root<'a, T>(&self, object: Object<'a, T>) -> RootedObject<'h, T> {
        self.guard.roots.insert(object.ptr, Box::new(AtomicU32::new(1)));
        RootedObject {
            ptr: object.ptr,
            heap: self.heap,
            _phantom: PhantomData,
        }
    }
    pub fn alloc<T: ObjectType>(&self, size: usize) -> Object<'_, T> {
        let ptr = self.alloc_raw(size_of::<u64>() + size * size_of::<T::Element<'static>>());
        unsafe { ptr.0.cast::<u64>().write(size as u64) };
        Object {
            ptr,
            phantom: PhantomData,
        }
    }
}

impl<'h, T> RootedObject<'h, T> {
    pub fn lock(&self) -> RootedObjectLock<'h, T> {
        RootedObjectLock {
            ptr: self.ptr,
            _heap_lock: self.heap.lock(),
            _phantom: PhantomData,
        }
    }
}

impl<'h, T> RootedObjectLock<'h, T> {
    pub fn get(&self) -> Object<'_, T> {
        Object {
            ptr: self.ptr,
            phantom: PhantomData,
        }
    }
}

use std::alloc::{Layout, alloc, dealloc};
use std::iter::repeat_n;
use std::marker::PhantomData;
use std::ptr::null_mut;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicU8, AtomicUsize, Ordering};
use std::{ptr, slice};

use crate::value::{Value, ValueTag};

const HEAP_ALIGN: usize = size_of::<*mut ()>();
static HEAP_EXISTS: AtomicBool = AtomicBool::new(false);

struct ObjectHeader {
    next: *mut ObjectHeader,
    tag: Tag,
    marked: bool,
}

enum Tag {
    Array,
}

pub struct Heap {
    objects_head: AtomicPtr<ObjectHeader>,
    stacks_head: Mutex<*mut StackInner>,
    mutators: AtomicUsize,
}

impl Heap {
    pub fn new() -> Option<Heap> {
        HEAP_EXISTS
            .compare_exchange(false, true, Ordering::Release, Ordering::Relaxed)
            .ok()
            .map(|_| Heap {
                objects_head: AtomicPtr::new(ptr::null_mut()),
                stacks_head: Mutex::new(null_mut()),
                mutators: AtomicUsize::new(0),
            })
    }
    pub fn mutator(&self) -> Mutator<'_> {
        self.mutators.fetch_add(1, Ordering::Acquire);
        Mutator { heap: self }
    }
    unsafe fn alloc_raw(&self, size: usize, tag: Tag) -> *mut ObjectHeader {
        unsafe {
            let size = size_of::<ObjectHeader>() + size;
            let layout = Layout::from_size_align_unchecked(size, HEAP_ALIGN);
            let ptr = alloc(layout).cast::<ObjectHeader>();
            (*ptr).tag = tag;
            (*ptr).next = self.objects_head.swap(ptr, Ordering::Relaxed);
            (*ptr).marked = false;
            ptr
        }
    }
    pub fn create_stack<'s>(&self) -> Stack<'_, 's> {
        unsafe {
            let mut stacks_head = self.stacks_head.lock().unwrap();
            let inner = alloc(Layout::new::<StackInner>()).cast::<StackInner>();
            (*inner).next = *stacks_head;
            (*inner).prev = null_mut();
            (*inner).heap = ptr::from_ref(self);
            (*inner).value_data = Vec::new();
            (*inner).value_tag = Vec::new();
            *stacks_head = inner;
            Stack {
                inner,
                _phantom: PhantomData,
            }
        }
    }
}

impl<'h> Mutator<'h> {
    pub fn alloc_array<'s>(&self, elements: &[Value<'_, 's>]) -> Object<'_, Array<'s>> {
        unsafe {
            let ptr = self.heap.alloc_raw(
                size_of::<ArrayHeader>() + size_of::<*mut ()>() * elements.len(),
                Tag::Array,
            );
            let header = ptr.add(1).cast::<ArrayHeader>();
            (*header).len = elements.len();
            let array = header.add(1).cast::<*mut ()>();
            let mut tag = ValueTag::Undefined;
            for (index, value) in elements.iter().enumerate() {
                if tag != value.tag() && tag != ValueTag::Undefined {
                    panic!("array types do not match!")
                }
                tag = value.tag();
                *array.add(index) = value.data();
            }
            (*header).tag.store(tag as u8, Ordering::Relaxed);
            Object::from_ptr(ptr.cast())
        }
    }
}

impl Drop for Heap {
    fn drop(&mut self) {
        unsafe {
            let mut cur = *self.objects_head.get_mut();
            while !cur.is_null() {
                let next = (*cur).next;
                dealloc_object(cur);
                cur = next;
            }
        }
    }
}

fn dealloc_object(ptr: *mut ObjectHeader) {
    unsafe {
        let size = match (*ptr).tag {
            Tag::Array => {
                let header = ptr.add(1).cast::<ArrayHeader>();
                size_of::<ArrayHeader>() + size_of::<*mut ()>() * (*header).len
            }
        };
        dealloc(
            ptr.cast(),
            Layout::from_size_align_unchecked(size_of::<ObjectHeader>() + size, HEAP_ALIGN),
        );
    }
}

pub struct Mutator<'h> {
    heap: &'h Heap,
}

impl<'h> Mutator<'h> {}

impl<'h> Drop for Mutator<'h> {
    fn drop(&mut self) {
        self.heap.mutators.fetch_sub(1, Ordering::Release);
    }
}

pub struct Array<'s>(PhantomData<&'s ()>);

pub struct Stack<'h, 's> {
    inner: *mut StackInner,
    _phantom: PhantomData<(&'h (), &'s ())>,
}

struct StackInner {
    heap: *const Heap,
    next: *mut StackInner,
    prev: *mut StackInner,
    value_data: Vec<*mut ()>,
    value_tag: Vec<ValueTag>,
}

impl<'h, 's> Drop for Stack<'h, 's> {
    fn drop(&mut self) {
        unsafe {
            let heap = (*self.inner).heap.as_ref_unchecked();
            let mut stacks_head = heap.stacks_head.lock().unwrap();
            let prev = (*self.inner).prev;
            let next = (*self.inner).next;
            if prev.is_null() {
                *stacks_head = next;
            } else {
                (*prev).next = next;
            }
            if !next.is_null() {
                (*next).prev = prev;
            }
            dealloc(self.inner.cast(), Layout::new::<StackInner>());
        }
    }
}

impl<'h, 's> Stack<'h, 's> {
    fn inner_mut<'m>(&mut self, _mutator: &'m Mutator<'h>) -> &mut StackInner {
        unsafe { self.inner.as_mut_unchecked() }
    }
    fn inner<'m>(&self, _mutator: &'m Mutator<'h>) -> &StackInner {
        unsafe { self.inner.as_ref_unchecked() }
    }
    pub fn grow<'m>(&mut self, size: usize, mutator: &'m Mutator<'h>) {
        let inner = self.inner_mut(mutator);
        inner.value_data.extend(repeat_n(null_mut(), size));
        inner.value_tag.extend(repeat_n(ValueTag::Undefined, size));
    }
    pub fn shrink<'m>(&mut self, size: usize, mutator: &'m Mutator<'h>) {
        let inner = self.inner_mut(mutator);
        let len = inner.value_data.len();
        inner.value_data.truncate(len - size);
        inner.value_tag.truncate(len - size);
    }
    pub fn get<'m>(&self, index: usize, mutator: &'m Mutator<'h>) -> Value<'m, 's> {
        let inner = self.inner(mutator);
        let data = inner.value_data[index];
        let tag = inner.value_tag[index];
        unsafe { Value::new(tag, data) }
    }
    pub fn set<'m>(&mut self, index: usize, value: Value, mutator: &'m Mutator<'h>) {
        let inner = self.inner_mut(mutator);
        inner.value_data[index] = value.data();
        inner.value_tag[index] = value.tag();
    }
}

struct ArrayHeader {
    len: usize,
    tag: AtomicU8,
}

pub struct Object<'m, T> {
    ptr: *mut ObjectHeader,
    _phantom: PhantomData<&'m T>,
}

impl<'m, T> Object<'m, T> {
    pub unsafe fn from_ptr(ptr: *mut ()) -> Object<'m, T> {
        Object {
            ptr: ptr.cast(),
            _phantom: PhantomData,
        }
    }
    pub fn as_ptr(&self) -> *mut () {
        self.ptr.cast()
    }
}

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
            let tag = (*self.header()).tag.load(Ordering::Relaxed);
            let tag = ValueTag::from_repr(tag).expect("invalid tag");
            if tag != value.tag() {
                panic!("array tag and value tag do not match")
            }
            self.slice()[index].store(value.data(), Ordering::Relaxed);
        }
    }
}

impl<'m, T> Clone for Object<'m, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'m, T> Copy for Object<'m, T> {}

unsafe impl<'m, T> Send for Object<'m, T> {}
unsafe impl<'m, T> Sync for Object<'m, T> {}

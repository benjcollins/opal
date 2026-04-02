use std::{
    alloc::{Layout, alloc, dealloc},
    ptr,
    sync::{
        Mutex,
        atomic::{AtomicBool, AtomicPtr, AtomicUsize, Ordering},
    },
};

use crate::{
    heap::{
        mutator::Mutator,
        object::{ArrayHeader, ObjectHeader, Tag},
        stack::{Stack, StackInner},
    },
    value::ValueTag,
};

pub mod mutator;
pub mod object;
pub mod stack;

pub struct Heap {
    objects_head: AtomicPtr<ObjectHeader>,
    stacks_head: Mutex<*mut StackInner>,
    mutators: AtomicUsize,
}

unsafe impl Sync for Heap {}
unsafe impl Send for Heap {}

pub const HEAP_ALIGN: usize = size_of::<*mut ()>();
static HEAP_EXISTS: AtomicBool = AtomicBool::new(false);

impl Heap {
    pub fn new() -> Option<Heap> {
        HEAP_EXISTS
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .ok()
            .map(|_| Heap {
                objects_head: AtomicPtr::new(ptr::null_mut()),
                stacks_head: Mutex::new(ptr::null_mut()),
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
            (*inner).prev = ptr::null_mut();
            (*inner).heap = ptr::from_ref(self);
            (*inner).value_data = vec![ptr::null_mut(); 1024];
            (*inner).value_tag = vec![ValueTag::Unit; 1024];
            *stacks_head = inner;
            Stack::new(inner)
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
            HEAP_EXISTS.store(false, Ordering::Release);
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

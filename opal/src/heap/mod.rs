pub mod mutator;
pub mod object;
pub mod stack;

use std::{
    alloc::{Layout, alloc, dealloc},
    collections::VecDeque,
    ptr,
    sync::{
        Mutex, RwLock,
        atomic::{AtomicBool, AtomicPtr, AtomicU8, Ordering},
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

struct Objects {
    head: AtomicPtr<ObjectHeader>,
}

pub struct Heap {
    objects: RwLock<Objects>,
    stacks_head: Mutex<*mut StackInner>,
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
                objects: RwLock::new(Objects {
                    head: AtomicPtr::new(ptr::null_mut()),
                }),
                stacks_head: Mutex::new(ptr::null_mut()),
            })
    }
    pub fn new_mutator(&self) -> Mutator<'_> {
        Mutator {
            heap_guard: self.objects.read().unwrap(),
        }
    }
    pub fn with_mutator<'h, T>(&'h self, mut f: impl FnMut(&Mutator<'h>) -> T) -> T {
        let mutator = self.new_mutator();
        f(&mutator)
    }
    pub fn new_stack<'s>(&self) -> Stack<'_, 's> {
        unsafe {
            let mut stacks_head = self.stacks_head.lock().unwrap();
            let inner = alloc(Layout::new::<StackInner>()).cast::<StackInner>();
            (*inner).next = *stacks_head;
            if !(*stacks_head).is_null() {
                (**stacks_head).prev = inner;
            }
            (*inner).prev = ptr::null_mut();
            (*inner).heap = ptr::from_ref(self);
            (&raw mut (*inner).value_data).write(Vec::from_iter((0..1024).map(|_| AtomicPtr::new(ptr::null_mut()))));
            (&raw mut (*inner).value_tag).write(Vec::from_iter((0..1024).map(|_| AtomicU8::new(ValueTag::Unit as u8))));
            *stacks_head = inner;
            Stack::new(inner)
        }
    }
    pub fn object_count(&self) -> usize {
        unsafe {
            let objects = self.objects.read().unwrap();
            let mut cur_object = objects.head.load(Ordering::Relaxed);
            let mut count = 0;
            while !cur_object.is_null() {
                count += 1;
                cur_object = (*cur_object).next.load(Ordering::Relaxed);
            }
            count
        }
    }
    pub fn collect_garabge(&self) {
        unsafe {
            let mut objects = self.objects.write().unwrap();
            let stacks_head = self.stacks_head.lock().unwrap();

            unmark_all_objects(&mut objects);

            let mut work_list = VecDeque::new();
            add_roots_to_work_list(*stacks_head, &mut work_list);
            trace_objects(&mut work_list);
            sweep_unreachable_objects(&mut objects);
        }
    }
}

unsafe fn unmark_all_objects(objects: &mut Objects) {
    unsafe {
        let mut cur_object = *objects.head.get_mut();
        while !cur_object.is_null() {
            (*cur_object).marked = false;
            cur_object = (*cur_object).next.load(Ordering::Relaxed);
        }
    }
}

unsafe fn add_roots_to_work_list(stacks_head: *mut StackInner, work_list: &mut VecDeque<*mut ObjectHeader>) {
    unsafe {
        let mut cur_stack = stacks_head;
        while !cur_stack.is_null() {
            let values = (*cur_stack).value_data.iter().zip((*cur_stack).value_tag.iter());

            for (value, tag) in values {
                if ValueTag::from_repr(tag.load(Ordering::Relaxed)).unwrap().is_array() {
                    let value = value.load(Ordering::Relaxed);
                    work_list.push_back(value.cast::<ObjectHeader>());
                }
            }
            cur_stack = (*cur_stack).next;
        }
    }
}

unsafe fn trace_objects(work_list: &mut VecDeque<*mut ObjectHeader>) {
    unsafe {
        while let Some(object_header) = work_list.pop_front() {
            if (*object_header).marked {
                continue;
            }
            (*object_header).marked = true;
            match (*object_header).tag {
                Tag::Array => {
                    let array_header = object_header.add(1).cast::<ArrayHeader>();
                    let value_tag = ValueTag::from_repr((*array_header).tag.load(Ordering::Relaxed)).unwrap();
                    if value_tag.is_array() {
                        let elements = array_header.add(1).cast::<*mut ()>();
                        for i in 0..(*array_header).len {
                            work_list.push_back(*elements.add(i).cast());
                        }
                    }
                }
            }
        }
    }
}

unsafe fn sweep_unreachable_objects(objects: &mut Objects) {
    unsafe {
        let mut prev_object: *mut ObjectHeader = ptr::null_mut();
        let mut cur_object = *objects.head.get_mut();
        while !cur_object.is_null() {
            let next_object = (*cur_object).next.load(Ordering::Relaxed);
            if (*cur_object).marked {
                prev_object = cur_object;
            } else {
                if !prev_object.is_null() {
                    (*prev_object)
                        .next
                        .store((*cur_object).next.load(Ordering::Relaxed), Ordering::Relaxed);
                } else {
                    *objects.head.get_mut() = (*cur_object).next.load(Ordering::Relaxed);
                }
                let size = match (*cur_object).tag {
                    Tag::Array => {
                        let header = cur_object.add(1).cast::<ArrayHeader>();
                        size_of::<ArrayHeader>() + size_of::<*mut ()>() * (*header).len
                    }
                };
                dealloc(
                    cur_object.cast(),
                    Layout::from_size_align_unchecked(size_of::<ObjectHeader>() + size, HEAP_ALIGN),
                );
            }
            cur_object = next_object;
        }
    }
}

impl Objects {
    unsafe fn alloc_raw(&self, size: usize, tag: Tag) -> *mut ObjectHeader {
        unsafe {
            let size = size_of::<ObjectHeader>() + size;
            let layout = Layout::from_size_align_unchecked(size, HEAP_ALIGN);
            let ptr = alloc(layout).cast::<ObjectHeader>();
            (*ptr).tag = tag;
            (*ptr).marked = false;
            let mut head = self.head.load(Ordering::Relaxed);
            (*ptr).next.store(head, Ordering::Relaxed);
            loop {
                match self
                    .head
                    .compare_exchange(head, ptr, Ordering::Release, Ordering::Relaxed)
                {
                    Ok(_) => break,
                    Err(new_head) => {
                        head = new_head;
                        (*ptr).next.store(head, Ordering::Relaxed);
                    }
                }
            }
            ptr
        }
    }
}

impl Drop for Heap {
    fn drop(&mut self) {
        self.collect_garabge();
        HEAP_EXISTS.store(false, Ordering::Release);
    }
}

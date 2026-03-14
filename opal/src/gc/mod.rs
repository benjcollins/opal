pub mod ref_;

use std::{
    alloc::{Layout, alloc, dealloc},
    cell::RefCell,
    collections::VecDeque,
    marker::PhantomData,
    mem::{align_of_val_raw, size_of_val_raw},
    ptr::{self, Pointee, copy, null_mut},
    sync::{Mutex, MutexGuard, Once},
};

use ref_::Gc;

#[derive(Debug, Clone, Copy)]
pub struct GcPtr(*mut u8);

#[repr(align(8))]
struct AllocHeader {
    next: GcPtr,
    size: usize,
    marked: bool,
}

pub struct GcRoot<R: Rootable> {
    ptr: *mut RootNode,
    _phantom: PhantomData<R>,
}

pub struct GlobalGc(Mutex<GcState>);

#[derive(Debug)]
pub struct WorkList {
    items: VecDeque<TraceablePtr>,
}

#[derive(Debug, Clone, Copy)]
pub struct TraceablePtr {
    ptr: GcPtr,
    trace: Option<TraceFn>,
}

impl WorkList {
    pub fn add(&mut self, ptr: TraceablePtr) {
        self.items.push_back(ptr);
    }
}

pub struct GcState {
    objects: GcPtr,
    roots: *mut RootNode,
}

unsafe impl Send for GcState {}

type TraceFn = fn(ptr: GcPtr, gc_state: &mut WorkList);

pub struct RootNode {
    prev: *mut RootNode,
    traceable_ptr: TraceablePtr,
    next: *mut RootNode,
}

pub unsafe trait Trace {
    const TRACE: bool;

    fn trace(this: &Self, work_list: &mut WorkList);
}

impl TraceablePtr {
    fn trace(&self, work_list: &mut WorkList) {
        if let Some(trace) = self.trace {
            (trace)(self.ptr, work_list)
        }
    }
}

pub trait Rootable {
    type Root<'gc>;
}

impl GcPtr {
    const fn null() -> GcPtr {
        GcPtr(null_mut())
    }
    fn is_null(self) -> bool {
        self.0.is_null()
    }
    fn header(self) -> *mut AllocHeader {
        self.0.cast::<AllocHeader>()
    }
    fn ptr_metadata<T: Pointee + ?Sized>(self) -> *mut T::Metadata {
        let (_, offset) = Layout::new::<AllocHeader>()
            .extend(Layout::new::<T::Metadata>())
            .unwrap();
        unsafe { self.0.add(offset).cast() }
    }
    fn data<T: Pointee + ?Sized>(self) -> *mut u64 {
        let (layout, _) = Layout::new::<AllocHeader>()
            .extend(Layout::new::<T::Metadata>())
            .unwrap();
        let offset = layout.align_to(HEAP_ALIGN).unwrap().pad_to_align().size();
        unsafe { self.0.add(offset).cast() }
    }
}

pub struct AllocationIter(GcPtr);

impl Iterator for AllocationIter {
    type Item = GcPtr;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.0.is_null() {
                None
            } else {
                let cur = self.0;
                self.0 = (*self.0.header()).next;
                Some(cur)
            }
        }
    }
}

pub struct RootIter(*mut RootNode);

impl Iterator for RootIter {
    type Item = *mut RootNode;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.0.is_null() {
                None
            } else {
                let cur = self.0;
                self.0 = (*self.0).next;
                Some(cur)
            }
        }
    }
}

impl GcState {
    fn allocations(&mut self) -> AllocationIter {
        AllocationIter(self.objects)
    }
    fn roots(&mut self) -> RootIter {
        RootIter(self.roots)
    }
    fn mark(&mut self) {
        unsafe {
            for ptr in self.allocations() {
                (*ptr.header()).marked = false;
            }
            let mut work_list = WorkList { items: VecDeque::new() };
            for root in self.roots() {
                work_list.items.push_back((*root).traceable_ptr);
            }
            while let Some(ptr) = work_list.items.pop_front() {
                if (*ptr.ptr.header()).marked {
                    continue;
                }
                (*ptr.ptr.header()).marked = true;
                ptr.trace(&mut work_list);
            }
        }
    }
    fn sweep(&mut self) {
        unsafe {
            let mut prev = GcPtr::null();
            for cur in self.allocations() {
                if (*cur.header()).marked {
                    prev = cur;
                    continue;
                }
                if !(*cur.header()).marked {
                    if prev.is_null() {
                        self.objects = (*cur.header()).next;
                    } else {
                        (*prev.header()).next = (*cur.header()).next;
                    }
                    let layout = Layout::from_size_align((*cur.header()).size, HEAP_ALIGN).unwrap();
                    dealloc(cur.0, layout);
                }
            }
        }
    }
}

const HEAP_ALIGN: usize = align_of::<u64>();

impl GlobalGc {
    pub fn init() -> Option<GlobalGc> {
        static INIT: Once = Once::new();
        let mut gc = None;
        INIT.call_once(|| {
            gc = Some(GlobalGc(Mutex::new(GcState {
                objects: GcPtr::null(),
                roots: null_mut(),
            })))
        });
        gc
    }
    fn lock(&self) -> MutexGuard<'_, GcState> {
        self.0.lock().unwrap()
    }
    pub fn allocation_count(&mut self) -> usize {
        let mut state = self.lock();
        state.allocations().count()
    }
    pub fn collect(&mut self) {
        let mut state = self.lock();
        state.mark();
        state.sweep();
    }
    pub unsafe fn alloc_raw<T: Trace + ?Sized>(&self, value: *const T) -> Gc<'_, T> {
        unsafe {
            assert!(align_of_val_raw(value) <= HEAP_ALIGN);
            let mut state = self.lock();
            let (src_ptr, metadata) = value.to_raw_parts();
            let layout = Layout::new::<AllocHeader>();
            let (layout, _) = layout.extend(Layout::new::<<T as Pointee>::Metadata>()).unwrap();
            let layout = layout.align_to(HEAP_ALIGN).unwrap().pad_to_align();
            let data_layout = Layout::from_size_align(size_of_val_raw(value), HEAP_ALIGN).unwrap();
            let (layout, _) = layout.extend(data_layout).unwrap();

            let dst_ptr = GcPtr(alloc(layout));
            (*dst_ptr.header()).size = size_of_val(&value);
            (*dst_ptr.header()).next = state.objects;
            state.objects = dst_ptr;

            *dst_ptr.ptr_metadata::<T>() = metadata;

            copy(
                src_ptr.cast::<u64>(),
                dst_ptr.data::<T>().cast::<u64>(),
                data_layout.pad_to_align().size() / HEAP_ALIGN,
            );

            Gc::from_ptr(dst_ptr)
        }
    }
    pub fn alloc<T: Trace>(&self, value: T) -> Gc<'_, T> {
        unsafe { self.alloc_raw(ptr::from_ref(&value)) }
    }
    pub fn alloc_slice<T: Trace + Copy + Clone + ?Sized>(&self, values: &[T]) -> Gc<'_, [T]> {
        unsafe { self.alloc_raw(ptr::from_ref(values)) }
    }
    pub fn alloc_boxed_slice<T: Trace + ?Sized>(&self, values: Box<T>) -> Gc<'_, T> {
        unsafe { self.alloc_raw(Box::as_ptr(&values)) }
    }
    pub fn alloc_str(&self, s: &str) -> Gc<'_, str> {
        unsafe { self.alloc_raw(ptr::from_ref(s)) }
    }
    pub fn get_ref<R: Rootable>(&self, root: GcRoot<R>) -> Gc<'_, R::Root<'_>> {
        unsafe { Gc::from_ptr((*root.ptr).traceable_ptr.ptr) }
    }
    pub fn root<'gc, R: Rootable>(&self, ref_: Gc<'gc, R::Root<'gc>>) -> GcRoot<R>
    where
        R::Root<'gc>: Trace,
    {
        unsafe {
            let mut state = self.lock();
            let node = alloc(Layout::new::<RootNode>()).cast::<RootNode>();
            (*node).traceable_ptr = ref_.as_traceable_ptr();
            (*node).prev = null_mut();
            (*node).next = state.roots;
            (*node).next = state.roots;
            if !state.roots.is_null() {
                (*state.roots).prev = node;
            }
            state.roots = node;
            GcRoot {
                ptr: node,
                _phantom: PhantomData,
            }
        }
    }
}

impl<R: Rootable> Drop for GcRoot<R> {
    fn drop(&mut self) {
        unsafe {
            let next = (*self.ptr).next;
            let prev = (*self.ptr).prev;
            if !next.is_null() {
                (*next).prev = prev;
            }
            if !prev.is_null() {
                (*prev).next = next;
            }
            dealloc(self.ptr.cast(), Layout::new::<RootNode>());
        }
    }
}

unsafe impl<T: Trace> Trace for Option<T> {
    const TRACE: bool = T::TRACE;

    fn trace(this: &Self, work_list: &mut WorkList) {
        if let Some(item) = this {
            Trace::trace(item, work_list);
        }
    }
}

unsafe impl<'gc, T: Trace + ?Sized> Trace for Gc<'gc, T> {
    const TRACE: bool = true;

    fn trace(this: &Self, work_list: &mut WorkList) {
        work_list.add(this.as_traceable_ptr());
    }
}

unsafe impl<'gc, T: Trace> Trace for [T] {
    const TRACE: bool = true;

    fn trace(this: &Self, work_list: &mut WorkList) {
        for elem in this {
            T::trace(elem, work_list);
        }
    }
}

unsafe impl Trace for &str {
    const TRACE: bool = false;
    fn trace(_: &Self, _: &mut WorkList) {}
}

unsafe impl Trace for i32 {
    const TRACE: bool = false;
    fn trace(_: &Self, _: &mut WorkList) {}
}

unsafe impl<T: Trace> Trace for RefCell<T> {
    const TRACE: bool = T::TRACE;

    fn trace(this: &Self, work_list: &mut WorkList) {
        T::trace(&this.borrow(), work_list);
    }
}

unsafe impl Trace for str {
    const TRACE: bool = false;
    fn trace(_: &Self, _: &mut WorkList) {}
}

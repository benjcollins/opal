use core::slice;
use std::{
    alloc::{Layout, alloc, dealloc},
    cell::RefCell,
    collections::VecDeque,
    marker::PhantomData,
    ops::Deref,
    ptr::null_mut,
    sync::{Mutex, MutexGuard, Once},
};

#[derive(Debug, Clone, Copy)]
pub struct GcPtr(*mut u8);

#[repr(align(8))]
struct AllocHeader {
    next: GcPtr,
    size: usize,
    marked: bool,
}

pub struct GcRef<'gc, T> {
    ptr: GcPtr,
    _phantom: PhantomData<&'gc T>,
}

pub struct GcSlice<'gc, T> {
    ptr: GcPtr,
    _phantom: PhantomData<&'gc T>,
}

impl<'gc, T> Clone for GcRef<'gc, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'gc, T> Clone for GcSlice<'gc, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'gc, T> Copy for GcRef<'gc, T> {}
impl<'gc, T> Copy for GcSlice<'gc, T> {}

pub struct GcRoot<R: Rootable> {
    ptr: *mut RootNode,
    _phantom: PhantomData<R>,
}

pub struct Gc(Mutex<GcState>);

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
    fn data(self) -> *mut u8 {
        unsafe { self.0.add(size_of::<AllocHeader>()) }
    }
}

impl<'gc, T: Trace> GcRef<'gc, T> {
    pub unsafe fn from_ptr(ptr: GcPtr) -> GcRef<'gc, T> {
        GcRef {
            ptr,
            _phantom: PhantomData,
        }
    }
    pub fn as_ptr(self) -> GcPtr {
        self.ptr
    }
    pub fn as_traceable_ptr(self) -> TraceablePtr {
        unsafe {
            TraceablePtr {
                ptr: self.as_ptr(),
                trace: T::TRACE.then_some(|ptr, work_list| T::trace(&GcRef::from_ptr(ptr), work_list)),
            }
        }
    }
}

impl<'gc, T: Trace> GcSlice<'gc, T> {
    pub unsafe fn from_ptr(ptr: GcPtr) -> GcSlice<'gc, T> {
        GcSlice {
            ptr,
            _phantom: PhantomData,
        }
    }
    pub fn as_ptr(self) -> GcPtr {
        self.ptr
    }
    pub fn as_traceable_ptr(self) -> TraceablePtr {
        unsafe {
            TraceablePtr {
                ptr: self.as_ptr(),
                trace: T::TRACE.then_some(|ptr, work_list| {
                    let slice = GcSlice::<T>::from_ptr(ptr);
                    for item in slice.iter() {
                        T::trace(item, work_list);
                    }
                }),
            }
        }
    }
}

impl<'gc, T> Deref for GcRef<'gc, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.data().cast::<T>().as_ref().unwrap() }
    }
}

impl<'gc, T> Deref for GcSlice<'gc, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe {
            let len_ptr = self.ptr.data().cast::<usize>();
            let data_ptr = len_ptr.add(1).cast::<T>();
            slice::from_raw_parts(data_ptr, *len_ptr)
        }
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
                    dealloc(cur.0, layout((*cur.header()).size));
                }
            }
        }
    }
}

fn layout(size: usize) -> Layout {
    Layout::from_size_align(size_of::<AllocHeader>() + size, align_of::<AllocHeader>()).unwrap()
}

impl Gc {
    pub fn init() -> Option<Gc> {
        static INIT: Once = Once::new();
        let mut gc = None;
        INIT.call_once(|| {
            gc = Some(Gc(Mutex::new(GcState {
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
    unsafe fn alloc_raw(&self, size: usize) -> GcPtr {
        unsafe {
            let mut state = self.lock();
            let ptr = GcPtr(alloc(Layout::from_size_align_unchecked(size_of::<AllocHeader>() + size, 8)).cast());
            (*ptr.header()).next = state.objects;
            (*ptr.header()).size = size;
            state.objects = ptr;
            ptr
        }
    }
    pub fn alloc<T: Trace>(&self, value: T) -> GcRef<'_, T> {
        unsafe {
            let ptr = self.alloc_raw(size_of::<T>());
            *ptr.data().cast::<T>() = value;
            GcRef::from_ptr(ptr)
        }
    }
    pub fn alloc_slice<T: Trace + Clone>(&self, values: &[T]) -> GcSlice<'_, T> {
        unsafe {
            let ptr = self.alloc_raw(size_of::<usize>() + size_of_val(values));
            *ptr.data().cast::<usize>() = values.len();
            for (index, value) in values.iter().enumerate() {
                *ptr.data().add(size_of::<usize>()).cast::<T>().add(index) = value.clone();
            }
            GcSlice::from_ptr(ptr)
        }
    }
    pub fn get_ref<R: Rootable>(&self, root: GcRoot<R>) -> GcRef<'_, R::Root<'_>> {
        unsafe {
            GcRef {
                ptr: (*root.ptr).traceable_ptr.ptr,
                _phantom: PhantomData,
            }
        }
    }
    pub fn root<'gc, R: Rootable>(&self, ref_: GcRef<'gc, R::Root<'gc>>) -> GcRoot<R>
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

unsafe impl<'gc, T: Trace> Trace for GcRef<'gc, T> {
    const TRACE: bool = true;

    fn trace(this: &Self, work_list: &mut WorkList) {
        work_list.add(this.as_traceable_ptr());
    }
}

unsafe impl<'gc, T: Trace> Trace for GcSlice<'gc, T> {
    const TRACE: bool = true;

    fn trace(this: &Self, work_list: &mut WorkList) {
        work_list.add(this.as_traceable_ptr());
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

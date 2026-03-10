use std::{
    alloc::{Layout, alloc, dealloc},
    marker::PhantomData,
    ops::Deref,
    ptr::null_mut,
    sync::{Mutex, MutexGuard},
};

#[derive(Debug, Clone, Copy)]
pub struct GcPtr(*mut ());

unsafe impl Send for GcPtr {}

#[repr(align(8))]
struct GcHeader {
    next: GcPtr,
}

pub struct GcRef<'m, T: Trace> {
    ptr: GcPtr,
    _phantom: PhantomData<&'m T>,
}

impl<'m, T: Trace> Clone for GcRef<'m, T> {
    fn clone(&self) -> Self {
        GcRef {
            ptr: self.ptr,
            _phantom: PhantomData,
        }
    }
}

impl<'m, T: Trace> Copy for GcRef<'m, T> {}

pub struct GcRoot<R: Rootable> {
    ptr: RootNodePtr,
    _phantom: PhantomData<R>,
}

#[derive(Debug, Clone, Copy)]
struct RootNodePtr(*mut RootNode);

unsafe impl Send for RootNodePtr {}

pub struct Gc(Mutex<GcState>);

struct GcState {
    objects: GcPtr,
    roots: RootNodePtr,
}

struct RootNode {
    prev: RootNodePtr,
    gc_ptr: GcPtr,
    next: RootNodePtr,
}

pub unsafe trait Trace {
    fn trace(&self);
}

pub trait Rootable {
    type Root<'a>: Trace;
}

impl GcPtr {
    const fn null() -> GcPtr {
        GcPtr(null_mut())
    }
    fn header(self) -> *mut GcHeader {
        self.0.cast::<GcHeader>()
    }
    fn data(self) -> *mut () {
        unsafe { self.0.offset(size_of::<GcHeader>() as isize) }
    }
}

impl RootNodePtr {
    const fn null() -> RootNodePtr {
        RootNodePtr(null_mut())
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
}

impl<'gc, T: Trace> Deref for GcRef<'gc, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.data().cast::<T>().as_ref().unwrap() }
    }
}

impl Gc {
    pub fn init() -> Gc {
        static GC: Mutex<Option<Gc>> = Mutex::new(Some(Gc(Mutex::new(GcState {
            objects: GcPtr::null(),
            roots: RootNodePtr::null(),
        }))));
        GC.lock().unwrap().take().expect("already acquired the global gc")
    }
    fn lock(&self) -> MutexGuard<'_, GcState> {
        self.0.lock().unwrap()
    }
    pub fn collect(&mut self) {
        println!("currently does nothing!")
    }
    pub fn alloc<T: Trace>(&self, value: T) -> GcRef<'_, T> {
        unsafe {
            let mut state = self.lock();
            let layout =
                Layout::from_size_align(size_of::<GcHeader>() + size_of::<T>(), align_of::<GcHeader>()).unwrap();
            let ptr = GcPtr(alloc(layout).cast());
            (*ptr.header()).next = state.objects;
            *ptr.data().cast::<T>() = value;
            state.objects = ptr;
            GcRef::from_ptr(ptr)
        }
    }
    pub fn get_ref<R: Rootable>(&self, root: GcRoot<R>) -> GcRef<'_, R::Root<'_>> {
        unsafe {
            GcRef {
                ptr: (*root.ptr.0).gc_ptr,
                _phantom: PhantomData,
            }
        }
    }
    pub fn root<R: Rootable>(&self, ref_: GcRef<'_, R::Root<'_>>) -> GcRoot<R> {
        unsafe {
            let mut state = self.lock();
            let node = alloc(Layout::new::<RootNode>()).cast::<RootNode>();
            (*node).gc_ptr = ref_.ptr;
            (*node).prev = RootNodePtr::null();
            (*node).next = state.roots;
            (*node).next = state.roots;
            if !state.roots.0.is_null() {
                (*state.roots.0).prev = RootNodePtr(node);
            }
            state.roots = RootNodePtr(node);
            GcRoot {
                ptr: RootNodePtr(node),
                _phantom: PhantomData,
            }
        }
    }
}

impl<R: Rootable> Drop for GcRoot<R> {
    fn drop(&mut self) {
        unsafe {
            let next = (*self.ptr.0).next;
            let prev = (*self.ptr.0).prev;
            if !next.0.is_null() {
                (*next.0).prev = prev;
            }
            if !prev.0.is_null() {
                (*prev.0).next = next;
            }
            dealloc(self.ptr.0.cast(), Layout::new::<RootNode>());
        }
    }
}

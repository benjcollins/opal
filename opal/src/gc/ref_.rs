use std::{
    marker::PhantomData,
    ops::Deref,
    ptr::{self, Pointee},
};

use super::{GcPtr, Trace, TraceablePtr};

pub struct Gc<'gc, T: ?Sized> {
    ptr: GcPtr,
    _phantom: PhantomData<&'gc T>,
}

impl<'gc, T: ?Sized> Clone for Gc<'gc, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'gc, T: ?Sized> Copy for Gc<'gc, T> {}

impl<'gc, T: Trace + ?Sized> Gc<'gc, T> {
    pub fn as_ptr(self) -> GcPtr {
        self.ptr
    }
    pub fn as_traceable_ptr(self) -> TraceablePtr {
        unsafe {
            TraceablePtr {
                ptr: self.as_ptr(),
                trace: T::TRACE.then_some(|ptr, work_list| T::trace(&Gc::from_ptr(ptr), work_list)),
            }
        }
    }
}

impl<'gc, T: ?Sized> Gc<'gc, T> {
    pub unsafe fn from_ptr(ptr: GcPtr) -> Gc<'gc, T> {
        Gc {
            ptr,
            _phantom: PhantomData,
        }
    }
}

impl<'gc, T: ?Sized + Pointee> Deref for Gc<'gc, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            ptr::from_raw_parts::<T>(self.ptr.data::<T>(), *self.ptr.ptr_metadata::<T>())
                .as_ref()
                .unwrap()
        }
    }
}

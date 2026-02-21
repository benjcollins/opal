use std::{convert::Infallible, marker::PhantomData, mem::transmute, ptr};

use crate::{
    heap::{HeapObject, ObjectHeader},
    infer::Type,
    lower::CompiledFun,
    vm::{Fun, RuntimeError},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Value<'f>(u64, PhantomData<&'f ()>);

pub trait ValueConv {
    const TYPE: Type;

    fn into_value<'h>(self) -> Value<'h>;
    fn from_value<'h>(value: Value<'h>) -> Self;
}

pub trait NativeFunResult {
    type Output: ValueConv;
    fn map<'h>(self) -> Result<Value<'h>, RuntimeError>;
}

impl<T: ValueConv> NativeFunResult for Result<T, RuntimeError> {
    type Output = T;
    fn map<'h>(self) -> Result<Value<'h>, RuntimeError> {
        self.map(|t| t.into_value())
    }
}

impl ValueConv for i64 {
    const TYPE: Type = Type::Int;
    fn into_value<'h>(self) -> Value<'h> {
        Value::from_int(self)
    }
    fn from_value<'h>(value: Value<'h>) -> Self {
        value.as_int()
    }
}

impl ValueConv for bool {
    const TYPE: Type = Type::Bool;
    fn into_value<'h>(self) -> Value<'h> {
        Value::from_bool(self)
    }
    fn from_value<'h>(value: Value<'h>) -> Self {
        value.as_bool()
    }
}

impl ValueConv for f64 {
    const TYPE: Type = Type::Float;
    fn into_value<'h>(self) -> Value<'h> {
        Value::from_float(self)
    }
    fn from_value<'h>(value: Value<'h>) -> Self {
        value.as_float()
    }
}

impl ValueConv for () {
    const TYPE: Type = Type::Unit;
    fn into_value<'h>(self) -> Value<'h> {
        Value::from_unit(())
    }
    fn from_value<'h>(_: Value<'h>) -> Self {}
}

impl ValueConv for Infallible {
    const TYPE: Type = Type::Void;

    fn into_value<'h>(self) -> Value<'h> {
        match self {}
    }

    fn from_value<'h>(_: Value<'h>) -> Self {
        unreachable!()
    }
}

impl<'f> Value<'f> {
    pub fn from_unit(_: ()) -> Value<'f> {
        Value(0, PhantomData)
    }
    pub fn from_int(value: i64) -> Value<'f> {
        Value(value as u64, PhantomData)
    }
    pub fn from_float(value: f64) -> Value<'f> {
        Value(value.to_bits(), PhantomData)
    }
    pub fn from_bool(value: bool) -> Value<'f> {
        Value(value as u64, PhantomData)
    }
    pub fn from_fun(fun: Fun) -> Value<'f> {
        match fun {
            Fun::Native(fun) => Value(fun as usize as u64 | 1 << 63, PhantomData),
            Fun::Compiled(fun) => Value(ptr::from_ref(fun) as u64, PhantomData),
        }
    }
    pub fn from_object(object: HeapObject<'f>) -> Value<'f> {
        Value(object.ptr.addr() as u64, PhantomData)
    }
    pub fn as_float(self) -> f64 {
        f64::from_bits(self.0)
    }
    pub fn as_int(self) -> i64 {
        self.0 as i64
    }
    pub fn as_bool(self) -> bool {
        self.0 != 0
    }
    pub unsafe fn as_fun(self) -> Fun<'f> {
        unsafe {
            if self.0 >> 63 == 1 {
                let ptr = self.0 & !(1 << 63);
                Fun::Native(transmute(ptr as *const ()))
            } else {
                Fun::Compiled((self.0 as *const CompiledFun).as_ref().unwrap())
            }
        }
    }
    pub unsafe fn as_object(self) -> HeapObject<'f> {
        HeapObject {
            ptr: self.0 as *mut ObjectHeader,
            phantom: PhantomData,
        }
    }
    // pub unsafe fn as_array(self) -> &'f [Value<'f>] {
    //     let ptr = self.0 as *const u64;
    //     unsafe { slice::from_raw_parts(ptr.add(1) as *const Value<'f>, ptr::read(ptr) as usize) }
    // }
}

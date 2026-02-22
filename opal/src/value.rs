use std::{convert::Infallible, marker::PhantomData, mem::transmute, ptr};

use crate::{
    heap::{ArrayObject, HeapObject, ObjectHeader},
    lower::CompiledFun,
    ty::BorrowedType,
    vm::{Fun, RuntimeError},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Value<'f>(u64, PhantomData<&'f ()>);

pub struct Array<'h, T> {
    object: ArrayObject<'h>,
    phantom: PhantomData<T>,
}

pub trait ValueConv<'h> {
    const TYPE: BorrowedType<'static>;

    fn into_value(self) -> Value<'h>;
    fn from_value(value: Value<'h>) -> Self;
}

pub trait NativeFunResult<'h> {
    type Output: ValueConv<'h>;
    fn map(self) -> Result<Value<'h>, RuntimeError>;
}

impl<'h, T: ValueConv<'h>> NativeFunResult<'h> for Result<T, RuntimeError> {
    type Output = T;
    fn map(self) -> Result<Value<'h>, RuntimeError> {
        self.map(|t| t.into_value())
    }
}

impl<'h, T: ValueConv<'h>> ValueConv<'h> for Array<'h, T> {
    const TYPE: BorrowedType<'static> = BorrowedType::Array(&T::TYPE);

    fn into_value(self) -> Value<'h> {
        Value::from_object(self.object.heap_object())
    }

    fn from_value(value: Value<'h>) -> Self {
        unsafe {
            Array {
                object: value.as_object().as_array().unwrap(),
                phantom: PhantomData,
            }
        }
    }
}

impl<'h> ValueConv<'h> for i64 {
    const TYPE: BorrowedType<'static> = BorrowedType::Int;
    fn into_value(self) -> Value<'h> {
        Value::from_int(self)
    }
    fn from_value(value: Value<'h>) -> Self {
        value.as_int()
    }
}

impl<'h> ValueConv<'h> for bool {
    const TYPE: BorrowedType<'static> = BorrowedType::Bool;
    fn into_value(self) -> Value<'h> {
        Value::from_bool(self)
    }
    fn from_value(value: Value<'h>) -> Self {
        value.as_bool()
    }
}

impl<'h> ValueConv<'h> for f64 {
    const TYPE: BorrowedType<'static> = BorrowedType::Float;
    fn into_value(self) -> Value<'h> {
        Value::from_float(self)
    }
    fn from_value(value: Value<'h>) -> Self {
        value.as_float()
    }
}

impl<'h> ValueConv<'h> for () {
    const TYPE: BorrowedType<'static> = BorrowedType::Unit;
    fn into_value(self) -> Value<'h> {
        Value::from_unit(())
    }
    fn from_value(_: Value<'h>) -> Self {}
}

impl<'h> ValueConv<'h> for Infallible {
    const TYPE: BorrowedType<'static> = BorrowedType::Void;

    fn into_value(self) -> Value<'h> {
        match self {}
    }

    fn from_value(_: Value<'h>) -> Self {
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
}

impl<'h, T: ValueConv<'h>> Array<'h, T> {
    pub fn len(&self) -> i64 {
        self.object.len() as i64
    }
    pub fn get(&self, index: i64) -> T {
        T::from_value(self.object.get(index as u64))
    }
    pub fn set(&self, index: i64, value: T) {
        self.object.set(index as u64, value.into_value());
    }
}

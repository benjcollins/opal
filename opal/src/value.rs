use std::{convert::Infallible, marker::PhantomData, mem::transmute, ptr};

use crate::{
    heap::{ArrayObject, HeapObject, ObjectHeader},
    lower::CompiledFun,
    ty::{BorrowedType, NumericType},
    vm::{Fun, RuntimeError},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Value<'f>(*mut (), PhantomData<&'f ()>);

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

impl<'h> ValueConv<'h> for Int {
    const TYPE: BorrowedType<'static> = BorrowedType::Numeric(NumericType::Int);
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

impl<'h> ValueConv<'h> for Float {
    const TYPE: BorrowedType<'static> = BorrowedType::Numeric(NumericType::Float);
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

#[cfg(target_pointer_width = "64")]
pub type Int = i64;
#[cfg(target_pointer_width = "32")]
pub type Int = i32;

#[cfg(target_pointer_width = "64")]
pub type UInt = u64;
#[cfg(target_pointer_width = "32")]
pub type UInt = u32;

#[cfg(target_pointer_width = "64")]
pub type Float = f64;
#[cfg(target_pointer_width = "32")]
pub type Float = f32;

const NATIVE_FUN_BIT: usize = 1;

impl<'f> Value<'f> {
    fn new(ptr: *mut ()) -> Value<'f> {
        Value(ptr, PhantomData)
    }
    pub fn from_unit(_: ()) -> Value<'f> {
        Value::new(ptr::null_mut())
    }
    pub fn from_int(value: Int) -> Value<'f> {
        Value::new(ptr::without_provenance_mut(value as usize))
    }
    pub fn from_float(value: Float) -> Value<'f> {
        Value::new(ptr::without_provenance_mut(value.to_bits() as usize))
    }
    pub fn from_bool(value: bool) -> Value<'f> {
        Value::new(ptr::without_provenance_mut(value as usize))
    }
    pub fn from_fun(fun: Fun) -> Value<'f> {
        match fun {
            Fun::Native(fun) => Value::new((fun as *mut ()).map_addr(|addr| addr | NATIVE_FUN_BIT)),
            Fun::Compiled(fun) => Value::new(ptr::from_ref(fun) as *mut ()),
        }
    }
    pub fn from_object(object: HeapObject<'f>) -> Value<'f> {
        Value::new(object.ptr as *mut ())
    }
    pub fn as_float(self) -> Float {
        Float::from_bits(self.0.addr() as UInt)
    }
    pub fn as_int(self) -> Int {
        self.0.addr() as Int
    }
    pub fn as_bool(self) -> bool {
        self.0.addr() != 0
    }
    pub unsafe fn as_fun(self) -> Fun<'f> {
        unsafe {
            if self.0.addr() & NATIVE_FUN_BIT != 0 {
                let ptr = self.0.map_addr(|addr| addr & !NATIVE_FUN_BIT);
                Fun::Native(transmute(ptr))
            } else {
                Fun::Compiled((self.0 as *const CompiledFun).as_ref().unwrap())
            }
        }
    }
    pub unsafe fn as_object(self) -> HeapObject<'f> {
        unsafe { HeapObject::new(self.0 as *mut ObjectHeader) }
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

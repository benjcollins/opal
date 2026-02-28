use std::{convert::Infallible, marker::PhantomData, mem::transmute, ptr};

use crate::{
    heap::{ArrayObject, ObjectRef},
    lower::CompiledFun,
    ty::{BorrowedType, NumericType},
    vm::{Fun, RuntimeError},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Value(*mut ());

pub struct Array<'a, T> {
    object: ObjectRef<'a, ArrayObject>,
    phantom: PhantomData<T>,
}

pub trait ValueConv {
    const TYPE: BorrowedType<'static>;

    fn into_value(self) -> Value;
    fn from_value(value: Value) -> Self;
}

pub trait NativeFunResult {
    type Output: ValueConv;
    fn map(self) -> Result<Value, RuntimeError>;
}

impl<'h, T: ValueConv> NativeFunResult for Result<T, RuntimeError> {
    type Output = T;
    fn map(self) -> Result<Value, RuntimeError> {
        self.map(|t| t.into_value())
    }
}

impl<'a, T: ValueConv> ValueConv for Array<'a, T> {
    const TYPE: BorrowedType<'static> = BorrowedType::Array(&T::TYPE);
    fn into_value(self) -> Value {
        todo!()
    }
    fn from_value(value: Value) -> Self {
        todo!()
    }
}

impl<'h> ValueConv for Int {
    const TYPE: BorrowedType<'static> = BorrowedType::Numeric(NumericType::Int);
    fn into_value(self) -> Value {
        Value::from_int(self)
    }
    fn from_value(value: Value) -> Self {
        value.as_int()
    }
}

impl ValueConv for bool {
    const TYPE: BorrowedType<'static> = BorrowedType::Bool;
    fn into_value(self) -> Value {
        Value::from_bool(self)
    }
    fn from_value(value: Value) -> Self {
        value.as_bool()
    }
}

impl<'h> ValueConv for Float {
    const TYPE: BorrowedType<'static> = BorrowedType::Numeric(NumericType::Float);
    fn into_value(self) -> Value {
        Value::from_float(self)
    }
    fn from_value(value: Value) -> Self {
        value.as_float()
    }
}

impl<'h> ValueConv for () {
    const TYPE: BorrowedType<'static> = BorrowedType::Unit;
    fn into_value(self) -> Value {
        Value::from_unit(())
    }
    fn from_value(_: Value) -> Self {}
}

impl<'h> ValueConv for Infallible {
    const TYPE: BorrowedType<'static> = BorrowedType::Void;

    fn into_value(self) -> Value {
        match self {}
    }

    fn from_value(_: Value) -> Self {
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

impl Value {
    fn new(data: *mut ()) -> Value {
        Value(data)
    }
    pub fn from_unit(_: ()) -> Value {
        Value::new(ptr::null_mut())
    }
    pub fn from_int(value: Int) -> Value {
        Value::new(ptr::without_provenance_mut(value as usize))
    }
    pub fn from_float(value: Float) -> Value {
        Value::new(ptr::without_provenance_mut(value.to_bits() as usize))
    }
    pub fn from_bool(value: bool) -> Value {
        Value::new(ptr::without_provenance_mut(value as usize))
    }
    pub fn from_fun(fun: Fun) -> Value {
        match fun {
            Fun::Native(fun) => Value::new((fun as *mut ()).map_addr(|addr| addr | NATIVE_FUN_BIT)),
            Fun::Compiled(fun) => Value::new(ptr::from_ref(fun) as *mut ()),
        }
    }
    // pub fn from_object(object: HeapObject) -> Value {
    //     Value::new(object.ptr as *mut ())
    // }
    pub fn as_float(self) -> Float {
        Float::from_bits(self.0.addr() as UInt)
    }
    pub fn as_int(self) -> Int {
        self.0.addr() as Int
    }
    pub fn as_bool(self) -> bool {
        self.0.addr() != 0
    }
    pub unsafe fn as_fun(self) -> Fun {
        unsafe {
            if self.0.addr() & NATIVE_FUN_BIT != 0 {
                let ptr = self.0.map_addr(|addr| addr & !NATIVE_FUN_BIT);
                Fun::Native(transmute(ptr))
            } else {
                Fun::Compiled((self.0 as *const CompiledFun).as_ref().unwrap())
            }
        }
    }
    // pub unsafe fn as_object(self) -> HeapObject<'f> {
    //     unsafe { HeapObject::new(self.0 as *mut ObjectHeader) }
    // }
}

// impl<'h, T: ValueConv> Array<'h, T> {
//     pub fn len(&self) -> i64 {
//         self.object.len() as i64
//     }
//     pub fn get(&self, index: i64) -> T {
//         T::from_value(self.object.get(index as u64))
//     }
//     pub fn set(&self, index: i64, value: T) {
//         self.object.set(index as u64, value.into_value());
//     }
// }

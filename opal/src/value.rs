use std::{convert::Infallible, marker::PhantomData, mem::transmute, ptr};

use strum::{EnumIs, FromRepr};

use crate::{
    heap::{Bytecode, Object, ObjectType, Values},
    runtime::NativeFun,
    ty::{BorrowedType, NumericType},
    vm::RuntimeError,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Value<'a>(*mut (), PhantomData<&'a ()>);

pub struct Array<'a, T> {
    object: Object<'a, Values>,
    phantom: PhantomData<T>,
}

pub trait ValueConv<'a> {
    const TYPE: BorrowedType<'static>;

    fn into_value(self) -> Value<'a>;
    fn from_value(value: Value<'a>) -> Self;
}

pub trait NativeFunResult<'a> {
    type Output: ValueConv<'a>;
    fn map(self) -> Result<Value<'a>, RuntimeError>;
}

impl<'a, T: ValueConv<'a>> NativeFunResult<'a> for Result<T, RuntimeError> {
    type Output = T;
    fn map(self) -> Result<Value<'a>, RuntimeError> {
        self.map(|t| t.into_value())
    }
}

impl<'a, T: ValueConv<'a>> ValueConv<'a> for Array<'a, T> {
    const TYPE: BorrowedType<'static> = BorrowedType::Array(&T::TYPE);
    fn into_value(self) -> Value<'a> {
        Value::from_object(self.object)
    }
    fn from_value(value: Value<'a>) -> Self {
        Array {
            object: unsafe { value.as_object() },
            phantom: PhantomData,
        }
    }
}

impl<'a> ValueConv<'a> for i64 {
    const TYPE: BorrowedType<'static> = BorrowedType::Numeric(NumericType::Int);
    fn into_value(self) -> Value<'a> {
        Value::from_int(self)
    }
    fn from_value(value: Value<'a>) -> Self {
        value.as_int()
    }
}

impl<'a> ValueConv<'a> for bool {
    const TYPE: BorrowedType<'static> = BorrowedType::Bool;
    fn into_value(self) -> Value<'a> {
        Value::from_bool(self)
    }
    fn from_value(value: Value<'a>) -> Self {
        value.as_bool()
    }
}

impl<'a> ValueConv<'a> for f64 {
    const TYPE: BorrowedType<'static> = BorrowedType::Numeric(NumericType::Float);
    fn into_value(self) -> Value<'a> {
        Value::from_float(self)
    }
    fn from_value(value: Value<'a>) -> Self {
        value.as_float()
    }
}

impl<'a> ValueConv<'a> for () {
    const TYPE: BorrowedType<'static> = BorrowedType::Unit;
    fn into_value(self) -> Value<'a> {
        Value::from_unit(())
    }
    fn from_value(_: Value<'a>) -> Self {}
}

impl<'a> ValueConv<'a> for Infallible {
    const TYPE: BorrowedType<'static> = BorrowedType::Void;

    fn into_value(self) -> Value<'a> {
        match self {}
    }

    fn from_value(_: Value<'a>) -> Self {
        unreachable!()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, EnumIs, FromRepr)]
#[repr(u64)]
enum PointerTag {
    NativeFun,
    ObjectValues,
    ObjectBytecode,
}

const ADDR_MASK: u64 = 0xffffffffffff;

const POINTER_PREFIX: u64 = 0x7ffc << 48;
const POINTER_PREFIX_MASK: u64 = 0xfffc << 48;

const META_TAG_MASK: u64 = 0x3 << 48;

fn is_pointer(value: u64) -> bool {
    value & POINTER_PREFIX_MASK == POINTER_PREFIX
}

enum Pointer<'a> {
    NativeFun(NativeFun),
    ObjectValues(Object<'a, Values>),
    ObjectBytecode(Object<'a, Bytecode>),
}

impl<'a> Value<'a> {
    fn new(ptr: *mut ()) -> Value<'a> {
        Value(ptr, PhantomData)
    }
    pub fn from_unit(_: ()) -> Value<'a> {
        Value::new(ptr::null_mut())
    }
    pub fn from_int(value: i64) -> Value<'a> {
        if has_pointer_prefix(value as u64) {
            panic!("integer int value")
        }
        Value::new(ptr::without_provenance_mut(value as usize))
    }
    pub fn from_float(value: f64) -> Value<'a> {
        let bits = value.to_bits();
        if has_pointer_prefix(bits) {
            panic!("invalid float value")
        }
        Value::new(ptr::without_provenance_mut(value.to_bits() as usize))
    }
    pub fn from_bool(value: bool) -> Value<'a> {
        Value::new(ptr::without_provenance_mut(value as usize))
    }
    // pub fn from_native_fun(fun: NativeFun) -> Value<'a> {
    //     Value::new(fun as *mut ())
    // }
    // pub fn from_object<T: ObjectType>(object: Object<'a, T>) -> Value<'a> {
    //     Value::new(object.as_ptr().map_addr(|addr| addr | (POINTER_PREFIX as usize)).cast())
    // }
    pub fn as_float(self) -> f64 {
        f64::from_bits(self.0.addr() as u64)
    }
    pub fn as_int(self) -> i64 {
        self.0.addr() as i64
    }
    pub fn as_bool(self) -> bool {
        self.0.addr() != 0
    }
    fn from_pointer(ptr: Pointer<'a>) -> Value<'a> {
        todo!()
    }
    fn as_pointer(self) -> Pointer<'a> {
        if !is_pointer(self.0 as u64) {
            panic!("invalid pointer value")
        }
        let tag = PointerTag::from_repr(((self.0.addr() as u64) & META_TAG_MASK) >> 48).unwrap();
        let ptr = self.0.map_addr(|addr| addr & ADDR_MASK as usize);
        unsafe {
            match tag {
                PointerTag::NativeFun => Pointer::NativeFun(transmute::<*mut (), NativeFun>(ptr)),
                PointerTag::ObjectValues => Pointer::ObjectValues(Object::from_ptr(ptr as *mut u8)),
                PointerTag::ObjectBytecode => Pointer::ObjectBytecode(Object::from_ptr(ptr as *mut u8)),
            }
        }
    }
    // pub fn as_native_fun(self) -> NativeFun {
    //     let (ptr, flags) = self.as_pointer();
    //     if !flags.is_native_fun {
    //         panic!("this is not a native function")
    //     }
    //     unsafe { transmute::<*mut u8, NativeFun>(ptr) }
    // }
    // pub fn as_object<T: ObjectType>(self) -> Object<'a, T> {
    //     if !self.is_object() {
    //         panic!("invalid object")
    //     }
    //     unsafe { Object::from_ptr(self.0.map_addr(|addr| addr & !(POINTER_PREFIX as usize)) as *mut u8) }
    // }
    // pub fn is_object(self) -> bool {
    //     (self.0.addr() as u64) & POINTER_PREFIX_MASK == POINTER_PREFIX
    // }
}

impl<'a, T: ValueConv<'a>> Array<'a, T> {
    pub fn len(&self) -> i64 {
        self.object.len() as i64
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn get(&self, index: i64) -> T {
        T::from_value(self.object.get(index as usize))
    }
    pub fn set(&self, index: i64, value: T) {
        self.object.set(index as usize, value.into_value());
    }
}

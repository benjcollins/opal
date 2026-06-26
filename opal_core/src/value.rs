use std::{marker::PhantomData, mem, ptr};

use crate::bytecode::Fun;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Value<'a>(*const (), PhantomData<&'a Fun<'a>>);

#[cfg(target_pointer_width = "32")]
pub type Int = i32;
#[cfg(target_pointer_width = "32")]
pub type Float = f32;
#[cfg(target_pointer_width = "32")]
pub type UnsignedInt = u32;

#[cfg(target_pointer_width = "64")]
pub type Int = i64;
#[cfg(target_pointer_width = "64")]
pub type Float = f64;
#[cfg(target_pointer_width = "64")]
pub type UnsignedInt = u64;

type RustFun = for<'a> fn(&[Value<'a>]) -> Value<'a>;

pub enum FunValue<'a> {
    Rust(RustFun),
    Opal(&'a Fun<'a>),
    Null,
}

impl<'a> Value<'a> {
    const fn new(ptr: *const ()) -> Value<'a> {
        Value(ptr, PhantomData)
    }

    pub const UNIT: Value<'a> = Value::new(ptr::without_provenance(0));

    pub fn from_int(value: Int) -> Value<'a> {
        Value::new(ptr::without_provenance(value as usize))
    }
    pub fn from_float(value: Float) -> Value<'a> {
        Value::new(ptr::without_provenance(value.to_bits() as usize))
    }
    pub fn from_bool(value: bool) -> Value<'a> {
        Value::new(ptr::without_provenance(if value { 1 } else { 0 }))
    }
    pub fn from_fun(fun: FunValue<'a>) -> Value<'a> {
        Value::new(match fun {
            FunValue::Rust(fun_ptr) => fun_ptr as *const (),
            FunValue::Opal(fun) => ptr::from_ref(fun).map_addr(|addr| addr | 1).cast(),
            FunValue::Null => ptr::without_provenance(0),
        })
    }

    pub fn as_int(self) -> Int {
        self.0.addr() as Int
    }
    pub fn as_float(self) -> Float {
        Float::from_bits(self.0.addr() as UnsignedInt)
    }
    pub fn as_bool(self) -> bool {
        self.0.addr() == 1
    }
    pub fn as_fun(self) -> FunValue<'a> {
        unsafe {
            if self.0.addr() == 0 {
                FunValue::Null
            } else if self.0.addr() & 1 == 1 {
                FunValue::Opal(self.0.cast::<Fun>().as_ref().unwrap())
            } else {
                FunValue::Rust(mem::transmute::<*const (), RustFun>(self.0))
            }
        }
    }
}

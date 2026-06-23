use std::ptr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Value(*const ());

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

impl Value {
    pub const UNIT: Value = Value(ptr::without_provenance(0));

    pub fn from_int(value: Int) -> Value {
        Value(ptr::without_provenance(value as usize))
    }
    pub fn from_float(value: Float) -> Value {
        Value(ptr::without_provenance(value.to_bits() as usize))
    }
    pub fn from_bool(value: bool) -> Value {
        Value(ptr::without_provenance(if value { 1 } else { 0 }))
    }

    pub fn as_int(self) -> Int {
        self.0 as usize as Int
    }
    pub fn as_float(self) -> Float {
        Float::from_bits(self.0 as usize as UnsignedInt)
    }
}

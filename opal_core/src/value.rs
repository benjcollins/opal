use std::ptr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Value(*const ());

#[cfg(target_pointer_width = "32")]
type Int = i32;
#[cfg(target_pointer_width = "32")]
type Float = f32;

#[cfg(target_pointer_width = "64")]
type Int = i64;
#[cfg(target_pointer_width = "64")]
type Float = f64;

impl Value {
    pub const UNIT: Value = Value(ptr::without_provenance(0));

    pub fn int(value: Int) -> Value {
        Value(ptr::without_provenance(value as usize))
    }
    pub fn float(value: Float) -> Value {
        Value(ptr::without_provenance(value.to_bits() as usize))
    }
    pub fn bool(value: bool) -> Value {
        Value(ptr::without_provenance(if value { 1 } else { 0 }))
    }
}

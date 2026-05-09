use crate::heap::object::{Object, ObjectTrait};

#[derive(Debug, Clone, Copy)]
pub struct Bytes {
    pub(super) len: usize,
}

impl ObjectTrait for Bytes {
    type Item = u8;

    fn size(&self) -> usize {
        self.len
    }
}

impl Object<Bytes> {
    pub fn as_bytes(&self) -> &[u8] {
        self.extended()
    }
}

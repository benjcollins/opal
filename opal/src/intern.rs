use std::{
    alloc::{Layout, alloc, dealloc},
    fmt,
    ptr::copy,
    str,
    sync::{
        LazyLock,
        atomic::{AtomicU32, AtomicUsize, Ordering, fence},
    },
};

use dashmap::{DashMap, Entry};

struct Interner {
    id_to_entry: DashMap<u32, StringEntry>,
    str_to_id: DashMap<String, u32>,
    next_id: AtomicU32,
}

#[derive(Debug)]
struct StringEntry {
    count: AtomicUsize,
    ptr: *mut u8,
    len: usize,
}

unsafe impl Send for StringEntry {}
unsafe impl Sync for StringEntry {}

#[derive(PartialEq, Eq, Hash)]
pub struct InternedStr(u32);

static INTERNER: LazyLock<Interner> = LazyLock::new(|| Interner {
    id_to_entry: DashMap::new(),
    str_to_id: DashMap::new(),
    next_id: AtomicU32::new(0),
});

impl InternedStr {
    pub fn new(string: impl Into<String>) -> InternedStr {
        InternedStr(match INTERNER.str_to_id.entry(string.into()) {
            Entry::Occupied(entry) => {
                let id = *entry.get();
                let entry = INTERNER.id_to_entry.get(&id).unwrap();
                entry.count.fetch_add(1, Ordering::Relaxed);
                id
            }
            Entry::Vacant(entry) => {
                let id = INTERNER.next_id.fetch_add(1, Ordering::Relaxed);
                let len = entry.key().len();
                let count = AtomicUsize::new(1);
                unsafe {
                    let ptr = alloc(Layout::from_size_align_unchecked(len, 1));
                    copy(entry.key().as_ptr(), ptr, len);
                    INTERNER.id_to_entry.insert(id, StringEntry { count, ptr, len });
                }
                entry.insert(id);
                id
            }
        })
    }
    pub fn as_str(&self) -> &str {
        let entry = INTERNER.id_to_entry.get(&self.0).unwrap();
        unsafe { str::from_raw_parts(entry.ptr, entry.len) }
    }
    pub fn id(&self) -> u32 {
        self.0
    }
}

impl Clone for InternedStr {
    fn clone(&self) -> Self {
        let entry = INTERNER.id_to_entry.get(&self.0).unwrap();
        entry.count.fetch_add(1, Ordering::Relaxed);
        InternedStr(self.0)
    }
}

impl Drop for InternedStr {
    fn drop(&mut self) {
        let entry = INTERNER.id_to_entry.get(&self.0).unwrap();
        if entry.count.fetch_sub(1, Ordering::Release) == 0 {
            fence(Ordering::Acquire);
            let (_, entry) = INTERNER.id_to_entry.remove(&self.0).unwrap();
            let str = unsafe { str::from_raw_parts(entry.ptr, entry.len) };
            INTERNER.str_to_id.remove(str);
            unsafe { dealloc(entry.ptr, Layout::from_size_align_unchecked(entry.len, 1)) };
        }
    }
}

impl fmt::Debug for InternedStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl fmt::Display for InternedStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

#[cfg(test)]
mod tests {

    use crate::intern::InternedStr;

    #[test]
    fn test() {
        let a = InternedStr::new("hello");
        assert!(a.id() == 0);

        let b = InternedStr::new("hello");
        assert!(b.id() == 0);

        let c = InternedStr::new("goodbye");
        assert!(c.id() == 1);
    }
}

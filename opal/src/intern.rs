use std::{
    alloc::{Layout, alloc, dealloc},
    collections::{HashMap, hash_map::Entry},
    fmt,
    ptr::copy,
    str,
    sync::{
        RwLock,
        atomic::{AtomicUsize, Ordering},
    },
};

struct Interner {
    id_to_entry: HashMap<u32, StringEntry>,
    str_to_id: HashMap<String, u32>,
    next_id: u32,
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

static INTERNER: RwLock<Option<Interner>> = RwLock::new(None);

fn with_interner_mut<T>(f: impl FnOnce(&mut Interner) -> T) -> T {
    let mut interner = INTERNER.write().unwrap();
    f(interner.get_or_insert_with(|| Interner {
        id_to_entry: HashMap::new(),
        str_to_id: HashMap::new(),
        next_id: 0,
    }))
}

fn with_interner<T>(f: impl FnOnce(&Interner) -> T) -> T {
    let interner = INTERNER.read().unwrap();
    f(interner
        .as_ref()
        .expect("no point trying to read from interner before any strings have been interened"))
}

impl InternedStr {
    pub fn new(string: impl Into<String>) -> InternedStr {
        with_interner_mut(|interner| {
            InternedStr(match interner.str_to_id.entry(string.into()) {
                Entry::Occupied(entry) => {
                    let id = *entry.get();
                    let entry = interner.id_to_entry.get(&id).unwrap();
                    entry.count.fetch_add(1, Ordering::Relaxed);
                    id
                }
                Entry::Vacant(entry) => {
                    let id = interner.next_id;
                    interner.next_id += 1;
                    unsafe {
                        let len = entry.key().len();
                        let ptr = alloc(Layout::from_size_align_unchecked(len, 1));
                        copy(entry.key().as_ptr(), ptr, len);
                        let count = AtomicUsize::new(1);
                        interner.id_to_entry.insert(id, StringEntry { count, ptr, len });
                        entry.insert(id);
                        id
                    }
                }
            })
        })
    }
    fn with_entry<T>(&self, f: impl FnOnce(&StringEntry) -> T) -> T {
        with_interner(|interner| f(interner.id_to_entry.get(&self.0).unwrap()))
    }
    pub fn as_str(&self) -> &str {
        self.with_entry(|entry| unsafe { str::from_raw_parts(entry.ptr, entry.len) })
    }
    pub fn id(&self) -> u32 {
        self.0
    }
}

impl Clone for InternedStr {
    fn clone(&self) -> Self {
        self.with_entry(|entry| {
            entry.count.fetch_add(1, Ordering::Relaxed);
        });
        InternedStr(self.0)
    }
}

impl Drop for InternedStr {
    fn drop(&mut self) {
        with_interner_mut(|interner| {
            let entry = interner.id_to_entry.get_mut(&self.0).unwrap();
            *entry.count.get_mut() -= 1;
            if *entry.count.get_mut() == 0 {
                let entry = interner.id_to_entry.remove(&self.0).unwrap();
                let str = unsafe { str::from_raw_parts(entry.ptr, entry.len) };
                interner.str_to_id.remove(str);
                unsafe { dealloc(entry.ptr, Layout::from_size_align_unchecked(entry.len, 1)) };
            }
        })
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

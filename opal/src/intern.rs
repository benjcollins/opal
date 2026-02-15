use core::fmt;
use std::{cell::RefCell, collections::HashMap, marker::PhantomData};

#[derive(PartialEq, Eq, Hash)]
pub struct InternedStr {
    index: u16,
    _marker: PhantomData<*const ()>,
}

struct Interner {
    vec: Vec<String>,
    free_list: Vec<u16>,
    map: HashMap<&'static str, u16>,
}

struct String {
    str: &'static str,
    ref_count: u32,
}

thread_local! {
    static INTERNER: RefCell<Interner> = RefCell::new(Interner {
        vec: Vec::new(),
        map: HashMap::new(),
        free_list: Vec::new(),
    });
}

impl InternedStr {
    pub fn intern(s: &str) -> InternedStr {
        let index = INTERNER.with_borrow_mut(|interner| match interner.map.get(s) {
            Some(&index) => {
                interner.vec[index as usize].ref_count += 1;
                index
            }
            None => {
                let str = Box::leak(s.to_string().into_boxed_str());
                let index = if let Some(index) = interner.free_list.pop() {
                    interner.vec[index as usize] = String { str, ref_count: 1 };
                    index
                } else {
                    let index = interner.vec.len() as u16;
                    interner.vec.push(String { str, ref_count: 1 });
                    index
                };
                interner.map.insert(str, index);
                index
            }
        });
        InternedStr {
            index: index as u16,
            _marker: PhantomData,
        }
    }
    pub fn as_str(&self) -> &str {
        INTERNER.with_borrow(|interner| interner.vec[self.index as usize].str)
    }
}

impl Clone for InternedStr {
    fn clone(&self) -> Self {
        INTERNER.with_borrow_mut(|interner| {
            interner.vec[self.index as usize].ref_count += 1;
        });
        InternedStr {
            index: self.index,
            _marker: PhantomData,
        }
    }
}

impl Drop for InternedStr {
    fn drop(&mut self) {
        INTERNER.with_borrow_mut(|interner| {
            let s = &mut interner.vec[self.index as usize];
            s.ref_count -= 1;
            if s.ref_count == 0 {
                interner.map.remove(s.str);
                unsafe { drop(Box::from_raw(s.str.as_ptr() as *mut u8)) };
                interner.free_list.push(self.index);
            }
        });
    }
}

impl fmt::Debug for InternedStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.as_str())
    }
}

impl fmt::Display for InternedStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

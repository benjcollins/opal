use core::fmt;
use std::{cell::RefCell, collections::HashMap, marker::PhantomData};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct InternedStr {
    index: u16,
    _marker: PhantomData<*const ()>,
}

struct Interner {
    vec: Vec<&'static str>,
    map: HashMap<&'static str, u16>,
}

thread_local! {
    static INTERNER: RefCell<Interner> = RefCell::new(Interner {
        vec: Vec::new(),
        map: HashMap::new(),
    });
}

impl InternedStr {
    pub fn intern(s: &str) -> InternedStr {
        let index = INTERNER.with(|interner| {
            let mut interner = interner.borrow_mut();
            match interner.map.get(s) {
                Some(index) => *index as u16,
                None => {
                    let s = Box::leak(s.to_string().into_boxed_str());
                    let index = interner.vec.len() as u16;
                    interner.vec.push(s);
                    interner.map.insert(s, index);
                    index
                }
            }
        });
        InternedStr {
            index: index as u16,
            _marker: PhantomData,
        }
    }
    pub fn as_str(&self) -> &'static str {
        INTERNER.with(|interner| {
            interner.borrow().vec[self.index as usize]
        })
    }
}

impl fmt::Debug for InternedStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("InternedStr").field(&self.as_str()).finish()
    }
}

impl fmt::Display for InternedStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
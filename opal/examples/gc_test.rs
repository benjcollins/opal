use std::{cell::RefCell, marker::PhantomData};

use opal::gc::{GlobalGc, Trace, ref_::Gc};
use opal_proc::{Rootable, Trace};

#[derive(Trace)]
struct Node<'gc, T> {
    prev: RefCell<Option<Gc<'gc, Node<'gc, T>>>>,
    payload: T,
    next: RefCell<Option<Gc<'gc, Node<'gc, T>>>>,
}

#[derive(Rootable)]
#[lifetime('gc)]
#[root(Node<'gc, T>)]
struct NodeRoot<T: 'static>(PhantomData<T>);

impl<'gc, T: Trace> Node<'gc, T> {
    fn new(payload: T) -> Node<'gc, T> {
        Node {
            prev: RefCell::new(None),
            payload,
            next: RefCell::new(None),
        }
    }
    fn set_prev(&self, prev: Option<Gc<'gc, Node<'gc, T>>>) {
        *self.prev.borrow_mut() = prev;
    }
    fn set_next(&self, next: Option<Gc<'gc, Node<'gc, T>>>) {
        *self.next.borrow_mut() = next;
    }
}

#[derive(Trace, Clone, Copy)]
struct Tree<'gc, T: Trace> {
    payload: T,
    children: Gc<'gc, [Gc<'gc, Tree<'gc, T>>]>,
}

#[derive(Rootable)]
#[lifetime('gc)]
#[root(Tree<'gc, T>)]
struct TreeRoot<T: Trace + 'static>(PhantomData<T>);

fn main() {
    let mut gc = GlobalGc::init().expect("could not acquire global gc");

    let a_tree = gc.alloc(Tree {
        payload: "A",
        children: gc.alloc_slice(&[]),
    });
    let b_tree = gc.alloc(Tree {
        payload: "B",
        children: gc.alloc_slice(&[]),
    });
    let c_tree = gc.alloc(Tree {
        payload: "C",
        children: gc.alloc_slice(&[a_tree, b_tree]),
    });

    let a = gc.alloc(Node::new("A"));
    let b = gc.alloc(Node::new("B"));
    let c = gc.alloc(Node::new("C"));

    a.set_next(Some(b));
    b.set_next(Some(c));
    b.set_prev(Some(a));
    c.set_prev(Some(b));

    let root = gc.root::<NodeRoot<_>>(c);
    let root2 = gc.root::<TreeRoot<_>>(c_tree);

    println!("{}", gc.allocation_count());

    gc.collect();

    println!("{}", gc.allocation_count());
}

// garbage collector todo:
// mark + sweep phases [DONE]
// derive macro for trace [DONE]
// slice types [DONE]
// derive macro for root [DONE]
// optimisation to skip elements with no gc refs [DONE]
// drop for gc type
// trace for enums

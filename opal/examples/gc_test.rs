use opal::gc::{Gc, GcRef, Rootable, Trace};

struct PersonRoot;

impl Rootable for PersonRoot {
    type Root<'gc> = Person<'gc>;
}

struct Person<'gc> {
    mother: Option<GcRef<'gc, Person<'gc>>>,
    father: Option<GcRef<'gc, Person<'gc>>>,
    name: &'static str,
    age: u32,
    children: Vec<GcRef<'gc, Person<'gc>>>,
}

unsafe impl<'m> Trace for Person<'m> {
    fn trace(&self) {}
}

fn main() {
    let mut gc = Gc::init();

    let fred_root = {
        let andy = gc.alloc(Person {
            name: "Andy",
            age: 67,
            mother: None,
            father: None,
            children: vec![],
        });

        let fred = gc.alloc(Person {
            name: "Fred",
            age: 33,
            mother: None,
            father: Some(andy),
            children: vec![],
        });

        gc.root::<PersonRoot>(fred)
    };

    gc.collect();

    let fred = gc.get_ref(fred_root);
    let andy = fred.father.unwrap();

    println!("{}", andy.name);
    println!("{}", andy.age);
}

// garbage collector todo:
// mark + sweep phases
// derive macro for trace
// drop for gc type
// array types

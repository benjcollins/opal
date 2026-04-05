use std::{sync::Arc, thread};

use opal::{heap::Heap, value::Value};

fn main() {
    let heap = Arc::new(Heap::new().expect("could not create heap!"));
    let mut threads = vec![];

    for _ in 0..8 {
        let heap = heap.clone();
        threads.push(thread::spawn(move || {
            let stack = heap.new_stack();
            for i in 0..100 {
                println!("creating mutator");
                heap.with_mutator(|mutator| {
                    println!("allocating array");
                    let array = mutator.alloc_array(10);
                    stack.set(i % 10, Value::array(array), mutator);
                });
                println!("destroying mutator");
            }
        }));
    }

    thread::spawn(move || {
        loop {
            let initial_object_count = heap.object_count();
            println!("object count: {}", initial_object_count);

            heap.collect_garabge();
            println!("just collected garabage!");

            println!("freed objects: {}", heap.object_count())
        }
    });

    for thread in threads {
        thread.join().unwrap();
    }
}

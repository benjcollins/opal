use opal::heap::Heap;

fn main() {
    let mut heap = Heap::new().expect("could not create heap");

    let mutator = heap.mutator();

    let _ = mutator.alloc_native(Custom(5));

    drop(mutator);

    heap.collect_garbage();
}

struct Custom(u32);

impl Drop for Custom {
    fn drop(&mut self) {
        println!("{}", self.0)
    }
}

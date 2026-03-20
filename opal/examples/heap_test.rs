use opal::heap2::Heap;

fn main() {
    let mut heap = Heap::init().expect("could not create heap");

    let thing = heap.alloc_native(Custom(5));

    heap.collect_garbage();
}

struct Custom(u32);

impl Drop for Custom {
    fn drop(&mut self) {
        println!("{}", self.0)
    }
}

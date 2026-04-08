use std::thread;

use opal::lexer::Lexer;

fn main() {
    let source = include_str!("../../examples/example.opal");
    let mut threads = vec![];
    for _ in 0..10 {
        threads.push(thread::spawn(|| {
            let mut lexer = Lexer::new(source);
            let mut tokens = vec![];
            while let Some((token, _span)) = lexer.next_token() {
                tokens.push(token);
            }
        }));
    }
    for thread in threads {
        thread.join().unwrap();
    }
}

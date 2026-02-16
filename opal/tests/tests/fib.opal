module fib;

fun fib_rec(n: Int) -> Int {
    if (n == 0) {
        return 0;
    } else if (n == 1) {
        return 1;
    } else {
        return fib_rec(n - 1) + fib_rec(n - 2);
    }
}

fun fib_loop(n: Int) -> Int {
    let i = 0;
    let a = 1;
    let b = 0;
    while (i < n) {
        let t = a;
        a += b;
        b = t;
        i += 1;
    }
    return b;
}

fun test_fib_rec_10() {
    assert(fib_rec(10) == 55);
}

fun test_fib_loop_10() {
    assert(fib_loop(10) == 55);
}

fun test_fib_rec_25() {
    assert(fib_rec(25) == 75025);
}

fun test_fib_loop_50() {
    assert(fib_loop(50) == 12586269025);
}

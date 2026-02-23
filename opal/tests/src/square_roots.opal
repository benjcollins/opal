module square_roots;

/* a module to calculate the square root of a number using newton's method */

// calculate square root
fun approx_square_root(n: Float, c: Int) -> Float {
    let x = n / 2.0;
    let i = 0;
    while (i < c) {
        x := (x + n / x) / 2.0;
        i += 1;
    }
    return x;
}

fun abs(n: Float) -> Float {
    if (n > 0.0) {
        return n;
    } else {
        return 0.0 - n;
    }
}

fun test_approx_square_root_123() {
    let n = 123.0;
    let n_approx = approx_square_root(n * n, 10);
    let error = abs(n - n_approx);
    assert(error < 0.001);
}

fun test_approx_square_root_738() {
    let n = 738.0;
    let n_approx = approx_square_root(n * n, 15);
    let error = abs(n - n_approx);
    assert(error < 0.001);
}

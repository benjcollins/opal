module sum;

fun sum_rec(n: Int) -> Int {
    if (n == 0) {
        return 0;
    } else {
        return n + sum_rec(n - 1);
    }
}

fun sum_loop(n: Int) -> Int {
    let total = 0;
    let i = 0;
    while (i <= n) {
        total = total + i;
        i = i + 1;
    }
    return total;
}

fun test_sum_rec_10() {
    assert(sum_rec(10) == 55);
}

fun test_sum_loop_10() {
    assert(sum_loop(10) == 55);
}

fun test_sum_rec_100() {
    assert(sum_rec(100) == 5050);
}

fun test_sum_loop_100() {
    assert(sum_loop(100) == 5050);
}

module count_ones;

fun count_ones(n: Int) -> Int {
    let count = 0;
    while (n != 0) {
        if ((n & 1) != 0) {
            count += 1;
        }
        n >>= 1;
    }
    return count;
}

fun test_count_ones_5() {
    assert(count_ones(5) == 2);
}

fun test_count_ones_11() {
    assert(count_ones(11) == 3);
}

fun test_count_ones_14() {
    assert(count_ones(15) == 4);
}

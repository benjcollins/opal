module sum;

fun sum(list: List[Int]) -> Int {
    let i = 0;
    let total = 0;
    while (i < len(list)) {
        total += list[i];
        i += 1;
    }
    return total;
}

fun test_sum() {
    let total = sum([1, 2, 3, 4, 5]);
    assert(total == 15);
}

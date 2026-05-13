module sum;

fun sum(list: List[Int]) -> Int {
    var i = 0;
    var total = 0;
    while (i < len(list)) {
        total += list[i];
        i += 1;
    }
    return total;
}

fun test_sum() {
    var total = sum([1, 2, 3, 4, 5]);
    assert(total == 15);
}

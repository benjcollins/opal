module double;

fun test_double() {
    let list = [1, 2, 3];

    double(list);

    assert(list == [2, 4, 6]);
}

fun double(list: List[Int]) {
    let i = 0;
    while (i < len(list)) {
        list[i] *= 2;
        i += 1;
    }
}

module main;

fun main() {
    let n = 17;
    print_int(count_ones(n));
}

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

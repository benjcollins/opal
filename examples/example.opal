module main;

fun test() {

}

fun main() {
    let i = 1;
    while (i < 100) {
        if (i % 7 == 0) {
            break;
        }
        i += 1;
    }
    print_int(i);
}

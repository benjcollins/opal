module main;

fun main() {
    let array = [1, 2, 3];

    array[1] = 9;

    let i = 0;
    while (i < len(array)) {
        print_int(array[i]);
        i += 1;
    }

    print_array(array);
}

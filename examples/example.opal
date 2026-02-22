module main;

fun main() {
    let arr = [1, 2, 3];

    let i = 0;
    while (i < len(arr)) {
        print_int(arr[i]);
        i += 1;
    }

    print_array(arr);
}

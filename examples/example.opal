module main;

fun main() {
    let array = [9, 3, 4, 7, 1];
    print_array(array);
    sort(array);
    print_array(array);
}

fun sort(array: Array[Int]) {
    let i = 0;
    while (i < len(array)) {
        let j = i + 1;
        while (j < len(array)) {
            if (array[i] > array[j]) {
                let temp = array[i];
                array[i] = array[j];
                array[j] = temp;
            }
            j += 1;
        }
        i += 1;
    }
}

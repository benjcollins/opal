module sort;

fun test_sort() {
    var list = [9, 3, 4, 7, 1];
    sort(list);
    assert(list == [1, 3, 4, 7, 9]);
}

fun sort(list: List[Int]) {
    var i = 0;
    while (i < len(list)) {
        var j = i + 1;
        while (j < len(list)) {
            if (list[i] > list[j]) {
                var temp = list[i];
                list[i] := list[j];
                list[j] := temp;
            }
            j += 1;
        }
        i += 1;
    }
}

module main;

fun find[T](list: List[T], pred: fun(T) -> Bool, default: T) -> T {
    var i = 0;
    while (i < len(list)) {
        if (pred(list[i])) {
            return list[i];
        }
        i += 1;
    }
    return default;
}

fun even(n: Int) -> Bool {
    return n % 2 == 0;
}

fun main() {
    print(find([1, 2, 3], even, 0));
}

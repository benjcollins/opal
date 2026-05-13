module primes;

fun generate_sieve(size: Int) -> List[Bool] {
    var sieve = [false; size];
    var i = 2;
    while (i < len(sieve)) {
        var j = i * 2;
        while (j < len(sieve)) {
            sieve[j] := true;
            j += i;
        }
        i += 1;
    }
    return sieve;
}

fun count_primes(sieve: List[Bool]) -> Int {
    var i = 1;
    var prime_count = 0;
    while (i < len(sieve)) {
        if (!sieve[i]) {
            prime_count += 1;
        }
        i += 1;
    }
    return prime_count;
}

fun collect_primes(sieve: List[Bool]) -> List[Int] {
    var prime_count = count_primes(sieve);
    var primes = [0; prime_count];
    var i = 1;
    var j = 0;
    while (i < len(sieve)) {
        if (!sieve[i]) {
            primes[j] := i;
            j += 1;
        }
        i += 1;
    }
    return primes;
}

fun primes(n: Int) -> List[Int] {
    var sieve = generate_sieve(n);
    return collect_primes(sieve);
}

fun test_primes_10() {
    var primes = primes(10);
    assert(primes == [1, 2, 3, 5, 7]);
}

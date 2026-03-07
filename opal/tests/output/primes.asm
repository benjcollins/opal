collect_primes:
  Mov r(2), r(0);
  Call r(1), c(0), 2;
  ArrayInit r(2), r(1);
  IBLte r(1), r(3), 4;
  ArraySet r(2), c(1), r(3);
  IAdd r(3), r(3), c(2);
  Jump -3;
  Mov r(3), c(2);
  Mov r(4), c(1);
  Mov r(6), r(0);
  Call r(5), c(3), 6;
  IBLte r(5), r(3), 9;
  ArrayGet r(7), r(0), r(3);
  SEq r(6), r(7), c(1);
  BEq r(6), c(2), 2;
  Jump 3;
  ArraySet r(2), r(3), r(4);
  IAdd r(4), r(4), c(2);
  IAdd r(3), r(3), c(2);
  Jump -10;
  Ret r(2);

count_primes:
  Mov r(1), c(0);
  Mov r(2), c(1);
  Mov r(4), r(0);
  Call r(3), c(2), 4;
  IBLte r(3), r(1), 8;
  ArrayGet r(5), r(0), r(1);
  SEq r(4), r(5), c(1);
  BEq r(4), c(0), 2;
  Jump 2;
  IAdd r(2), r(2), c(0);
  IAdd r(1), r(1), c(0);
  Jump -9;
  Ret r(2);

generate_sieve:
  ArrayInit r(1), r(0);
  IBLte r(0), r(2), 4;
  ArraySet r(1), c(0), r(2);
  IAdd r(2), r(2), c(1);
  Jump -3;
  Mov r(2), c(2);
  Mov r(4), r(1);
  Call r(3), c(3), 4;
  IBLte r(3), r(2), 10;
  Mov r(4), r(2);
  Mov r(6), r(1);
  Call r(5), c(4), 6;
  IBLte r(5), r(4), 4;
  IAdd r(4), r(4), r(2);
  ArraySet r(1), c(1), r(4);
  Jump -5;
  IAdd r(2), r(2), c(1);
  Jump -11;
  Ret r(1);

primes:
  Mov r(2), r(0);
  Call r(1), c(0), 2;
  Mov r(3), r(1);
  Call r(2), c(1), 3;
  Ret r(2);

test_primes_10:
  Mov r(1), c(1);
  Call r(0), c(0), 1;
  ArrayGet r(3), r(0), c(3);
  SEq r(2), r(3), c(4);
  Call r(1), c(2), 2;
  ArrayGet r(3), r(0), c(4);
  SEq r(2), r(3), c(6);
  Call r(1), c(5), 2;
  ArrayGet r(3), r(0), c(6);
  SEq r(2), r(3), c(8);
  Call r(1), c(7), 2;
  ArrayGet r(3), r(0), c(8);
  SEq r(2), r(3), c(10);
  Call r(1), c(9), 2;
  ArrayGet r(3), r(0), c(12);
  SEq r(2), r(3), c(13);
  Call r(1), c(11), 2;
  Ret c(3);


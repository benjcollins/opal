sum:
  Mov r(1), c(0);
  Mov r(2), c(1);
  Mov r(4), r(0);
  Call r(3), c(2), 4;
  IBLte r(3), r(1), 5;
  ArrayGet r(4), r(0), r(1);
  IAdd r(2), r(2), r(4);
  IAdd r(1), r(1), c(3);
  Jump -6;
  Ret r(2);

test_sum:
  ArrayInit r(1), c(1);
  ArraySet r(1), c(2), c(3);
  ArraySet r(1), c(4), c(5);
  ArraySet r(1), c(6), c(7);
  ArraySet r(1), c(8), c(9);
  ArraySet r(1), c(10), c(11);
  Call r(0), c(0), 1;
  SEq r(2), r(0), c(13);
  Call r(1), c(12), 2;
  Ret c(14);


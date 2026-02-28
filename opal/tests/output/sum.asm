sum:
  Mov r(1), c(0);
  Mov r(2), c(0);
  Mov r(4), r(0);
  Call r(3), c(1), 4;
  IBLte r(3), r(1), 5;
  ArrayGet r(4), r(0), r(1);
  IAdd r(2), r(2), r(4);
  IAdd r(1), r(1), c(2);
  Jump -6;
  Ret r(2);

test_sum:
  ArrayInit r(1), c(1);
  ArraySet r(1), c(2), c(3);
  ArraySet r(1), c(4), c(2);
  ArraySet r(1), c(5), c(4);
  ArraySet r(1), c(6), c(5);
  ArraySet r(1), c(1), c(6);
  Call r(0), c(0), 1;
  SEq r(2), r(0), c(8);
  Call r(1), c(7), 2;
  Ret c(3);


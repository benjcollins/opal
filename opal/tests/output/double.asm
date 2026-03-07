double:
  Mov r(1), c(0);
  Mov r(3), r(0);
  Call r(2), c(1), 3;
  IBLte r(2), r(1), 6;
  ArrayGet r(3), r(0), r(1);
  IMul r(3), r(3), c(2);
  ArraySet r(0), r(3), r(1);
  IAdd r(1), r(1), c(3);
  Jump -7;
  Ret c(0);

test_double:
  ArrayInit r(0), c(0);
  ArraySet r(0), c(1), c(2);
  ArraySet r(0), c(3), c(1);
  ArraySet r(0), c(0), c(3);
  Mov r(2), r(0);
  Call r(1), c(4), 2;
  ArrayGet r(3), r(0), c(2);
  SEq r(2), r(3), c(3);
  Call r(1), c(5), 2;
  ArrayGet r(3), r(0), c(1);
  SEq r(2), r(3), c(7);
  Call r(1), c(6), 2;
  ArrayGet r(3), r(0), c(3);
  SEq r(2), r(3), c(9);
  Call r(1), c(8), 2;
  Ret c(2);


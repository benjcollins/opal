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
  Ret c(4);

test_double:
  ArrayInit r(0), c(0);
  ArraySet r(0), c(1), c(2);
  ArraySet r(0), c(3), c(4);
  ArraySet r(0), c(5), c(6);
  Mov r(2), r(0);
  Call r(1), c(7), 2;
  ArrayInit r(3), c(9);
  ArraySet r(3), c(10), c(11);
  ArraySet r(3), c(12), c(13);
  ArraySet r(3), c(14), c(15);
  SEq r(2), r(0), r(3);
  Call r(1), c(8), 2;
  Ret c(16);


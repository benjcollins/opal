sort:
  Mov r(1), c(0);
  Mov r(3), r(0);
  Call r(2), c(1), 3;
  IBLte r(2), r(1), 17;
  IAdd r(3), r(1), c(2);
  Mov r(5), r(0);
  Call r(4), c(3), 5;
  IBLte r(4), r(3), 11;
  ArrayGet r(5), r(0), r(1);
  ArrayGet r(6), r(0), r(3);
  IBLt r(6), r(5), 2;
  Jump 5;
  ArrayGet r(7), r(0), r(1);
  ArrayGet r(8), r(0), r(3);
  ArraySet r(0), r(8), r(1);
  ArraySet r(0), r(7), r(3);
  IAdd r(3), r(3), c(4);
  Jump -12;
  IAdd r(1), r(1), c(5);
  Jump -18;
  Ret c(6);

test_sort:
  ArrayInit r(0), c(0);
  ArraySet r(0), c(1), c(2);
  ArraySet r(0), c(3), c(4);
  ArraySet r(0), c(5), c(6);
  ArraySet r(0), c(7), c(8);
  ArraySet r(0), c(9), c(10);
  Mov r(2), r(0);
  Call r(1), c(11), 2;
  ArrayInit r(3), c(13);
  ArraySet r(3), c(14), c(15);
  ArraySet r(3), c(16), c(17);
  ArraySet r(3), c(18), c(19);
  ArraySet r(3), c(20), c(21);
  ArraySet r(3), c(22), c(23);
  SEq r(2), r(0), r(3);
  Call r(1), c(12), 2;
  Ret c(24);


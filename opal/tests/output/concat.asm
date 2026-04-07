concat:
  Mov r(5), r(0);
  Call r(4), c(0), 5;
  Mov r(6), r(1);
  Call r(5), c(1), 6;
  IAdd r(3), r(4), r(5);
  ArrayInit r(2), r(3);
  Mov r(6), c(3);
  IBLte r(3), r(6), 4;
  ArraySet r(2), c(2), r(6);
  IAdd r(6), r(6), c(4);
  Jump -3;
  Mov r(4), r(0);
  Mov r(5), r(2);
  Mov r(6), c(6);
  Mov r(7), c(7);
  Mov r(9), r(0);
  Call r(8), c(8), 9;
  Call r(3), c(5), 4;
  Mov r(4), r(1);
  Mov r(5), r(2);
  Mov r(6), c(10);
  Mov r(8), r(0);
  Call r(7), c(11), 8;
  Mov r(9), r(1);
  Call r(8), c(12), 9;
  Call r(3), c(9), 4;
  Ret r(2);

copy:
  Mov r(5), c(0);
  IBLte r(4), r(5), 7;
  IAdd r(6), r(3), r(5);
  IAdd r(8), r(2), r(5);
  ArrayGet r(7), r(0), r(8);
  ArraySet r(1), r(7), r(6);
  IAdd r(5), r(5), c(1);
  Jump -6;
  Ret c(2);

test_concat:
  ArrayInit r(0), c(0);
  ArraySet r(0), c(1), c(2);
  ArraySet r(0), c(3), c(4);
  ArraySet r(0), c(5), c(6);
  ArrayInit r(1), c(7);
  ArraySet r(1), c(8), c(9);
  ArraySet r(1), c(10), c(11);
  Mov r(3), r(0);
  Mov r(4), r(1);
  Call r(2), c(12), 3;
  ArrayInit r(5), c(14);
  ArraySet r(5), c(15), c(16);
  ArraySet r(5), c(17), c(18);
  ArraySet r(5), c(19), c(20);
  ArraySet r(5), c(21), c(22);
  ArraySet r(5), c(23), c(24);
  SEq r(4), r(2), r(5);
  Call r(3), c(13), 4;
  Ret c(25);


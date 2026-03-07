array_eq:
  Mov r(3), r(0);
  Call r(2), c(0), 3;
  Mov r(4), r(1);
  Call r(3), c(1), 4;
  BEq r(2), r(3), 2;
  Ret c(2);
  Mov r(4), c(2);
  Mov r(6), r(0);
  Call r(5), c(3), 6;
  IBLte r(5), r(4), 7;
  ArrayGet r(6), r(0), r(4);
  ArrayGet r(7), r(1), r(4);
  BEq r(6), r(7), 2;
  Ret c(2);
  IAdd r(4), r(4), c(4);
  Jump -8;
  Ret c(4);

concat:
  Mov r(5), r(0);
  Call r(4), c(0), 5;
  Mov r(6), r(1);
  Call r(5), c(1), 6;
  IAdd r(3), r(4), r(5);
  ArrayInit r(2), r(3);
  IBLte r(3), r(6), 4;
  ArraySet r(2), c(2), r(6);
  IAdd r(6), r(6), c(3);
  Jump -3;
  Mov r(4), r(0);
  Mov r(5), r(2);
  Mov r(6), c(2);
  Mov r(7), c(2);
  Mov r(9), r(0);
  Call r(8), c(5), 9;
  Call r(3), c(4), 4;
  Mov r(4), r(1);
  Mov r(5), r(2);
  Mov r(6), c(2);
  Mov r(8), r(0);
  Call r(7), c(7), 8;
  Mov r(9), r(1);
  Call r(8), c(8), 9;
  Call r(3), c(6), 4;
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
  Ret c(0);

test_concat:
  ArrayInit r(0), c(0);
  ArraySet r(0), c(1), c(2);
  ArraySet r(0), c(3), c(1);
  ArraySet r(0), c(0), c(3);
  ArrayInit r(1), c(3);
  ArraySet r(1), c(4), c(2);
  ArraySet r(1), c(5), c(1);
  Mov r(3), r(0);
  Mov r(4), r(1);
  Call r(2), c(6), 3;
  Mov r(5), r(2);
  ArrayInit r(6), c(5);
  ArraySet r(6), c(1), c(2);
  ArraySet r(6), c(3), c(1);
  ArraySet r(6), c(0), c(3);
  ArraySet r(6), c(4), c(0);
  ArraySet r(6), c(5), c(4);
  Call r(4), c(8), 5;
  Call r(3), c(7), 4;
  Ret c(2);


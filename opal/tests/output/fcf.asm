double:
  IMul r(1), r(0), c(0);
  Ret r(1);

filter_int:
  Mov r(4), r(0);
  Call r(3), c(0), 4;
  ArrayInit r(2), r(3);
  Mov r(4), c(2);
  IBLte r(3), r(4), 4;
  ArraySet r(2), c(1), r(4);
  IAdd r(4), r(4), c(3);
  Jump -3;
  Mov r(3), c(4);
  Mov r(4), c(5);
  Mov r(6), r(0);
  Call r(5), c(6), 6;
  IBLte r(5), r(3), 10;
  ArrayGet r(7), r(0), r(3);
  Call r(6), r(1), 7;
  BEq r(6), c(7), 2;
  Jump 4;
  ArrayGet r(7), r(0), r(3);
  ArraySet r(2), r(7), r(4);
  IAdd r(4), r(4), c(8);
  IAdd r(3), r(3), c(9);
  Jump -11;
  ArrayInit r(6), r(4);
  Mov r(7), c(11);
  IBLte r(4), r(7), 4;
  ArraySet r(6), c(10), r(7);
  IAdd r(7), r(7), c(12);
  Jump -3;
  Mov r(7), c(13);
  IBLte r(4), r(7), 5;
  ArrayGet r(8), r(2), r(7);
  ArraySet r(6), r(8), r(7);
  IAdd r(7), r(7), c(14);
  Jump -4;
  Ret r(6);

is_even:
  IMod r(2), r(0), c(0);
  SEq r(1), r(2), c(1);
  Ret r(1);

map_int:
  Mov r(4), r(0);
  Call r(3), c(0), 4;
  ArrayInit r(2), r(3);
  Mov r(4), c(2);
  IBLte r(3), r(4), 4;
  ArraySet r(2), c(1), r(4);
  IAdd r(4), r(4), c(3);
  Jump -3;
  Mov r(3), c(4);
  Mov r(5), r(0);
  Call r(4), c(5), 5;
  IBLte r(4), r(3), 6;
  ArrayGet r(6), r(0), r(3);
  Call r(5), r(1), 6;
  ArraySet r(2), r(5), r(3);
  IAdd r(3), r(3), c(6);
  Jump -7;
  Ret r(2);

test_double:
  ArrayInit r(0), c(0);
  ArraySet r(0), c(1), c(2);
  ArraySet r(0), c(3), c(4);
  ArraySet r(0), c(5), c(6);
  Mov r(2), r(0);
  Mov r(3), c(8);
  Call r(1), c(7), 2;
  ArrayInit r(4), c(10);
  ArraySet r(4), c(11), c(12);
  ArraySet r(4), c(13), c(14);
  ArraySet r(4), c(15), c(16);
  SEq r(3), r(1), r(4);
  Call r(2), c(9), 3;
  Ret c(17);

test_filter:
  ArrayInit r(0), c(0);
  ArraySet r(0), c(1), c(2);
  ArraySet r(0), c(3), c(4);
  ArraySet r(0), c(5), c(6);
  ArraySet r(0), c(7), c(8);
  ArraySet r(0), c(9), c(10);
  ArraySet r(0), c(11), c(12);
  Mov r(2), r(0);
  Mov r(3), c(14);
  Call r(1), c(13), 2;
  ArrayInit r(4), c(16);
  ArraySet r(4), c(17), c(18);
  ArraySet r(4), c(19), c(20);
  ArraySet r(4), c(21), c(22);
  SEq r(3), r(1), r(4);
  Call r(2), c(15), 3;
  Ret c(23);


array_eq:
  Mov r(3), r(0);
  Call r(2), c(0), 3;
  Mov r(4), r(1);
  Call r(3), c(1), 4;
  BEq r(2), r(3), 2;
  Ret c(2);
  Mov r(4), c(3);
  Mov r(6), r(0);
  Call r(5), c(4), 6;
  IBLte r(5), r(4), 7;
  ArrayGet r(6), r(0), r(4);
  ArrayGet r(7), r(1), r(4);
  BEq r(6), r(7), 2;
  Ret c(5);
  IAdd r(4), r(4), c(6);
  Jump -8;
  Ret c(7);

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

test_array_diff_len:
  ArrayInit r(1), c(1);
  ArraySet r(1), c(2), c(3);
  ArraySet r(1), c(4), c(5);
  ArrayInit r(2), c(6);
  ArraySet r(2), c(7), c(8);
  ArraySet r(2), c(9), c(10);
  ArraySet r(2), c(11), c(12);
  Call r(0), c(0), 1;
  BEq r(0), c(13), 2;
  Call r(1), c(14), 2;
  Ret c(15);

test_array_eq:
  ArrayInit r(2), c(2);
  ArraySet r(2), c(3), c(4);
  ArraySet r(2), c(5), c(6);
  ArraySet r(2), c(7), c(8);
  ArrayInit r(3), c(9);
  ArraySet r(3), c(10), c(11);
  ArraySet r(3), c(12), c(13);
  ArraySet r(3), c(14), c(15);
  Call r(1), c(1), 2;
  Call r(0), c(0), 1;
  Ret c(16);

test_array_not_eq:
  ArrayInit r(1), c(1);
  ArraySet r(1), c(2), c(3);
  ArraySet r(1), c(4), c(5);
  ArraySet r(1), c(6), c(7);
  ArrayInit r(2), c(8);
  ArraySet r(2), c(9), c(10);
  ArraySet r(2), c(11), c(12);
  ArraySet r(2), c(13), c(14);
  Call r(0), c(0), 1;
  BEq r(0), c(15), 2;
  Call r(1), c(16), 2;
  Ret c(17);

test_sort:
  ArrayInit r(0), c(0);
  ArraySet r(0), c(1), c(2);
  ArraySet r(0), c(3), c(4);
  ArraySet r(0), c(5), c(6);
  ArraySet r(0), c(7), c(8);
  ArraySet r(0), c(9), c(10);
  Mov r(2), r(0);
  Call r(1), c(11), 2;
  Mov r(3), r(0);
  ArrayInit r(4), c(14);
  ArraySet r(4), c(15), c(16);
  ArraySet r(4), c(17), c(18);
  ArraySet r(4), c(19), c(20);
  ArraySet r(4), c(21), c(22);
  ArraySet r(4), c(23), c(24);
  Call r(2), c(13), 3;
  Call r(1), c(12), 2;
  Ret c(25);


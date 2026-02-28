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
  IAdd r(3), r(3), c(2);
  Jump -12;
  IAdd r(1), r(1), c(2);
  Jump -18;
  Ret c(0);

test_array_diff_len:
  ArrayInit r(1), c(1);
  ArraySet r(1), c(2), c(3);
  ArraySet r(1), c(1), c(2);
  ArrayInit r(2), c(4);
  ArraySet r(2), c(2), c(3);
  ArraySet r(2), c(1), c(2);
  ArraySet r(2), c(4), c(1);
  Call r(0), c(0), 1;
  BEq r(0), c(3), 2;
  Call r(1), c(5), 2;
  Ret c(3);

test_array_eq:
  ArrayInit r(2), c(2);
  ArraySet r(2), c(3), c(4);
  ArraySet r(2), c(5), c(3);
  ArraySet r(2), c(2), c(5);
  ArrayInit r(3), c(2);
  ArraySet r(3), c(3), c(4);
  ArraySet r(3), c(5), c(3);
  ArraySet r(3), c(2), c(5);
  Call r(1), c(1), 2;
  Call r(0), c(0), 1;
  Ret c(4);

test_array_not_eq:
  ArrayInit r(1), c(1);
  ArraySet r(1), c(2), c(3);
  ArraySet r(1), c(4), c(2);
  ArraySet r(1), c(5), c(4);
  ArrayInit r(2), c(1);
  ArraySet r(2), c(2), c(3);
  ArraySet r(2), c(4), c(2);
  ArraySet r(2), c(1), c(4);
  Call r(0), c(0), 1;
  BEq r(0), c(3), 2;
  Call r(1), c(6), 2;
  Ret c(3);

test_sort:
  ArrayInit r(0), c(0);
  ArraySet r(0), c(1), c(2);
  ArraySet r(0), c(3), c(4);
  ArraySet r(0), c(5), c(6);
  ArraySet r(0), c(7), c(3);
  ArraySet r(0), c(4), c(5);
  Mov r(2), r(0);
  Call r(1), c(8), 2;
  Mov r(3), r(0);
  ArrayInit r(4), c(0);
  ArraySet r(4), c(4), c(2);
  ArraySet r(4), c(3), c(4);
  ArraySet r(4), c(5), c(6);
  ArraySet r(4), c(7), c(3);
  ArraySet r(4), c(1), c(5);
  Call r(2), c(10), 3;
  Call r(1), c(9), 2;
  Ret c(2);


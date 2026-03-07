count_ones:
  Mov r(1), c(0);
  BEq r(0), c(0), 7;
  And r(2), r(0), c(1);
  BNEq r(2), c(0), 2;
  Jump 2;
  IAdd r(1), r(1), c(1);
  ShiftRight r(0), r(0), c(1);
  Jump -6;
  Ret r(1);

test_count_ones_11:
  Mov r(3), c(2);
  Call r(2), c(1), 3;
  SEq r(1), r(2), c(3);
  Call r(0), c(0), 1;
  Ret c(4);

test_count_ones_14:
  Mov r(3), c(2);
  Call r(2), c(1), 3;
  SEq r(1), r(2), c(3);
  Call r(0), c(0), 1;
  Ret c(4);

test_count_ones_5:
  Mov r(3), c(2);
  Call r(2), c(1), 3;
  SEq r(1), r(2), c(3);
  Call r(0), c(0), 1;
  Ret c(4);


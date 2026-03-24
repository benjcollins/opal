test_shadowing:
  Mov r(0), c(0);
  BEq c(1), c(2), 2;
  Jump 4;
  Mov r(1), c(3);
  Mov r(3), r(1);
  Call r(2), c(4), 3;
  SEq r(2), r(0), c(6);
  Call r(1), c(5), 2;
  Ret c(7);


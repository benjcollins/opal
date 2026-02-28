test_shadowing:
  Mov r(0), c(0);
  BEq c(1), c(1), 2;
  Jump 4;
  Mov r(1), c(1);
  Mov r(3), r(1);
  Call r(2), c(2), 3;
  SEq r(2), r(0), c(0);
  Call r(1), c(3), 2;
  Ret c(4);


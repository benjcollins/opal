many_args:
  SEq r(4), r(0), c(1);
  Call r(3), c(0), 4;
  SEq r(4), r(1), c(3);
  Call r(3), c(2), 4;
  SEq r(4), r(2), c(5);
  Call r(3), c(4), 4;
  Ret c(6);

test_many_args:
  IAdd r(2), c(1), c(2);
  IAdd r(1), r(2), c(3);
  Mov r(2), c(4);
  Mov r(3), c(5);
  Call r(0), c(0), 1;
  Ret c(6);


fib_loop:
  Mov r(1), c(0);
  Mov r(2), c(1);
  Mov r(3), c(2);
  IBLte r(0), r(1), 6;
  Mov r(4), r(2);
  IAdd r(2), r(2), r(3);
  Mov r(3), r(4);
  IAdd r(1), r(1), c(3);
  Jump -5;
  Ret r(3);

fib_rec:
  BNEq r(0), c(0), 2;
  Ret c(1);
  BNEq r(0), c(2), 2;
  Ret c(3);
  ISub r(3), r(0), c(5);
  Call r(2), c(4), 3;
  ISub r(4), r(0), c(7);
  Call r(3), c(6), 4;
  IAdd r(1), r(2), r(3);
  Ret r(1);

test_fib_loop_10:
  Mov r(3), c(2);
  Call r(2), c(1), 3;
  SEq r(1), r(2), c(3);
  Call r(0), c(0), 1;
  Ret c(4);

test_fib_loop_50:
  Mov r(3), c(2);
  Call r(2), c(1), 3;
  SEq r(1), r(2), c(3);
  Call r(0), c(0), 1;
  Ret c(4);

test_fib_rec_10:
  Mov r(3), c(2);
  Call r(2), c(1), 3;
  SEq r(1), r(2), c(3);
  Call r(0), c(0), 1;
  Ret c(4);

test_fib_rec_25:
  Mov r(3), c(2);
  Call r(2), c(1), 3;
  SEq r(1), r(2), c(3);
  Call r(0), c(0), 1;
  Ret c(4);


abs:
  FBLte r(0), c(0), 2;
  Ret r(0);
  FSub r(1), c(1), r(0);
  Ret r(1);

approx_square_root:
  FDiv r(2), r(0), c(0);
  Mov r(3), c(1);
  IBLte r(1), r(3), 6;
  FDiv r(5), r(0), r(2);
  FAdd r(4), r(2), r(5);
  FDiv r(2), r(4), c(2);
  IAdd r(3), r(3), c(3);
  Jump -5;
  Ret r(2);

test_approx_square_root_123:
  Mov r(0), c(0);
  FMul r(2), r(0), r(0);
  Mov r(3), c(2);
  Call r(1), c(1), 2;
  FSub r(3), r(0), r(1);
  Call r(2), c(3), 3;
  FSLt r(4), r(2), c(5);
  Call r(3), c(4), 4;
  Ret c(6);

test_approx_square_root_738:
  Mov r(0), c(0);
  FMul r(2), r(0), r(0);
  Mov r(3), c(2);
  Call r(1), c(1), 2;
  FSub r(3), r(0), r(1);
  Call r(2), c(3), 3;
  FSLt r(4), r(2), c(5);
  Call r(3), c(4), 4;
  Ret c(6);


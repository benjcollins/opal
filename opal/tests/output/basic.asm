test_branch:
  Mov r(0), c(0);
  BEq r(0), c(0), 2;
  Call r(1), c(1), 2;
  Ret c(0);

test_float_arith:
  FAdd r(2), c(1), c(2);
  SEq r(1), r(2), c(3);
  Call r(0), c(0), 1;
  FSub r(2), c(1), c(2);
  SEq r(1), r(2), c(5);
  Call r(0), c(4), 1;
  FMul r(2), c(1), c(2);
  SEq r(1), r(2), c(7);
  Call r(0), c(6), 1;
  FDiv r(2), c(1), c(2);
  SEq r(1), r(2), c(9);
  Call r(0), c(8), 1;
  FMod r(2), c(1), c(2);
  SEq r(1), r(2), c(11);
  Call r(0), c(10), 1;
  Ret c(12);

test_float_comp_branch:
  BNEq c(0), c(1), 2;
  Call r(0), c(2), 1;
  BEq c(0), c(0), 2;
  Call r(0), c(3), 1;
  FBLte c(1), c(0), 2;
  Call r(0), c(4), 1;
  FBLte c(1), c(0), 2;
  Call r(0), c(5), 1;
  FBLt c(0), c(6), 2;
  Call r(0), c(7), 1;
  FBLt c(1), c(0), 2;
  Call r(0), c(8), 1;
  Ret c(9);

test_float_comp_branch_invert:
  BNEq c(0), c(1), 3;
  Call r(0), c(2), 1;
  Jump -2;
  BEq c(0), c(0), 3;
  Call r(0), c(3), 1;
  Jump -2;
  FBLte c(1), c(0), 3;
  Call r(0), c(4), 1;
  Jump -2;
  FBLte c(1), c(0), 3;
  Call r(0), c(5), 1;
  Jump -2;
  FBLt c(0), c(6), 3;
  Call r(0), c(7), 1;
  Jump -2;
  FBLt c(1), c(0), 3;
  Call r(0), c(8), 1;
  Jump -2;
  Ret c(9);

test_float_comp_set:
  FSLt r(1), c(2), c(1);
  Call r(0), c(0), 1;
  FSLt r(1), c(2), c(1);
  Call r(0), c(3), 1;
  FSLte r(1), c(1), c(1);
  Call r(0), c(4), 1;
  FSLte r(1), c(1), c(1);
  Call r(0), c(5), 1;
  SEq r(1), c(1), c(1);
  Call r(0), c(6), 1;
  SNEq r(1), c(1), c(2);
  Call r(0), c(7), 1;
  Ret c(8);

test_int_arith:
  IAdd r(2), c(1), c(2);
  SEq r(1), r(2), c(3);
  Call r(0), c(0), 1;
  ISub r(2), c(1), c(2);
  SEq r(1), r(2), c(5);
  Call r(0), c(4), 1;
  IMul r(2), c(1), c(2);
  SEq r(1), r(2), c(7);
  Call r(0), c(6), 1;
  IDiv r(2), c(9), c(2);
  SEq r(1), r(2), c(2);
  Call r(0), c(8), 1;
  IMod r(2), c(1), c(2);
  SEq r(1), r(2), c(11);
  Call r(0), c(10), 1;
  Ret c(12);

test_int_comp:
  ISLt r(1), c(2), c(1);
  Call r(0), c(0), 1;
  ISLt r(1), c(2), c(1);
  Call r(0), c(3), 1;
  ISLte r(1), c(1), c(1);
  Call r(0), c(4), 1;
  ISLte r(1), c(1), c(1);
  Call r(0), c(5), 1;
  SEq r(1), c(1), c(1);
  Call r(0), c(6), 1;
  SNEq r(1), c(1), c(2);
  Call r(0), c(7), 1;
  Ret c(8);

test_int_comp_branch:
  BNEq c(0), c(1), 2;
  Call r(0), c(2), 1;
  BEq c(0), c(0), 2;
  Call r(0), c(3), 1;
  IBLte c(1), c(0), 2;
  Call r(0), c(4), 1;
  IBLte c(1), c(0), 2;
  Call r(0), c(5), 1;
  IBLt c(0), c(6), 2;
  Call r(0), c(7), 1;
  IBLt c(1), c(0), 2;
  Call r(0), c(8), 1;
  Ret c(9);

test_int_comp_branch_invert:
  BNEq c(0), c(1), 3;
  Call r(0), c(2), 1;
  Jump -2;
  BEq c(0), c(0), 3;
  Call r(0), c(3), 1;
  Jump -2;
  IBLte c(1), c(0), 3;
  Call r(0), c(4), 1;
  Jump -2;
  IBLte c(1), c(0), 3;
  Call r(0), c(5), 1;
  Jump -2;
  IBLt c(0), c(6), 3;
  Call r(0), c(7), 1;
  Jump -2;
  IBLt c(1), c(0), 3;
  Call r(0), c(8), 1;
  Jump -2;
  Ret c(9);

test_logical_op:
  BEq c(2), c(1), 2;
  BEq c(2), c(2), 3;
  Mov r(1), c(1);
  Jump 2;
  Mov r(1), c(2);
  Call r(0), c(0), 1;
  BEq c(2), c(1), 3;
  BEq c(1), c(1), 2;
  Call r(0), c(3), 1;
  BEq c(1), c(1), 3;
  BEq c(2), c(1), 2;
  Call r(0), c(4), 1;
  BEq c(1), c(1), 3;
  BEq c(1), c(1), 2;
  Call r(0), c(5), 1;
  BEq c(2), c(2), 4;
  BEq c(2), c(2), 3;
  Mov r(1), c(1);
  Jump 2;
  Mov r(1), c(2);
  Call r(0), c(6), 1;
  BEq c(2), c(2), 4;
  BEq c(1), c(2), 3;
  Mov r(1), c(1);
  Jump 2;
  Mov r(1), c(2);
  Call r(0), c(7), 1;
  BEq c(1), c(2), 4;
  BEq c(2), c(2), 3;
  Mov r(1), c(1);
  Jump 2;
  Mov r(1), c(2);
  Call r(0), c(8), 1;
  BEq c(1), c(2), 2;
  BEq c(1), c(1), 2;
  Call r(0), c(9), 1;
  Ret c(1);

test_logical_op_invert:
  BEq c(0), c(1), 4;
  BEq c(1), c(1), 3;
  Call r(0), c(2), 1;
  Jump -3;
  BEq c(1), c(1), 4;
  BEq c(0), c(1), 3;
  Call r(0), c(3), 1;
  Jump -3;
  BEq c(1), c(1), 4;
  BEq c(1), c(1), 3;
  Call r(0), c(4), 1;
  Jump -3;
  BEq c(1), c(0), 2;
  BEq c(1), c(1), 3;
  Call r(0), c(5), 1;
  Jump -3;
  Mov r(0), c(0);
  BEq r(0), c(1), 4;
  BEq r(0), c(1), 3;
  Mov r(0), c(1);
  Jump -3;
  SEq r(2), r(0), c(1);
  Call r(1), c(6), 2;
  Mov r(1), c(0);
  BEq r(1), c(0), 2;
  BEq r(1), c(1), 3;
  Mov r(1), c(1);
  Jump -3;
  SEq r(3), r(1), c(1);
  Call r(2), c(7), 3;
  Mov r(2), c(0);
  BEq r(2), c(0), 2;
  BEq c(1), c(1), 3;
  Mov r(2), c(1);
  Jump -3;
  SEq r(4), r(2), c(1);
  Call r(3), c(8), 4;
  Mov r(3), c(0);
  BEq c(1), c(0), 2;
  BEq r(3), c(1), 3;
  Mov r(3), c(1);
  Jump -3;
  SEq r(5), r(3), c(1);
  Call r(4), c(9), 5;
  Ret c(1);

test_prefix_ops:
  SEq r(1), c(1), c(1);
  Call r(0), c(0), 1;
  XOr r(2), c(4), c(3);
  ISub r(3), c(1), c(5);
  SEq r(1), r(2), r(3);
  Call r(0), c(2), 1;
  Mov r(2), c(3);
  SEq r(1), r(2), c(3);
  Call r(0), c(6), 1;
  FSub r(2), c(8), c(9);
  FSub r(3), c(1), c(10);
  SEq r(1), r(2), r(3);
  Call r(0), c(7), 1;
  ISub r(2), c(12), c(3);
  ISub r(3), c(1), c(13);
  SEq r(1), r(2), r(3);
  Call r(0), c(11), 1;
  Ret c(1);

test_unit:
  Ret c(0);


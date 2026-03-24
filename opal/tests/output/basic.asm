test_branch:
  Mov r(0), c(0);
  BEq r(0), c(1), 2;
  Call r(1), c(2), 2;
  Ret c(3);

test_float_arith:
  FAdd r(2), c(1), c(2);
  SEq r(1), r(2), c(3);
  Call r(0), c(0), 1;
  FSub r(2), c(5), c(6);
  SEq r(1), r(2), c(7);
  Call r(0), c(4), 1;
  FMul r(2), c(9), c(10);
  SEq r(1), r(2), c(11);
  Call r(0), c(8), 1;
  FDiv r(2), c(13), c(14);
  SEq r(1), r(2), c(15);
  Call r(0), c(12), 1;
  FMod r(2), c(17), c(18);
  SEq r(1), r(2), c(19);
  Call r(0), c(16), 1;
  Ret c(20);

test_float_comp_branch:
  BNEq c(0), c(1), 2;
  Call r(0), c(2), 1;
  BEq c(3), c(4), 2;
  Call r(0), c(5), 1;
  FBLte c(7), c(6), 2;
  Call r(0), c(8), 1;
  FBLte c(9), c(10), 2;
  Call r(0), c(11), 1;
  FBLt c(12), c(13), 2;
  Call r(0), c(14), 1;
  FBLt c(16), c(15), 2;
  Call r(0), c(17), 1;
  Ret c(18);

test_float_comp_branch_invert:
  BNEq c(0), c(1), 3;
  Call r(0), c(2), 1;
  Jump -2;
  BEq c(3), c(4), 3;
  Call r(0), c(5), 1;
  Jump -2;
  FBLte c(7), c(6), 3;
  Call r(0), c(8), 1;
  Jump -2;
  FBLte c(9), c(10), 3;
  Call r(0), c(11), 1;
  Jump -2;
  FBLt c(12), c(13), 3;
  Call r(0), c(14), 1;
  Jump -2;
  FBLt c(16), c(15), 3;
  Call r(0), c(17), 1;
  Jump -2;
  Ret c(18);

test_float_comp_set:
  FSLt r(1), c(2), c(1);
  Call r(0), c(0), 1;
  FSLt r(1), c(4), c(5);
  Call r(0), c(3), 1;
  FSLte r(1), c(8), c(7);
  Call r(0), c(6), 1;
  FSLte r(1), c(10), c(11);
  Call r(0), c(9), 1;
  SEq r(1), c(13), c(14);
  Call r(0), c(12), 1;
  SNEq r(1), c(16), c(17);
  Call r(0), c(15), 1;
  Ret c(18);

test_int_arith:
  IAdd r(2), c(1), c(2);
  SEq r(1), r(2), c(3);
  Call r(0), c(0), 1;
  ISub r(2), c(5), c(6);
  SEq r(1), r(2), c(7);
  Call r(0), c(4), 1;
  IMul r(2), c(9), c(10);
  SEq r(1), r(2), c(11);
  Call r(0), c(8), 1;
  IDiv r(2), c(13), c(14);
  SEq r(1), r(2), c(15);
  Call r(0), c(12), 1;
  IMod r(2), c(17), c(18);
  SEq r(1), r(2), c(19);
  Call r(0), c(16), 1;
  Ret c(20);

test_int_comp:
  ISLt r(1), c(2), c(1);
  Call r(0), c(0), 1;
  ISLt r(1), c(4), c(5);
  Call r(0), c(3), 1;
  ISLte r(1), c(8), c(7);
  Call r(0), c(6), 1;
  ISLte r(1), c(10), c(11);
  Call r(0), c(9), 1;
  SEq r(1), c(13), c(14);
  Call r(0), c(12), 1;
  SNEq r(1), c(16), c(17);
  Call r(0), c(15), 1;
  Ret c(18);

test_int_comp_branch:
  BNEq c(0), c(1), 2;
  Call r(0), c(2), 1;
  BEq c(3), c(4), 2;
  Call r(0), c(5), 1;
  IBLte c(7), c(6), 2;
  Call r(0), c(8), 1;
  IBLte c(9), c(10), 2;
  Call r(0), c(11), 1;
  IBLt c(12), c(13), 2;
  Call r(0), c(14), 1;
  IBLt c(16), c(15), 2;
  Call r(0), c(17), 1;
  Ret c(18);

test_int_comp_branch_invert:
  BNEq c(0), c(1), 3;
  Call r(0), c(2), 1;
  Jump -2;
  BEq c(3), c(4), 3;
  Call r(0), c(5), 1;
  Jump -2;
  IBLte c(7), c(6), 3;
  Call r(0), c(8), 1;
  Jump -2;
  IBLte c(9), c(10), 3;
  Call r(0), c(11), 1;
  Jump -2;
  IBLt c(12), c(13), 3;
  Call r(0), c(14), 1;
  Jump -2;
  IBLt c(16), c(15), 3;
  Call r(0), c(17), 1;
  Jump -2;
  Ret c(18);

test_logical_op:
  BEq c(3), c(4), 2;
  BEq c(5), c(6), 3;
  Mov r(1), c(1);
  Jump 2;
  Mov r(1), c(2);
  Call r(0), c(0), 1;
  BEq c(7), c(8), 3;
  BEq c(9), c(10), 2;
  Call r(0), c(11), 1;
  BEq c(12), c(13), 3;
  BEq c(14), c(15), 2;
  Call r(0), c(16), 1;
  BEq c(17), c(18), 3;
  BEq c(19), c(20), 2;
  Call r(0), c(21), 1;
  BEq c(25), c(26), 4;
  BEq c(27), c(28), 3;
  Mov r(1), c(23);
  Jump 2;
  Mov r(1), c(24);
  Call r(0), c(22), 1;
  BEq c(32), c(33), 4;
  BEq c(34), c(35), 3;
  Mov r(1), c(30);
  Jump 2;
  Mov r(1), c(31);
  Call r(0), c(29), 1;
  BEq c(39), c(40), 4;
  BEq c(41), c(42), 3;
  Mov r(1), c(37);
  Jump 2;
  Mov r(1), c(38);
  Call r(0), c(36), 1;
  BEq c(43), c(44), 2;
  BEq c(45), c(46), 2;
  Call r(0), c(47), 1;
  Ret c(48);

test_logical_op_invert:
  BEq c(0), c(1), 4;
  BEq c(2), c(3), 3;
  Call r(0), c(4), 1;
  Jump -3;
  BEq c(5), c(6), 4;
  BEq c(7), c(8), 3;
  Call r(0), c(9), 1;
  Jump -3;
  BEq c(10), c(11), 4;
  BEq c(12), c(13), 3;
  Call r(0), c(14), 1;
  Jump -3;
  BEq c(15), c(16), 2;
  BEq c(17), c(18), 3;
  Call r(0), c(19), 1;
  Jump -3;
  Mov r(0), c(20);
  BEq r(0), c(21), 4;
  BEq r(0), c(22), 3;
  Mov r(0), c(23);
  Jump -3;
  SEq r(2), r(0), c(25);
  Call r(1), c(24), 2;
  Mov r(1), c(26);
  BEq r(1), c(27), 2;
  BEq r(1), c(28), 3;
  Mov r(1), c(29);
  Jump -3;
  SEq r(3), r(1), c(31);
  Call r(2), c(30), 3;
  Mov r(2), c(32);
  BEq r(2), c(33), 2;
  BEq c(34), c(35), 3;
  Mov r(2), c(36);
  Jump -3;
  SEq r(4), r(2), c(38);
  Call r(3), c(37), 4;
  Mov r(3), c(39);
  BEq c(40), c(41), 2;
  BEq r(3), c(42), 3;
  Mov r(3), c(43);
  Jump -3;
  SEq r(5), r(3), c(45);
  Call r(4), c(44), 5;
  Ret c(46);

test_prefix_ops:
  SEq r(1), c(1), c(2);
  Call r(0), c(0), 1;
  XOr r(2), c(5), c(4);
  ISub r(3), c(7), c(6);
  SEq r(1), r(2), r(3);
  Call r(0), c(3), 1;
  Mov r(2), c(9);
  SEq r(1), r(2), c(10);
  Call r(0), c(8), 1;
  FSub r(2), c(12), c(13);
  FSub r(3), c(15), c(14);
  SEq r(1), r(2), r(3);
  Call r(0), c(11), 1;
  ISub r(2), c(17), c(18);
  ISub r(3), c(20), c(19);
  SEq r(1), r(2), r(3);
  Call r(0), c(16), 1;
  Ret c(21);

test_unit:
  Ret c(1);


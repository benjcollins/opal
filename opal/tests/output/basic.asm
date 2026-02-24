test_branch:
  MOV r(0), c(0);
  BEQ r(0), c(0), 2;
  CALL r(1), c(1), 2;
  RET c(0);

test_float_arith:
  FADD r(2), c(1), c(2);
  SEQ r(1), r(2), c(3);
  CALL r(0), c(0), 1;
  FSUB r(2), c(1), c(2);
  SEQ r(1), r(2), c(5);
  CALL r(0), c(4), 1;
  FMUL r(2), c(1), c(2);
  SEQ r(1), r(2), c(7);
  CALL r(0), c(6), 1;
  FDIV r(2), c(1), c(2);
  SEQ r(1), r(2), c(9);
  CALL r(0), c(8), 1;
  FMOD r(2), c(1), c(2);
  SEQ r(1), r(2), c(11);
  CALL r(0), c(10), 1;
  RET c(12);

test_float_comp_branch:
  BNE c(0), c(1), 2;
  CALL r(0), c(2), 1;
  BEQ c(0), c(0), 2;
  CALL r(0), c(3), 1;
  FBLE c(1), c(0), 2;
  CALL r(0), c(4), 1;
  FBLE c(1), c(0), 2;
  CALL r(0), c(5), 1;
  FBLT c(0), c(6), 2;
  CALL r(0), c(7), 1;
  FBLT c(1), c(0), 2;
  CALL r(0), c(8), 1;
  RET c(9);

test_float_comp_branch_invert:
  BNE c(0), c(1), 3;
  CALL r(0), c(2), 1;
  JMP -2;
  BEQ c(0), c(0), 3;
  CALL r(0), c(3), 1;
  JMP -2;
  FBLE c(1), c(0), 3;
  CALL r(0), c(4), 1;
  JMP -2;
  FBLE c(1), c(0), 3;
  CALL r(0), c(5), 1;
  JMP -2;
  FBLT c(0), c(6), 3;
  CALL r(0), c(7), 1;
  JMP -2;
  FBLT c(1), c(0), 3;
  CALL r(0), c(8), 1;
  JMP -2;
  RET c(9);

test_float_comp_set:
  FSLT r(1), c(2), c(1);
  CALL r(0), c(0), 1;
  FSLT r(1), c(2), c(1);
  CALL r(0), c(3), 1;
  FSLE r(1), c(1), c(1);
  CALL r(0), c(4), 1;
  FSLE r(1), c(1), c(1);
  CALL r(0), c(5), 1;
  SEQ r(1), c(1), c(1);
  CALL r(0), c(6), 1;
  SNE r(1), c(1), c(2);
  CALL r(0), c(7), 1;
  RET c(8);

test_int_arith:
  IADD r(2), c(1), c(2);
  SEQ r(1), r(2), c(3);
  CALL r(0), c(0), 1;
  ISUB r(2), c(1), c(2);
  SEQ r(1), r(2), c(5);
  CALL r(0), c(4), 1;
  IMUL r(2), c(1), c(2);
  SEQ r(1), r(2), c(7);
  CALL r(0), c(6), 1;
  IDIV r(2), c(9), c(2);
  SEQ r(1), r(2), c(2);
  CALL r(0), c(8), 1;
  IMOD r(2), c(1), c(2);
  SEQ r(1), r(2), c(11);
  CALL r(0), c(10), 1;
  RET c(12);

test_int_comp:
  ISLT r(1), c(2), c(1);
  CALL r(0), c(0), 1;
  ISLT r(1), c(2), c(1);
  CALL r(0), c(3), 1;
  ISLE r(1), c(1), c(1);
  CALL r(0), c(4), 1;
  ISLE r(1), c(1), c(1);
  CALL r(0), c(5), 1;
  SEQ r(1), c(1), c(1);
  CALL r(0), c(6), 1;
  SNE r(1), c(1), c(2);
  CALL r(0), c(7), 1;
  RET c(8);

test_int_comp_branch:
  BNE c(0), c(1), 2;
  CALL r(0), c(2), 1;
  BEQ c(0), c(0), 2;
  CALL r(0), c(3), 1;
  IBLE c(1), c(0), 2;
  CALL r(0), c(4), 1;
  IBLE c(1), c(0), 2;
  CALL r(0), c(5), 1;
  IBLT c(0), c(6), 2;
  CALL r(0), c(7), 1;
  IBLT c(1), c(0), 2;
  CALL r(0), c(8), 1;
  RET c(9);

test_int_comp_branch_invert:
  BNE c(0), c(1), 3;
  CALL r(0), c(2), 1;
  JMP -2;
  BEQ c(0), c(0), 3;
  CALL r(0), c(3), 1;
  JMP -2;
  IBLE c(1), c(0), 3;
  CALL r(0), c(4), 1;
  JMP -2;
  IBLE c(1), c(0), 3;
  CALL r(0), c(5), 1;
  JMP -2;
  IBLT c(0), c(6), 3;
  CALL r(0), c(7), 1;
  JMP -2;
  IBLT c(1), c(0), 3;
  CALL r(0), c(8), 1;
  JMP -2;
  RET c(9);

test_logical_op:
  BEQ c(2), c(1), 2;
  BEQ c(2), c(2), 3;
  MOV r(1), c(1);
  JMP 2;
  MOV r(1), c(2);
  CALL r(0), c(0), 1;
  BEQ c(2), c(1), 3;
  BEQ c(1), c(1), 2;
  CALL r(0), c(3), 1;
  BEQ c(1), c(1), 3;
  BEQ c(2), c(1), 2;
  CALL r(0), c(4), 1;
  BEQ c(1), c(1), 3;
  BEQ c(1), c(1), 2;
  CALL r(0), c(5), 1;
  BEQ c(2), c(2), 4;
  BEQ c(2), c(2), 3;
  MOV r(1), c(1);
  JMP 2;
  MOV r(1), c(2);
  CALL r(0), c(6), 1;
  BEQ c(2), c(2), 4;
  BEQ c(1), c(2), 3;
  MOV r(1), c(1);
  JMP 2;
  MOV r(1), c(2);
  CALL r(0), c(7), 1;
  BEQ c(1), c(2), 4;
  BEQ c(2), c(2), 3;
  MOV r(1), c(1);
  JMP 2;
  MOV r(1), c(2);
  CALL r(0), c(8), 1;
  BEQ c(1), c(2), 2;
  BEQ c(1), c(1), 2;
  CALL r(0), c(9), 1;
  RET c(1);

test_logical_op_invert:
  BEQ c(0), c(1), 4;
  BEQ c(1), c(1), 3;
  CALL r(0), c(2), 1;
  JMP -3;
  BEQ c(1), c(1), 4;
  BEQ c(0), c(1), 3;
  CALL r(0), c(3), 1;
  JMP -3;
  BEQ c(1), c(1), 4;
  BEQ c(1), c(1), 3;
  CALL r(0), c(4), 1;
  JMP -3;
  BEQ c(1), c(0), 2;
  BEQ c(1), c(1), 3;
  CALL r(0), c(5), 1;
  JMP -3;
  MOV r(0), c(0);
  BEQ r(0), c(1), 4;
  BEQ r(0), c(1), 3;
  MOV r(0), c(1);
  JMP -3;
  SEQ r(2), r(0), c(1);
  CALL r(1), c(6), 2;
  MOV r(1), c(0);
  BEQ r(1), c(0), 2;
  BEQ r(1), c(1), 3;
  MOV r(1), c(1);
  JMP -3;
  SEQ r(3), r(1), c(1);
  CALL r(2), c(7), 3;
  MOV r(2), c(0);
  BEQ r(2), c(0), 2;
  BEQ c(1), c(1), 3;
  MOV r(2), c(1);
  JMP -3;
  SEQ r(4), r(2), c(1);
  CALL r(3), c(8), 4;
  MOV r(3), c(0);
  BEQ c(1), c(0), 2;
  BEQ r(3), c(1), 3;
  MOV r(3), c(1);
  JMP -3;
  SEQ r(5), r(3), c(1);
  CALL r(4), c(9), 5;
  RET c(1);

test_prefix_ops:
  SEQ r(1), c(1), c(1);
  CALL r(0), c(0), 1;
  XOR r(2), c(4), c(3);
  ISUB r(3), c(1), c(5);
  SEQ r(1), r(2), r(3);
  CALL r(0), c(2), 1;
  MOV r(2), c(3);
  SEQ r(1), r(2), c(3);
  CALL r(0), c(6), 1;
  FSUB r(2), c(8), c(9);
  FSUB r(3), c(1), c(10);
  SEQ r(1), r(2), r(3);
  CALL r(0), c(7), 1;
  ISUB r(2), c(12), c(3);
  ISUB r(3), c(1), c(13);
  SEQ r(1), r(2), r(3);
  CALL r(0), c(11), 1;
  RET c(1);

test_unit:
  RET c(0);


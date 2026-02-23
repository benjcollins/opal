count_ones:
  MOV r(1), c(0);
  BEQ r(0), c(0), 7;
  AND r(2), r(0), c(1);
  BNE r(2), c(0), 2;
  JMP 2;
  IADD r(1), r(1), c(1);
  SHR r(0), r(0), c(1);
  JMP -6;
  RET r(1);

test_count_ones_11:
  MOV r(3), c(2);
  CALL r(2), c(1), 3;
  SEQ r(1), r(2), c(3);
  CALL r(0), c(0), 1;
  RET c(4);

test_count_ones_14:
  MOV r(3), c(2);
  CALL r(2), c(1), 3;
  SEQ r(1), r(2), c(3);
  CALL r(0), c(0), 1;
  RET c(4);

test_count_ones_5:
  MOV r(3), c(2);
  CALL r(2), c(1), 3;
  SEQ r(1), r(2), c(3);
  CALL r(0), c(0), 1;
  RET c(4);


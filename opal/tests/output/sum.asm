sum:
  MOV r(1), c(0);
  MOV r(2), c(0);
  MOV r(4), r(0);
  CALL r(3), c(1), 4;
  IBLE r(3), r(1), 5;
  GET_ARRAY r(4), r(0), r(1);
  IADD r(2), r(2), r(4);
  IADD r(1), r(1), c(2);
  JMP -6;
  RET r(2);

test_sum:
  MOV r(2), c(1);
  MOV r(3), c(2);
  MOV r(4), c(3);
  MOV r(5), c(4);
  MOV r(6), c(5);
  INIT_ARRAY r(1), 2, 5;
  CALL r(0), c(0), 1;
  SEQ r(2), r(0), c(7);
  CALL r(1), c(6), 2;
  RET c(8);


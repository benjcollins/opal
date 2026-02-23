array_eq:
  MOV r(3), r(0);
  CALL r(2), c(0), 3;
  MOV r(4), r(1);
  CALL r(3), c(1), 4;
  BEQ r(2), r(3), 2;
  RET c(2);
  MOV r(4), c(2);
  MOV r(6), r(0);
  CALL r(5), c(3), 6;
  IBLE r(5), r(4), 7;
  GET_ARRAY r(6), r(0), r(4);
  GET_ARRAY r(7), r(1), r(4);
  BEQ r(6), r(7), 2;
  RET c(2);
  IADD r(4), r(4), c(4);
  JMP -8;
  RET c(4);

sort:
  MOV r(1), c(0);
  MOV r(3), r(0);
  CALL r(2), c(1), 3;
  IBLE r(2), r(1), 17;
  IADD r(3), r(1), c(2);
  MOV r(5), r(0);
  CALL r(4), c(3), 5;
  IBLE r(4), r(3), 11;
  GET_ARRAY r(5), r(0), r(1);
  GET_ARRAY r(6), r(0), r(3);
  IBLT r(6), r(5), 2;
  JMP 5;
  GET_ARRAY r(7), r(0), r(1);
  GET_ARRAY r(8), r(0), r(3);
  SET_ARRAY r(0), r(8), r(1);
  SET_ARRAY r(0), r(7), r(3);
  IADD r(3), r(3), c(2);
  JMP -12;
  IADD r(1), r(1), c(2);
  JMP -18;
  RET c(0);

test_array_diff_len:
  MOV r(2), c(1);
  MOV r(3), c(2);
  INIT_ARRAY r(1), 2, 2;
  MOV r(3), c(1);
  MOV r(4), c(2);
  MOV r(5), c(3);
  INIT_ARRAY r(2), 3, 3;
  CALL r(0), c(0), 1;
  BEQ r(0), c(4), 2;
  CALL r(1), c(5), 2;
  RET c(4);

test_array_eq:
  MOV r(3), c(2);
  MOV r(4), c(3);
  MOV r(5), c(4);
  INIT_ARRAY r(2), 3, 3;
  MOV r(4), c(2);
  MOV r(5), c(3);
  MOV r(6), c(4);
  INIT_ARRAY r(3), 4, 3;
  CALL r(1), c(1), 2;
  CALL r(0), c(0), 1;
  RET c(5);

test_array_not_eq:
  MOV r(2), c(1);
  MOV r(3), c(2);
  MOV r(4), c(3);
  INIT_ARRAY r(1), 2, 3;
  MOV r(3), c(1);
  MOV r(4), c(2);
  MOV r(5), c(4);
  INIT_ARRAY r(2), 3, 3;
  CALL r(0), c(0), 1;
  BEQ r(0), c(5), 2;
  CALL r(1), c(6), 2;
  RET c(5);

test_sort:
  MOV r(1), c(0);
  MOV r(2), c(1);
  MOV r(3), c(2);
  MOV r(4), c(3);
  MOV r(5), c(4);
  INIT_ARRAY r(0), 1, 5;
  MOV r(2), r(0);
  CALL r(1), c(5), 2;
  MOV r(3), r(0);
  MOV r(5), c(4);
  MOV r(6), c(1);
  MOV r(7), c(2);
  MOV r(8), c(3);
  MOV r(9), c(0);
  INIT_ARRAY r(4), 5, 5;
  CALL r(2), c(7), 3;
  CALL r(1), c(6), 2;
  RET c(8);


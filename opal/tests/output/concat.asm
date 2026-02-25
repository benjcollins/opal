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

concat:
  MOV r(5), r(0);
  CALL r(4), c(0), 5;
  MOV r(6), r(1);
  CALL r(5), c(1), 6;
  IADD r(3), r(4), r(5);
  NEW_ARRAY r(2), r(3);
  IBLE r(3), r(6), 4;
  SET_ARRAY r(2), c(2), r(6);
  IADD r(6), r(6), c(3);
  JMP -3;
  MOV r(4), r(0);
  MOV r(5), r(2);
  MOV r(6), c(2);
  MOV r(7), c(2);
  MOV r(9), r(0);
  CALL r(8), c(5), 9;
  CALL r(3), c(4), 4;
  MOV r(4), r(1);
  MOV r(5), r(2);
  MOV r(6), c(2);
  MOV r(8), r(0);
  CALL r(7), c(7), 8;
  MOV r(9), r(1);
  CALL r(8), c(8), 9;
  CALL r(3), c(6), 4;
  RET r(2);

copy:
  MOV r(5), c(0);
  IBLE r(4), r(5), 7;
  IADD r(6), r(3), r(5);
  IADD r(8), r(2), r(5);
  GET_ARRAY r(7), r(0), r(8);
  SET_ARRAY r(1), r(7), r(6);
  IADD r(5), r(5), c(1);
  JMP -6;
  RET c(0);

test_concat:
  NEW_ARRAY r(0), c(0);
  SET_ARRAY r(0), c(1), c(2);
  SET_ARRAY r(0), c(3), c(1);
  SET_ARRAY r(0), c(0), c(3);
  NEW_ARRAY r(1), c(3);
  SET_ARRAY r(1), c(4), c(2);
  SET_ARRAY r(1), c(5), c(1);
  MOV r(3), r(0);
  MOV r(4), r(1);
  CALL r(2), c(6), 3;
  MOV r(5), r(2);
  NEW_ARRAY r(6), c(5);
  SET_ARRAY r(6), c(1), c(2);
  SET_ARRAY r(6), c(3), c(1);
  SET_ARRAY r(6), c(0), c(3);
  SET_ARRAY r(6), c(4), c(0);
  SET_ARRAY r(6), c(5), c(4);
  CALL r(4), c(8), 5;
  CALL r(3), c(7), 4;
  RET c(2);


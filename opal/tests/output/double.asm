double:
  MOV r(1), c(0);
  MOV r(3), r(0);
  CALL r(2), c(1), 3;
  IBLE r(2), r(1), 6;
  GET_ARRAY r(3), r(0), r(1);
  IMUL r(3), r(3), c(2);
  SET_ARRAY r(0), r(3), r(1);
  IADD r(1), r(1), c(3);
  JMP -7;
  RET c(0);

test_double:
  NEW_ARRAY r(0), c(0);
  SET_ARRAY r(0), c(1), c(2);
  SET_ARRAY r(0), c(3), c(1);
  SET_ARRAY r(0), c(0), c(3);
  MOV r(2), r(0);
  CALL r(1), c(4), 2;
  GET_ARRAY r(3), r(0), c(2);
  SEQ r(2), r(3), c(3);
  CALL r(1), c(5), 2;
  GET_ARRAY r(3), r(0), c(1);
  SEQ r(2), r(3), c(7);
  CALL r(1), c(6), 2;
  GET_ARRAY r(3), r(0), c(3);
  SEQ r(2), r(3), c(9);
  CALL r(1), c(8), 2;
  RET c(2);


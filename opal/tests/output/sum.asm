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
  NEW_ARRAY r(1), c(1);
  SET_ARRAY r(1), c(2), c(3);
  SET_ARRAY r(1), c(4), c(2);
  SET_ARRAY r(1), c(5), c(4);
  SET_ARRAY r(1), c(6), c(5);
  SET_ARRAY r(1), c(1), c(6);
  CALL r(0), c(0), 1;
  SEQ r(2), r(0), c(8);
  CALL r(1), c(7), 2;
  RET c(3);


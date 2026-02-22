test_shadowing:
  MOV r(0), c(0);
  BEQ c(1), c(1), 2;
  JMP 4;
  MOV r(1), c(1);
  MOV r(3), r(1);
  CALL r(2), c(2), 3;
  SEQ
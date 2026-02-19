fib_loop:
  MOV r(1), c(0);
  MOV r(2), c(1);
  MOV r(3), c(0);
  IBLE r(0), r(1), 6;
  MOV r(4), r(2);
  IADD r(2), r(2), r(3);
  MOV r(3), r(4);
  IADD r(1), r(1), c(1);
  JMP -5;
  RET r(3);

fib_rec:
  BNE r(0), c(0), 2;
  RET c(0);
  BNE r(0), c(1), 2;
  RET c(1);
  ISUB r(3), r(0), c(1);
  CALL r(2), c(2), 3;
  ISUB r(4), r(0), c(4);
  CALL r(3), c(3), 4;
  IADD r(1), r(2), r(3);
  RET r(1);

test_fib_loop_10:
  MOV r(3), c(2);
  CALL r(2), c(1), 3;
  SEQ r(1), r(2), c(3);
  CALL r(0), c(0), 1;
  RET c(4);

test_fib_loop_50:
  MOV r(3), c(2);
  CALL r(2), c(1), 3;
  SEQ r(1), r(2), c(3);
  CALL r(0), c(0), 1;
  RET c(4);

test_fib_rec_10:
  MOV r(3), c(2);
  CALL r(2), c(1), 3;
  SEQ r(1), r(2), c(3);
  CALL r(0), c(0), 1;
  RET c(4);

test_fib_rec_25:
  MOV r(3), c(2);
  CALL r(2), c(1), 3;
  SEQ r(1), r(2), c(3);
  CALL r(0), c(0), 1;
  RET c(4);


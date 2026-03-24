float format
s - sign
e - exponent
m - mantissa
seeeeeeeeeeemmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm

the ones are all required to encode that this is a NaN so float values can be encoded normally
the zero prefix is necessary so that all ints are encoded contiguously and so they don't require a decode step just a bounds check
will probably have to remove bitwise operations since they no longer make sense
01111111111111mmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm

# Int

no decoding or encoding operations are required however range is slightly reduced
maximum integer value is (0xdffbffffffffffff)
so need to check bounds after all operations this can be done in the Value::from_int constructor easily

# Float

encoded as normal full range allowed only NaN values used for encoding pointers

# Pointer

must be prefixed with
01111111_111111xx_xxxxxxxx_xxxxxxxx_xxxxxxxx_xxxxxxxx_xxxxxxxx_xxxxxxxx

v & (0xfffc << 48) == (0x7ffc << 48)

requires masking and unmasking on each operation
can be simply added to the from_object and to_object methods

# Bool

no changes required


seeeeeee_eeeemmmm_mmmmmmmm_mmmmmmmm_mmmmmmmm_mmmmmmmm_mmmmmmmm_mmmmmmmm
01111111_111111bb_xxxxxxxx_xxxxxxxx_xxxxxxxx_xxxxxxxx_xxxxxxxx_xxxxxxxx pointers

00000000_00000011_11111111_11111111_11111111_11111111_11111111_11111111
00_00_ff_ff_ff_ff_ff_ff

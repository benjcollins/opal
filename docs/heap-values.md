
heap values

header 32 bits

tag
- bytes (len 28 bits)
- record/tuple (gc bits 28 bits)
- array len gc bit (1 bit gc bit, 27 bit len)
- stack

tag - 2 bits
gc - 2 bits


ttggxxxx xxxxxxxx xxxxxxxx xxxxxxxx

xxxxxxxx xxxxxxxx xxxxxxxx xxxxxxdd
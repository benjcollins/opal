
heap values

- next allocation usize bits
- tag 2 bit
- marked 1 bit
- refcount 29 bits
- payload 32 bits
  - 32 bit len
  - 1 bit gc, 31 bit len
  - 32 bit one per field

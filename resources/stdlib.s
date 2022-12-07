print_dec:
    li  a7, 1          # set rars ecall identifier to 1 for print_int
    # the value to print should already be in a0, so nothing else needs to be done.
    ecall
    ret

print_str:
    li  a7, 4          # set rars ecall identifier to 1 for print_string
    # the address to print should already be in a0, so nothing else needs to be done.
    ecall
    ret
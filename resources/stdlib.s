print_dec:
    li  a7, 1 # set rars ecall identifier to 1 for print_int
    # the value to print should already be in a0, so nothing else needs to be done.
    ecall
    ret
print_str:
    li  a7, 4 # set rars ecall identifier to 4 for print_string
    # the address to print should already be in a0, so nothing else needs to be done.
    ecall
    ret
read_int:
    li  a7, 5 # set rars ecall identifier to 5 for read int
    ecall     # ecall will place result in a0
    ret
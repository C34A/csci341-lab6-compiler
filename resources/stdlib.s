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
read_string:
    li a7, 8 # set rars ecall identifier to 8 for read int
    ecall    # ecall assumes args already in a0 and a1
    ret
sbrk: # im not going to comment all of these since theyre really all the same.
    li a7, 9
    ecall
    ret
exit:
    li a7, 10
    ecall
    ret
print_char:
    li a7, 11
    ecall
    ret
read_char:
    li a7, 12
    ecall
    ret
get_cwd:
    li a7, 17
    ecall
    ret
get_time:
    li a7, 30
    ecall
    ret
sleep:
    li a7, 32
    ecall
    ret
print_hex:
    li a7, 34
    ecall
    ret
print_bin:
    li a7, 35
    ecall
    ret
print_unsigned:
    li a7, 36
    ecall
    ret
rand_seed:
    li a7, 40
    ecall
    ret
rand_int:
    li a7, 41
    ecall
    ret
rand_int_range:
    li a7, 42
    ecall
    ret
confirm_dialog:
    li a7, 50
    ecall
    ret
close:
    li a7, 57
    ecall
    ret
read_fd:
    li a7, 63
    ecall
    ret
write_fd:
    li a7, 64
    ecall
    ret
exit2:
    li a7, 93
    ecall
    ret
open:
    li a7, 1024
    ecall
    ret
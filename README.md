# Lab 6: Compiler
```
CSCI 341 C
2022-12-07
```

This is a compiler (written in rust) for an extremely rudimentary programming language targeting
the risc-v (RV32IM) instruction set, and RARS in particular. The language itself
is somewhat similar to rust in syntax, but much simpler in semantics. For 
example, it does not have any type system and has no checks for correctness.

The following is a short example:
```rust
let num = 0;
print_str("Enter a number: ");
set num = read_int();
if !(num & 1) {
  print_str("even!\n");
} else {
  if num % 3 == 0 {
    print_str("divisible by 3!\n");
  } else {
    print_str("not divisible by 3!\n");
  }
}
```

## Compiling and running

To compile the compiler, you will need a rust toolchain installed. That can be
downloaded here:
https://www.rust-lang.org/tools/install

At that point, one can simply `cargo build --release` to compile a release build.
A binary will be placed at `target/compiler/compiler[.exe]` which can be run
with a file path to compile:
```
./target/compiler/compiler tests/while.oh
```
the resulting assembly will be printed to `stdout`, while errors will (should?)
print to `stderr`.

### Dependencies

besides a rust toolchain, this compiler uses [logos](https://crates.io/crates/logos/0.11.0-rc2)
to generate a lexer, which is automatically downloaded and compiled.

## features

- expressions
  - operators (precedence similar to C):
  - +, -
  - *, /, %
  - `>>`, `<<`, `>>_` (logical right shift)
  - &, |, ^
  - <, >, <u (less unsigned)
  - ==
  - unary -, ~, !
- variables (`let`, `set`)
- calls to standard functions for RARS ecalls
- C-style strings
- `if`(/`else`) and `while`

### Standard functions

these all wrap RARS ecalls and can be found in `resources/stdlib.s`.

## known issues

- registers are not saved before function calls
- the entire standard library is always included, even if it isnt all used
- variables are only static, not on the stack
- extremely poor performance - this compiler attempts to generate correct code,
but performs no optimizations. this means that the end result tends to be somewhat
redundant and overly explicit.
- there are likely bugs.
- poor code quality due to rushed development

## implementation

- `expr.rs` - abstract syntax tree definitions
- `parse.rs` - lexer definition and recursive descent parser
- `riscv.rs` - compiler implementation
  - noteworthy: `compile_stmt` and `compile_expr`
- `main.rs` - command line interface

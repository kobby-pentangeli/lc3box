# lc3vm

Virtual machine for the [LC3 Box](../README.md) toolbox: a fetch–decode–execute interpreter that runs pre-assembled [_Little Computer 3_ (LC-3)](https://en.wikipedia.org/wiki/Little_Computer_3) object files, built on the shared [`lc3core`](../lc3core) instruction-set kernel.

The VM models the full 16-bit address space (65,536 words), the eight general-purpose registers, the program counter, and the negative/zero/positive condition codes. It implements all sixteen LC-3 opcodes and the standard trap routines for console and keyboard I/O, with memory-mapped keyboard input through the keyboard status and data registers.

## Object files

The VM runs `.obj` files: a stream of 16-bit big-endian words whose first word is the _origin_---the address at which the program is loaded---followed by the program image. Execution begins at the origin. Such files are produced by any LC-3 assembler; [examples/bootstrap.asm](../examples/bootstrap.asm) is a sample listing you can assemble.

## Traps

The six standard service routines are emulated by the host:

| Vector | Name    | Effect                                                          |
| ------ | ------- | --------------------------------------------------------------- |
| `x20`  | `GETC`  | Read one character from the keyboard into R0 (without echo)     |
| `x21`  | `OUT`   | Write the character in R0 to the console                        |
| `x22`  | `PUTS`  | Write the NUL-terminated string beginning at the address in R0  |
| `x23`  | `IN`    | Prompt, read one character into R0, and echo it                 |
| `x24`  | `PUTSP` | Write a packed string (two characters per word) addressed by R0 |
| `x25`  | `HALT`  | Stop execution                                                  |

An unrecognized trap vector is reported as an error rather than executed.

## Usage

Run any of the example programs in the repository's [examples](../examples) folder:

```sh
cargo run -p lc3vm -- examples/<program_name>.obj
```

`2048` and `rogue` are interactive; `hello-world` prints a greeting and halts. Interactive programs place the terminal in raw mode for the duration of the run and restore it on exit, including on error.

To install the VM as an `lc3vm` binary on your `PATH`:

```sh
cargo install --path .
```

## License

Licensed under either of [Apache License, Version 2.0](../LICENSE-APACHE) or [MIT license](../LICENSE-MIT) at your option.

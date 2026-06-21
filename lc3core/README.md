# lc3core

Shared instruction-set kernel for the [LC3 Box](../README.md) toolbox: the common data structures and definitions for the [_Little Computer 3_ (LC-3)](https://en.wikipedia.org/wiki/Little_Computer_3) instruction-set architecture, against which the virtual machine, the assembler, and the disassembler---and the forthcoming compiler---are all built.

`lc3core` defines:

- **Opcodes** — the sixteen LC-3 operations, their 4-bit encoding, and case-insensitive mnemonic parsing and rendering.
- **Registers** — the eight general-purpose registers, register-token parsing and `R0`–`R7` rendering, and the negative/zero/positive condition-code model.
- **Traps** — the standard service-routine vectors and their named aliases (`GETC`, `OUT`, `PUTS`, `IN`, `PUTSP`, `HALT`), parsed and rendered.
- **Memory map** — the address-space constants: the 2^16-word memory, the user-program origin, and the memory-mapped device registers.
- **Object format** — the big-endian `.obj` representation, with encoding and decoding.
- **Pseudo-ops** — the `.ORIG`/`.FILL`/`.BLKW`/`.STRINGZ`/`.END` assembler directive set.
- **Field helpers** — branch-condition parsing and `BR`-suffix rendering, and the signed/unsigned instruction-field range checks that the assembler encodes with and the virtual machine decodes against.

This shared encode-and-decode vocabulary---each operation, register, condition, and trap mapped both ways between bits and text---is why the assembler, the virtual machine, and the disassembler agree on every bit of the instruction set.

The crate is dependency-free and has no runtime of its own; to execute LC-3 programs, see the [`lc3vm`](../lc3vm) crate.

## License

Licensed under either of [Apache License, Version 2.0](../LICENSE-APACHE) or [MIT license](../LICENSE-MIT) at your option.

# lc3core

Shared instruction-set kernel for the [LC3 Box](../README.md) toolbox: the common data structures and definitions for the [_Little Computer 3_ (LC-3)](https://en.wikipedia.org/wiki/Little_Computer_3) instruction-set architecture, against which the virtual machine---and the forthcoming assembler, disassembler, and compiler---are all built.

`lc3core` defines:

- **Opcodes** — the sixteen LC-3 operations and their 4-bit decoding.
- **Registers** — the eight general-purpose registers and the negative/zero/positive condition-code model.
- **Traps** — the standard service-routine vectors (`GETC`, `OUT`, `PUTS`, `IN`, `PUTSP`, `HALT`).
- **Memory map** — the address-space constants: the 2^16-word memory, the user-program origin, and the memory-mapped device registers.
- **Object format** — the big-endian `.obj` representation, with encoding and decoding.
- **Pseudo-ops** — the `.ORIG`/`.FILL`/`.BLKW`/`.STRINGZ`/`.END` assembler directive set.

The crate is dependency-free and has no runtime of its own; to execute LC-3 programs, see the [`lc3vm`](../lc3vm) crate.

## License

Licensed under either of [Apache License, Version 2.0](../LICENSE-APACHE) or [MIT license](../LICENSE-MIT) at your option.

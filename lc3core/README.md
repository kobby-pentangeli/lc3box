# lc3core

Shared instruction-set kernel for the [LC3 Box](../README.md) toolbox: common data structures and algorithms for the [_Little Computer 3_ (LC-3)](https://en.wikipedia.org/wiki/Little_Computer_3) instruction-set architecture.

This crate defines the opcode set and instruction encoding, the register and condition-code model, the trap vectors, the memory-map constants, the `.obj` object-file format, and the assembly pseudo-ops. The virtual machine, assembler, disassembler, and compiler all build on these definitions.

`lc3core` is a library with no runtime of its own; to execute LC-3 programs, see the [`lc3vm`](../lc3vm) crate.

## License

Licensed under either of [Apache License, Version 2.0](../LICENSE-APACHE) or [MIT license](../LICENSE-MIT) at your option.

# lc3as

Assembler for the [LC3 Box](../README.md) toolbox: translates [_Little Computer 3_ (LC-3)](https://en.wikipedia.org/wiki/Little_Computer_3) assembly source into the big-endian `.obj` object files that [`lc3vm`](../lc3vm) executes.

`lc3as` is a two-pass assembler. The first pass walks the source to lay out the location counter and build the symbol table; the second resolves label references to PC-relative offsets and encodes each statement into LC-3 machine words. Instruction encoding goes through the shared [`lc3core`](../lc3core) kernel, so the assembler and the virtual machine agree on every bit of the instruction set.

## License

Licensed under either of [Apache License, Version 2.0](../LICENSE-APACHE) or [MIT license](../LICENSE-MIT) at your option.

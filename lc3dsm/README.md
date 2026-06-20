# lc3dsm

Disassembler for the [LC3 Box](../README.md) toolbox: the inverse of [`lc3as`](../lc3as), decoding the big-endian `.obj` object files that [`lc3vm`](../lc3vm) executes back into readable, re-assemblable [_Little Computer 3_ (LC-3)](https://en.wikipedia.org/wiki/Little_Computer_3) assembly.

`lc3dsm` sweeps an object image word by word, decodes each word into its instruction form, recovers labels from PC-relative references, and renders a re-assemblable assembly listing in which each line carries its address and hex encoding as a trailing comment. Decoding goes through the shared [`lc3core`](../lc3core) kernel, so the disassembler and the assembler agree on every bit of the instruction set, and reassembling a disassembled object reproduces the original image.

## License

Licensed under either of [Apache License, Version 2.0](../LICENSE-APACHE) or [MIT license](../LICENSE-MIT) at your option.

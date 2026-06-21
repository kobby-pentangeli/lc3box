# lc3dsm

Disassembler for the [LC3 Box](../README.md) toolbox: the inverse of [`lc3as`](../lc3as), decoding the big-endian `.obj` object files that [`lc3vm`](../lc3vm) executes back into readable, re-assemblable [_Little Computer 3_ (LC-3)](https://en.wikipedia.org/wiki/Little_Computer_3) assembly.

`lc3dsm` sweeps an object image word by word, decodes each word into its instruction form, recovers labels from PC-relative references, and renders a re-assemblable assembly listing. Decoding goes through the shared [`lc3core`](../lc3core) kernel, so the disassembler and the assembler agree on every bit of the instruction set.

## Listing format

The output is a single artifact that is both a human-readable listing and valid LC-3 assembly. Each word becomes one line---a mnemonic with its operands, or `.FILL xNNNN` for a word that is not a canonical instruction---framed by `.ORIG`/`.END`, with the line's address and hex encoding carried as a trailing `;` comment. Because a comment is whitespace to the assembler, the listing re-assembles unchanged.

Decoding is total and faithful. Every 16-bit word decodes either to a canonically re-assemblable instruction or to a `.FILL` data word: a reserved opcode, a branch with an empty condition field, or any instruction format carrying non-zero reserved bits is rendered as data. Numbers print in the assembler's own forms (`x`-hex, `#`-decimal), registers as `R0`–`R7`, the standard trap vectors as their aliases (`GETC`, `OUT`, `PUTS`, `IN`, `PUTSP`, `HALT`), and `JMP R7`/`JSRR` as `RET`/`JSRR`. A PC-relative reference whose target lies within the image is given a synthesized label, while one that points outside keeps its numeric offset.

## Round-trip guarantee

Paired with `lc3as`, `lc3dsm` closes the assemble/disassemble round-trip: re-assembling a disassembled object reproduces the original image, `assemble(disassemble(obj)) == obj`, byte for byte. This holds for every word---even a data word that happens to decode as an instruction re-encodes to the same bits---which is what lets the listing carry the raw encoding without ever losing information.

## Usage

Disassemble an object file; the listing is printed to standard output:

```sh
cargo run -p lc3dsm -- program.obj
```

Write it to a file with `-o`/`--output`:

```sh
cargo run -p lc3dsm -- program.obj -o program.asm
```

A malformed object file is reported with a clear message and a non-zero exit status. To install `lc3dsm` on your `PATH`:

```sh
cargo install --path .
```

## Example

Disassembling the bundled [`examples/hello-world.obj`](../examples/hello-world.obj):

```sh
cargo run -p lc3dsm -- examples/hello-world.obj
```

```asm
.ORIG x3000
        LEA R0, L_3003          ; x3000 xE002
        PUTS                    ; x3001 xF022
        HALT                    ; x3002 xF025
L_3003  .FILL x0048             ; x3003 x0048
        .FILL x0065             ; x3004 x0065
        .FILL x006C             ; x3005 x006C
        .FILL x006C             ; x3006 x006C
        .FILL x006F             ; x3007 x006F
        .FILL x0020             ; x3008 x0020
        .FILL x0057             ; x3009 x0057
        .FILL x006F             ; x300A x006F
        .FILL x0072             ; x300B x0072
        .FILL x006C             ; x300C x006C
        .FILL x0064             ; x300D x0064
        .FILL x0021             ; x300E x0021
        .FILL x0000             ; x300F x0000
.END
```

The recovered `L_3003` is the address `LEA` loads---the start of the `"Hello World!"` string, which the disassembler renders as its `.FILL` character words. Feeding this listing back to `lc3as` reproduces `hello-world.obj` exactly.

## License

Licensed under either of [Apache License, Version 2.0](../LICENSE-APACHE) or [MIT license](../LICENSE-MIT) at your option.

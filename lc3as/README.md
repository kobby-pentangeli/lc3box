# lc3as

Assembler for the [LC3 Box](../README.md) toolbox: translates [_Little Computer 3_ (LC-3)](https://en.wikipedia.org/wiki/Little_Computer_3) assembly source into the big-endian `.obj` object files that [`lc3vm`](../lc3vm) executes.

`lc3as` is a two-pass assembler. The first pass walks the source to lay out the location counter and build the symbol table; the second resolves label references to PC-relative offsets and encodes each statement into LC-3 machine words. Instruction encoding goes through the shared [`lc3core`](../lc3core) kernel, so the assembler and the virtual machine agree on every bit of the instruction set.

## Syntax

Source is line-oriented: each line is an optional label, then an instruction or directive, then an optional `;` comment. Mnemonics and register names are case-insensitive.

- **Instructions** — `ADD`, `AND`, `NOT`; the loads and stores `LD`, `LDI`, `LDR`, `LEA`, `ST`, `STI`, `STR`; the branches `BR`, `BRn`, `BRz`, `BRp`, `BRnz`, `BRnp`, `BRzp`, `BRnzp`; the jumps `JMP`, `RET`, `JSR`, `JSRR`; and `TRAP` and `RTI`.
- **Trap aliases** — `GETC`, `OUT`, `PUTS`, `IN`, `PUTSP`, `HALT`, the named forms of the standard service routines.
- **Registers** — `R0` through `R7`.
- **Numbers** — decimal with a `#` prefix (`#10`, `#-1`) and hexadecimal with an `x` prefix (`x3000`, `xFFFF`).
- **Labels** — a name marks an address and can be the target of a branch, jump, load, store, or `.FILL`.

### Directives

| Directive    | Effect                                                 |
| ------------ | ------------------------------------------------------ |
| `.ORIG a`    | Begin a segment loaded at address `a`                  |
| `.FILL v`    | Emit one word holding the value (or label address) `v` |
| `.BLKW n`    | Reserve `n` zero-filled words                          |
| `.STRINGZ s` | Emit `s` as a NUL-terminated string, one word per char |
| `.END`       | End the current segment                                |

A source file may contain several `.ORIG`/`.END` segments; each is assembled into its own object file.

## Usage

Assemble a source file; the object is written next to it with a `.obj` extension:

```sh
cargo run -p lc3as -- program.asm
```

Choose the output path with `-o`/`--output`:

```sh
cargo run -p lc3as -- program.asm -o build/program.obj
```

A program split across several segments is written as one object file per segment, each suffixed with its origin (for example `program-3000.obj`), and every written path is printed. Malformed source is reported with the offending line number and a non-zero exit status. Run the result on the virtual machine:

```sh
cargo run -p lc3vm -- program.obj
```

To install `lc3as` on your `PATH`:

```sh
cargo install --path .
```

## Example

[`examples/hello-world.asm`](../examples/hello-world.asm):

```asm
; Prints "Hello World!" to the console and halts.
.ORIG x3000
        LEA R0, HELLO
        PUTS
        HALT
HELLO   .STRINGZ "Hello World!"
.END
```

```sh
cargo run -p lc3as -- examples/hello-world.asm -o hello-world.obj
cargo run -p lc3vm -- hello-world.obj
```

## License

Licensed under either of [Apache License, Version 2.0](../LICENSE-APACHE) or [MIT license](../LICENSE-MIT) at your option.

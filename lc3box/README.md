# lc3box

The unified command-line driver for the [LC3 Box](../README.md) toolbox: a single frontend over the [_Little Computer 3_ (LC-3)](https://en.wikipedia.org/wiki/Little_Computer_3) tool libraries, with one subcommand per tool.

Each subcommand delegates to a library core built on the shared [`lc3core`](../lc3core) kernel: `run` executes through [`lc3vm`](../lc3vm), `asm` assembles through [`lc3as`](../lc3as), and `disasm` disassembles through [`lc3dsm`](../lc3dsm). The tools agree on every bit of the instruction set because they share that one kernel.

## Subcommands

| Command         | Effect                                                                                          |
| --------------- | ----------------------------------------------------------------------------------------------- |
| `run <file>`    | Execute a program: an `.asm` source is assembled in memory, an `.obj` object is loaded directly |
| `asm <file>`    | Assemble an assembly source file into a `.obj` object file                                      |
| `disasm <file>` | Disassemble a `.obj` object file into a re-assemblable listing                                  |

Run `lc3box --help` (or `lc3box <command> --help`) for the full options of each.

## Usage

Run a program directly from source, or from a pre-assembled object file:

```sh
cargo run -p lc3box -- run program.asm
cargo run -p lc3box -- run program.obj
```

Assemble a source file; the object is written next to it with a `.obj` extension, or to the path given with `-o`/`--output`:

```sh
cargo run -p lc3box -- asm program.asm
cargo run -p lc3box -- asm program.asm -o build/program.obj
```

Disassemble an object file; the listing prints to standard output, or to the path given with `-o`/`--output`:

```sh
cargo run -p lc3box -- disasm program.obj
cargo run -p lc3box -- disasm program.obj -o program.asm
```

A malformed input is reported with a clear message and a non-zero exit status. To install `lc3box` on your `PATH`:

```sh
cargo install --path .
```

## License

Licensed under either of [Apache License, Version 2.0](../LICENSE-APACHE) or [MIT license](../LICENSE-MIT) at your option.

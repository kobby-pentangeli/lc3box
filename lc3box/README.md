# lc3box

The unified command-line driver for the [LC3 Box](../README.md) toolbox: a single frontend over the [_Little Computer 3_ (LC-3)](https://en.wikipedia.org/wiki/Little_Computer_3) tool libraries, with one subcommand per tool.

Each subcommand delegates to a library core built on the shared [`lc3core`](../lc3core) kernel: `run` executes through [`lc3vm`](../lc3vm), `asm` assembles through [`lc3as`](../lc3as), `disasm` disassembles through [`lc3dsm`](../lc3dsm), and `dbg` debugs through [`lc3dbg`](../lc3dbg). The tools agree on every bit of the instruction set because they share that one kernel.

## Subcommands

| Command         | Effect                                                                                          |
| --------------- | ----------------------------------------------------------------------------------------------- |
| `run <file>`    | Execute a program: an `.asm` source is assembled in memory, an `.obj` object is loaded directly |
| `asm <file>`    | Assemble an assembly source file into a `.obj` object file                                      |
| `disasm <file>` | Disassemble a `.obj` object file into a re-assemblable listing                                  |
| `dbg <file>`    | Debug a program interactively: single-step, breakpoints, register and memory inspection         |

Run `lc3box --help` (or `lc3box <command> --help`) for the full options of each. A malformed input is reported with a clear message and a non-zero exit status, never a panic.

## Usage

### Run

Execute a program directly from source---assembled in memory---or from a pre-assembled object file. The form is chosen by the file extension:

```sh
cargo run -p lc3box -- run program.asm
cargo run -p lc3box -- run program.obj
```

This is the tight edit-run loop: write assembly, `run` it, repeat---no intermediate object file to manage. Interactive programs place the terminal in raw mode for the duration of the run and restore it on exit, including on error.

### Assemble

Translate a source file into an object file. With no `-o`/`--output`, the object is written next to the source with a `.obj` extension; a program split across several `.ORIG`/`.END` segments is written as one object file per segment, each suffixed with its origin:

```sh
cargo run -p lc3box -- asm program.asm
cargo run -p lc3box -- asm program.asm -o build/program.obj
```

### Disassemble

Decode an object file into a re-assemblable annotated listing, printed to standard output or written with `-o`/`--output`. Each line carries its address and hex encoding as a trailing `;` comment, labels are recovered from PC-relative references, and any word that is not a canonical instruction is rendered as `.FILL`:

```sh
cargo run -p lc3box -- disasm program.obj
cargo run -p lc3box -- disasm program.obj -o program.asm
```

Paired with `asm`, `disasm` closes the loop---re-assembling a disassembled object reproduces the original image byte for byte:

```sh
cargo run -p lc3box -- disasm examples/hello-world.obj -o hello-world.asm
cargo run -p lc3box -- asm hello-world.asm -o hello-world.obj
```

### Debug

Open an interactive debugging session on a program---from source or a pre-assembled object, chosen by the file extension---then drive it from a prompt:

```sh
cargo run -p lc3box -- dbg program.asm
cargo run -p lc3box -- dbg examples/2048.obj
```

Single-step with `step [n]`, run to a breakpoint or `HALT` with `continue`, set and clear breakpoints with `break`/`delete`, inspect and edit the machine with `registers`/`set`/`write`, and disassemble around the program counter with `disassemble`. `help` lists every command and `quit` leaves the session. An executing program drives the terminal directly---raw mode for the span of the run---while the prompt stays line-edited.

## Example

Run the bundled [`examples/hello-world.asm`](../examples/hello-world.asm) straight from source:

```sh
cargo run -p lc3box -- run examples/hello-world.asm
```

```text
Hello World!
```

## Library

`lc3box` is also an umbrella library that re-exports the tool crates under short module names, so one dependency reaches the whole toolbox: `lc3box::kernel` ([`lc3core`](../lc3core)), `lc3box::vm` ([`lc3vm`](../lc3vm)), `lc3box::asm` ([`lc3as`](../lc3as)), `lc3box::dsm` ([`lc3dsm`](../lc3dsm)), and `lc3box::dbg` ([`lc3dbg`](../lc3dbg)).

```rust
use lc3box::{asm, dsm, vm};

let image = asm::assemble(".ORIG x3000\nHALT\n.END\n")?;
let listing = dsm::disassemble(&image.blocks[0]);
let mut machine = vm::Lc3VM::new();
machine.load_program(&image.blocks)?;
```

Each module sits behind a like-named feature; `full` enables all five, and the default `cli` feature additionally builds the binary. Depend on `lc3box` with `default-features = false` and a single feature to pull in one tool alone, or on the individual tool crates directly for the most granular build:

```toml
lc3box = { version = "0.7", default-features = false, features = ["asm"] }
```

## Install

Install the `lc3box` command-line tool on your `PATH`---from crates.io, straight from the repository, or from a local checkout:

```sh
cargo install lc3box
cargo install --git https://github.com/kobby-pentangeli/lc3box
cargo install --path .
```

Then `lc3box run`, `lc3box asm`, `lc3box disasm`, and `lc3box dbg` are available directly.

## License

Licensed under either of [Apache License, Version 2.0](../LICENSE-APACHE) or [MIT license](../LICENSE-MIT) at your option.

# LC3 Box

[![CI](https://github.com/kobby-pentangeli/lc3box/workflows/CI/badge.svg)](https://github.com/kobby-pentangeli/lc3box/actions)
[![Release](https://img.shields.io/github/v/release/kobby-pentangeli/lc3box?sort=semver)](https://github.com/kobby-pentangeli/lc3box/releases)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

Pure-Rust toolbox for the [_Little Computer 3_ (LC-3)](https://en.wikipedia.org/wiki/Little_Computer_3) assembly language and instruction-set architecture. The goal is a complete LC-3 toolchain---assembler, disassembler, compiler, and virtual machine---sharing one instruction-set kernel.

Today the workspace ships that shared kernel, an assembler that turns LC-3 source into object files, a virtual machine that runs them, and a disassembler that turns them back into readable assembly; a compiler is planned.

## Status

| Component              | Crate     | Status    |
| ---------------------- | --------- | --------- |
| Instruction-set kernel | `lc3core` | Available |
| Virtual machine        | `lc3vm`   | Available |
| Assembler              | `lc3as`   | Available |
| Disassembler           | `lc3dsm`  | Available |
| Compiler               | `lc3c`    | Planned   |

## Project Structure

```text
lc3box/
├── lc3core/    # Shared instruction-set kernel: opcodes, registers, traps, memory map, .obj format
├── lc3as/      # Assembler: a two-pass translator from .asm source to .obj object files
├── lc3dsm/     # Disassembler: decodes .obj object files back into annotated, re-assemblable .asm
├── lc3vm/      # Virtual machine: a fetch–decode–execute interpreter for .obj programs
└── examples/   # LC-3 programs: .asm source and pre-assembled .obj
```

## Architecture

![LC-3 Box architecture diagram](assets/lc3box-arch.png)

Every tool builds on `lc3core`, the single source of truth for the LC-3 instruction set: the opcode set, the register and condition-code model, the trap vectors, the memory-map constants, and the big-endian `.obj` object-file format. `lc3as` encodes assembly source into that object format, `lc3vm` loads an object file into a full 16-bit address space and runs it through the classic fetch–decode–execute loop pictured above, and `lc3dsm` decodes an object file back into a re-assemblable listing---all going through `lc3core`, so every bit the assembler writes is the bit the VM decodes and the disassembler recovers.

## Usage

### Run a program

The [examples](examples) folder contains pre-assembled LC-3 programs (`.obj`). Run any of them with:

```sh
cargo run -p lc3vm -- examples/<program_name>.obj
```

For instance, `cargo run -p lc3vm -- examples/2048.obj` plays a terminal build of 2048; `rogue` and `hello-world` are also included. Interactive programs place the terminal in raw mode for the duration of the run and restore it on exit.

### Assemble and run

Assemble an LC-3 source listing into an object file, then run it on the VM:

```sh
cargo run -p lc3as -- examples/hello-world.asm -o hello-world.obj
cargo run -p lc3vm -- hello-world.obj
```

When `-o` is omitted, `lc3as` writes the object next to the source with a `.obj` extension. A program split across several `.ORIG`/`.END` segments---like [examples/bootstrap.asm](examples/bootstrap.asm)---is assembled into one object file per segment.

### Disassemble

Turn an object file back into a readable, re-assemblable listing, printed to standard output:

```sh
cargo run -p lc3dsm -- examples/2048.obj
```

Each line shows its address and hex encoding as a trailing comment, labels are recovered from PC-relative references, and any word that is not a canonical instruction is rendered as `.FILL`. Use `-o`/`--output` to write the listing to a file. Paired with `lc3as`, the disassembler closes the round-trip---re-assembling a disassembled object reproduces the original image:

```sh
cargo run -p lc3dsm -- examples/hello-world.obj -o hello-world.asm
cargo run -p lc3as -- hello-world.asm -o hello-world.obj
```

To install the tools as `lc3as`, `lc3dsm`, and `lc3vm` binaries on your `PATH`:

```sh
cargo install --path lc3as
cargo install --path lc3dsm
cargo install --path lc3vm
```

## Development

The workspace uses the Rust 2024 edition (Rust 1.88 or newer).

```sh
cargo +nightly fmt
cargo clippy --all-features --all-targets --workspace -- -D warnings
cargo build --release --all-features --all-targets
cargo test --all-features --all-targets --workspace
cargo doc --all-features --no-deps --workspace
```

## Contributing

Contributions are welcome! Please read our [Contributing Guidelines](CONTRIBUTING.md) and [Code of Conduct](CODE_OF_CONDUCT.md).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this codebase by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

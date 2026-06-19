# LC3 Box

[![CI](https://github.com/kobby-pentangeli/lc3box/workflows/CI/badge.svg)](https://github.com/kobby-pentangeli/lc3box/actions)
[![Release](https://img.shields.io/github/v/release/kobby-pentangeli/lc3box?sort=semver)](https://github.com/kobby-pentangeli/lc3box/releases)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

Pure-Rust toolbox for the [_Little Computer 3_ (LC-3)](https://en.wikipedia.org/wiki/Little_Computer_3) assembly language and instruction-set architecture. The goal is a complete LC-3 toolchain---assembler, disassembler, compiler, and virtual machine---sharing one instruction-set kernel.

Today the workspace ships that shared kernel and a virtual machine that runs pre-assembled LC-3 object files; the assembler, disassembler, and compiler are in progress.

## Status

| Component              | Crate     | Status    |
| ---------------------- | --------- | --------- |
| Instruction-set kernel | `lc3core` | Available |
| Virtual machine        | `lc3vm`   | Available |
| Assembler              | `lc3as`   | Planned   |
| Disassembler           | `lc3dsm`  | Planned   |
| Compiler               | `lc3c`    | Planned   |

## Project Structure

```text
lc3box/
├── lc3core/    # Shared instruction-set kernel: opcodes, registers, traps, memory map, .obj format
├── lc3vm/      # Virtual machine: a fetch–decode–execute interpreter for .obj programs
└── examples/   # Pre-assembled LC-3 programs (.obj) and a sample assembly listing
```

## Architecture

![LC-3 Box architecture diagram](assets/lc3box-arch.png)

Every tool builds on `lc3core`, the single source of truth for the LC-3 instruction set: the opcode set, the register and condition-code model, the trap vectors, the memory-map constants, and the big-endian `.obj` object-file format. `lc3vm` loads an object file into a full 16-bit address space and runs it through the classic fetch–decode–execute loop pictured above.

## Usage

The [examples](examples) folder contains pre-assembled LC-3 programs (`.obj`). Run any of them with:

```sh
cargo run -p lc3vm -- examples/<program_name>.obj
```

For instance, `cargo run -p lc3vm -- examples/2048.obj` plays a terminal build of 2048; `rogue` and `hello-world` are also included. Interactive programs place the terminal in raw mode for the duration of the run and restore it on exit.

[examples/bootstrap.asm](examples/bootstrap.asm) is a sample assembly listing; assemble it with any LC-3 assembler---including a web-based one---to produce an `.obj` this VM can run.

To install the VM as an `lc3vm` binary on your `PATH`:

```sh
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

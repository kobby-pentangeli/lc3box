# LC3 Box

[![CI](https://github.com/kobby-pentangeli/lc3box/workflows/CI/badge.svg)](https://github.com/kobby-pentangeli/lc3box/actions)
[![Release](https://img.shields.io/github/v/release/kobby-pentangeli/lc3box?sort=semver)](https://github.com/kobby-pentangeli/lc3box/releases)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

Pure Rust implementation of a compiler (`lc3c`), disassembler (`lc3dsm`), assembler (`lc3as`), and virtual machine (`lc3vm`) for the [_Little Computer 3_ (LC-3)](https://en.wikipedia.org/wiki/Little_Computer_3) assembly language and ISA.

## Architecture

![LC-3 Box architecture diagram](assets/lc3box-arch.png)

## Usage

The [examples](examples) folder contains some LC-3 compatible assembled programs (`.obj`) for the LC-3 ISA.

To run any of them, execute

```sh
cargo run -p lc3vm -- examples/<program_name>.obj
```

There's also a `bootstrap.asm` in the [examples](examples/bootstrap.asm). You can copy-paste the code into any Web-based and/or GUI LC-3 assembler to get a `.obj` version that can then be executed by this VM.

## Contributing

Contributions are welcome! Please read our [Contributing Guidelines](CONTRIBUTING.md) and [Code of Conduct](CODE_OF_CONDUCT.md).

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this codebase by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

# LC3 Box

An implementation of a virtual machine for the [_Little Computer 3_ (LC-3)](https://en.wikipedia.org/wiki/Little_Computer_3) language and ISA.

## Architecture

![LC-3 Box architecture diagram](./assets/lc3box-arch.png)

## Usage

The [examples](./examples) folder contains some LC-3 compatible assembled programs (`.obj`) for the LC-3 ISA.

To run any of them, execute

```sh
cargo run --bin emulator examples/<program_name>.obj
```

There's also a `bootstrap.asm` in the [examples](./examples/bootstrap.asm). You can copy-paste the code into any Web-based and/or GUI LC-3 assembler to get a `.obj` version that can then be executed by this VM.

## Contributing

Contributions are welcome! Please read our [Contributing Guidelines](CONTRIBUTING.md) and [Code of Conduct](CODE_OF_CONDUCT.md).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

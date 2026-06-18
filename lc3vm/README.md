# lc3vm

Virtual machine for the [LC3 Box](../README.md) toolbox: a fetch–decode–execute interpreter that runs pre-assembled [_Little Computer 3_ (LC-3)](https://en.wikipedia.org/wiki/Little_Computer_3) object files, built on the shared [`lc3core`](../lc3core) instruction-set kernel.

The VM models the full 16-bit address space, the eight general-purpose registers and condition codes, and the standard trap routines for console and keyboard I/O.

## Usage

Run any of the example programs in the repository's [examples](../examples) folder:

```sh
cargo run -p lc3vm -- examples/<program_name>.obj
```

## License

Licensed under either of [Apache License, Version 2.0](../LICENSE-APACHE) or [MIT license](../LICENSE-MIT) at your option.

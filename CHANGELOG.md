# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2026-06-21

Adds `lc3dsm`, the LC-3 disassembler---the assembler's inverse---closing the assemble/disassemble loop: any object file can now be turned back into readable, re-assemblable assembly, so assembled programs are no longer opaque.

### Added

- **`lc3dsm`**, a new disassembler that decodes a big-endian `.obj` object file back into a readable, re-assemblable LC-3 assembly listing. It recovers labels from PC-relative references, renders the named trap aliases and the `BR` condition variants, prints registers and numbers in source form, and emits any word that is not a canonical instruction as a `.FILL`. Each line carries its address and hex encoding as a trailing `;` comment, so the same artifact is both a human-readable listing and valid assembly that re-assembles to the original image.
- An `lc3dsm` command-line tool: `lc3dsm program.obj` prints the listing to standard output, with `-o`/`--output` to write it to a file. A malformed object file is reported with a clear message and a non-zero exit status, never a panic.
- Decode-and-render support in the shared `lc3core` kernel---opcode-to-mnemonic, register-number-to-name, condition-field-to-`BR`-suffix, and trap-vector-to-alias---the inverse of the encoding vocabulary added in `0.3.0`, shared with the disassembler.

### Changed

- Assembled programs are no longer opaque: any `.obj`---including the bundled `2048`, `rogue`, and `hello-world`---can be disassembled back to annotated, re-assemblable source, and `assemble(disassemble(obj))` reproduces the original object byte for byte.

## [0.3.0] - 2026-06-20

Adds `lc3as`, the LC-3 assembler, making the toolbox self-contained: write assembly, assemble it to an object file, and run it on `lc3vm`---no external assembler required.

### Added

- **`lc3as`**, a new two-pass assembler that translates LC-3 assembly source (`.asm`) into the big-endian `.obj` object files `lc3vm` runs. It supports the full assembly language: labels; every instruction, including the `BR` condition variants, `RET`/`JSR`/`JSRR`, `NOT`, and `RTI`; the named trap aliases (`GETC`, `OUT`, `PUTS`, `IN`, `PUTSP`, `HALT`); the `.ORIG`, `.FILL`, `.BLKW`, `.STRINGZ`, and `.END` directives; decimal (`#`) and hexadecimal (`x`) numbers; case-insensitive mnemonics; and `;` comments. A program written as several `.ORIG`/`.END` segments is assembled to one object file per segment.
- An `lc3as` command-line tool: `lc3as program.asm` assembles a source file to `program.obj`, with `-o`/`--output` to choose the destination. Malformed source is reported with the offending line number and a non-zero exit status, never a panic.
- A `hello-world.asm` example that assembles to the bundled `hello-world.obj`.
- Encoding support in the shared `lc3core` kernel---opcode values and mnemonic, register, branch-condition, and trap-alias parsing, plus instruction-field range checks---so the assembler and the virtual machine agree on every bit of the instruction set.

### Changed

- The toolbox no longer relies on an external (web) assembler: the bundled `examples/bootstrap.asm` listing---and your own programs---can now be assembled in-tree with `lc3as` and run on `lc3vm`.

## [0.2.0] - 2026-06-19

A re-implementation of the VM. The LC-3 instruction set is extracted into a reusable kernel crate, and the virtual machine is hardened from a tutorial prototype into a panic-free, production-quality interpreter---while still running the same pre-assembled `.obj` programs.

### Added

- **`lc3core`**, a new shared library crate that defines the LC-3 instruction set: the opcode set, the register and condition-code model, the trap vectors, the memory-map constants, the big-endian `.obj` object-file format, and the assembler pseudo-ops. The virtual machine---and the forthcoming assembler, disassembler, and compiler---all build on it.
- The virtual machine now addresses the entire 16-bit memory, including the final word at `xFFFF`, so programs that read or write the top of memory run correctly.

### Changed

- The VM is now the `lc3vm` crate within a Cargo workspace (the binary was renamed from `emulator`); run a program with `cargo run -p lc3vm -- <file.obj>`, or install it with `cargo install --path lc3vm` and run `lc3vm <file.obj>`.
- Failures are reported gracefully instead of panicking: a malformed object file, a program that does not fit in memory, or an I/O error now prints a clear message and exits non-zero, and the terminal is reliably returned to its normal mode on exit---including on error.
- The minimum supported Rust version (MSRV) is now 1.88.

### Fixed

- `JSRR` whose base register is R7 now jumps to the correct address; the base was previously overwritten before being read.
- Indirect loads and stores and other address calculations near the top of memory no longer overflow; address arithmetic now wraps as the architecture specifies.
- The program counter now wraps correctly at the end of the address space instead of overflowing past it.

## [0.1.0] - 2026-06-18

Initial release. A pure-Rust virtual machine that runs pre-assembled Little Computer 3 (LC-3) programs, faithful to the LC-3 instruction set architecture.

### Added

- **LC-3 virtual machine** (`lc3vm`) that loads and executes a pre-assembled object file (`.obj`), interpreting it against a full 16-bit address space.
- Support for the complete LC-3 instruction set: arithmetic and logic (`ADD`, `AND`, `NOT`), data movement (`LD`, `LDI`, `LDR`, `LEA`, `ST`, `STI`, `STR`), and control flow (`BR`, `JMP`/`RET`, `JSR`/`JSRR`).
- The six standard trap routines for console and keyboard I/O: `GETC`, `OUT`, `PUTS`, `IN`, `PUTSP`, and `HALT`.
- Condition codes (negative, zero, positive) updated after each result-producing instruction, driving conditional branches.
- Memory-mapped keyboard input through the keyboard status and data registers (`KBSR`/`KBDR`).
- A command-line interface that takes the path to an object file and runs it, with the terminal placed in raw mode for interactive programs and restored on exit or panic.
- Example programs to run out of the box (`2048`, `rogue`, `hello-world`), plus a `bootstrap.asm` source listing.
- Dual licensing under MIT or Apache-2.0.
- Continuous integration covering build, test, formatting, linting, documentation, code coverage, and a scheduled dependency security audit.
- Contributor documentation: contributing guidelines, code of conduct, issue and pull-request templates, and code ownership.

---

## Guidelines for Contributors

When adding entries to this changelog for future releases:

1. **Format**: Follow [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
2. **Categories**: Use Added, Changed, Deprecated, Removed, Fixed, Security
3. **Audience**: Write for users, not developers (focus on impact, not implementation)
4. **Links**: Add comparison links at the bottom: `[0.5.0]: https://github.com/kobby-pentangeli/lc3box/compare/v0.4.0...v0.5.0`

[0.4.0]: https://github.com/kobby-pentangeli/lc3box/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/kobby-pentangeli/lc3box/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/kobby-pentangeli/lc3box/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/kobby-pentangeli/lc3box/releases/tag/v0.1.0

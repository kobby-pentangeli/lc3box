# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
4. **Links**: Add comparison links at the bottom: `[0.2.0]: https://github.com/kobby-pentangeli/lc3box/compare/v0.1.0...v0.2.0`

[0.1.0]: https://github.com/kobby-pentangeli/lc3box/releases/tag/v0.1.0

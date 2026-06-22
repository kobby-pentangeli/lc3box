# lc3debug

Interactive debugger for the [LC3 Box](../README.md) toolbox: it drives the [`lc3vm`](../lc3vm) virtual machine one instruction at a time so you can watch a [_Little Computer 3_ (LC-3)](https://en.wikipedia.org/wiki/Little_Computer_3) program as it runs---single-step it, stop it at breakpoints, inspect and edit registers and memory, and disassemble the code around the program counter through [`lc3dsm`](../lc3dsm).

`lc3debug` is the debugging engine, not a terminal. The `Debugger` is a plain state machine over a loaded program---step, resume, breakpoints, register and memory access, a disassembly window---and `parse` turns one line of input into a `Command`. A frontend reads commands, calls the engine, and prints the result; the engine itself never touches the console, which keeps it exhaustively testable.

## Commands

Command keywords are case-insensitive and carry short aliases. Numeric operands take the assembler's own forms---`x`-hex (`x3000`), `#`-decimal (`#-1`), plus bare decimal and `0x` hex---and registers are `R0`–`R7` or `PC`.

| Command                    | Aliases    | Effect                                                            |
| -------------------------- | ---------- | ----------------------------------------------------------------- |
| `step [n]`                 | `s`, `si`  | Execute `n` instructions (default 1), stopping early at a `HALT`. |
| `continue`                 | `c`        | Run until a breakpoint, a `HALT`, or an error.                    |
| `break <addr>`             | `b`        | Set a breakpoint.                                                 |
| `delete <addr>`            | `d`        | Clear a breakpoint.                                               |
| `breaks`                   |            | List the active breakpoints.                                      |
| `registers`                | `regs`     | Show the register file.                                           |
| `set <reg> <value>`        |            | Write a register.                                                 |
| `write <addr> <value>`     | `w`        | Write a word of memory.                                           |
| `disassemble [addr] [len]` | `dis`, `x` | Disassemble a window (default: the program counter).              |
| `reset`                    |            | Reload the program and return to its entry point.                 |
| `help`                     | `h`, `?`   | Show the command summary.                                         |
| `quit`                     | `q`        | Leave the debugger.                                               |

A breakpoint stops the machine _before_ the instruction at its address runs. `step` is explicit and ignores breakpoints; only `continue` honors them. `reset` clears registers and memory and reloads the original program, but keeps the breakpoints you have set.

## Usage

`lc3debug` is a library crate; the command-line frontend is [`lc3box`](../lc3box), whose `dbg` subcommand opens a debugging session on a program:

```sh
cargo run -p lc3box -- dbg examples/2048.obj
```

It accepts either an assembled `.obj` image or `.asm` source, which `lc3box` assembles in memory before handing the program to the debugger.

To embed the engine in Rust, depend on `lc3debug`, build a `Debugger` from the program's object segments, and dispatch parsed commands:

```rust
use lc3debug::{Command, Debugger, Stop, parse};

let mut debugger = Debugger::new(program)?; // program: Vec<lc3core::ObjectFile>
debugger.add_breakpoint(0x3005);

if let Command::Continue = parse("continue")? {
    match debugger.resume()? {
        Stop::Breakpoint(address) => println!("stopped at x{address:04X}"),
        Stop::Halted => println!("halted"),
        _ => {}
    }
}
```

For a ready-made command loop instead of hand-dispatching, `repl` reads commands from any `BufRead`, drives the engine, and writes results to any `Write`---the same frontend `lc3box dbg` runs over the terminal.

## License

Licensed under either of [Apache License, Version 2.0](../LICENSE-APACHE) or [MIT license](../LICENSE-MIT) at your option.

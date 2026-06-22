//! The read-eval-print loop: a terminal frontend over the [`Debugger`].
//!
//! [`repl`] reads commands from a reader, dispatches each to the engine, and
//! writes the result to a writer. It is generic over its streams, so the binary
//! drives the real terminal while tests drive scripted input and capture output.
//! Only the span of an executing command (`step`, `continue`) takes the terminal
//! into raw mode---there the inferior program reads keys directly---while command
//! entry stays in the terminal's normal line-edited mode.

use std::io::{self, BufRead, Write};

use lc3core::ConditionFlag;
use lc3vm::{RawMode, Registers};

use crate::{Command, Debugger, ParseError, Register, Stop, parse};

/// The prompt written before each command is read.
const PROMPT: &str = "(lc3dbg) ";

/// The command summary shown by `help`.
const HELP: &str = "\
commands (case-insensitive; [] marks an optional operand):
  step [n]             s, si    run n instructions (default 1), stopping at HALT
  continue             c        run until a breakpoint, HALT, or an error
  break <addr>         b        set a breakpoint
  delete <addr>        d        clear a breakpoint
  breaks                        list the active breakpoints
  registers            regs     show the register file
  set <reg> <value>             write a register (R0-R7 or PC)
  write <addr> <value> w        write a word of memory
  disassemble [addr] [len]  dis, x  disassemble a window (default: the PC)
  reset                         reload the program and return to its entry point
  help                 h, ?     show this summary
  quit                 q        leave the debugger
";

/// Whether the loop keeps reading commands or leaves the session.
enum Flow {
    /// Read another command.
    Continue,
    /// Leave the debugger.
    Quit,
}

/// Runs an interactive debugging session over `debugger`, reading commands from
/// `input` and writing their results to `output`.
///
/// Each line is parsed and dispatched to the engine; a blank line is ignored and
/// a parse error is reported, both leaving the session running. The loop ends on
/// a `quit` command or end of input. While an executing command (`step`,
/// `continue`) runs, the terminal is placed in raw mode so an interactive
/// program reads keys directly; entry of the next command returns to the
/// terminal's normal mode. Off a real terminal---a pipe, or a test---raw mode is
/// simply skipped and the command still runs.
pub fn repl<R: BufRead, W: Write>(
    debugger: &mut Debugger,
    mut input: R,
    mut output: W,
) -> io::Result<()> {
    let mut line = String::new();
    loop {
        write!(output, "{PROMPT}")?;
        output.flush()?;

        line.clear();
        if input.read_line(&mut line)? == 0 {
            return Ok(());
        }

        let command = match parse(&line) {
            Ok(command) => command,
            Err(ParseError::Empty) => continue,
            Err(error) => {
                writeln!(output, "{error}")?;
                continue;
            }
        };

        // An executing command hands the terminal to the inferior program for
        // the span of the run; the guard restores normal mode before the next
        // command is read.
        let _terminal = matches!(&command, Command::Step(_) | Command::Continue)
            .then(|| RawMode::enable().ok())
            .flatten();
        if let Flow::Quit = dispatch(debugger, command, &mut output)? {
            return Ok(());
        }
    }
}

/// Dispatches one parsed `command` to the engine, writing its result.
fn dispatch<W: Write>(
    debugger: &mut Debugger,
    command: Command,
    output: &mut W,
) -> io::Result<Flow> {
    match command {
        Command::Step(count) => {
            let outcome = debugger.step(count);
            report_stop(output, outcome, debugger)?;
        }
        Command::Continue => {
            let outcome = debugger.resume();
            report_stop(output, outcome, debugger)?;
        }
        Command::Break(address) => {
            let verb = if debugger.add_breakpoint(address) {
                "set"
            } else {
                "already set"
            };
            writeln!(output, "breakpoint {verb} at x{address:04X}")?;
        }
        Command::Delete(address) => {
            let message = if debugger.remove_breakpoint(address) {
                format!("cleared breakpoint at x{address:04X}")
            } else {
                format!("no breakpoint at x{address:04X}")
            };
            writeln!(output, "{message}")?;
        }
        Command::Breakpoints => {
            let listing = debugger
                .breakpoints()
                .map(|address| format!("x{address:04X}"))
                .collect::<Vec<_>>();
            if listing.is_empty() {
                writeln!(output, "no breakpoints")?;
            } else {
                writeln!(output, "{}", listing.join("\n"))?;
            }
        }
        Command::Registers => write_registers(output, debugger.registers())?,
        Command::SetRegister(register, value) => {
            debugger.set_register(register, value);
            writeln!(output, "{} = x{value:04X}", register_name(register))?;
        }
        Command::WriteMemory(address, value) => {
            debugger.write_memory(address, value);
            writeln!(output, "x{address:04X} = x{value:04X}")?;
        }
        Command::Disassemble { address, len } => {
            let address = address.unwrap_or_else(|| debugger.registers().pc);
            write!(output, "{}", debugger.disassemble(address, len))?;
        }
        Command::Reset => match debugger.reset() {
            Ok(()) => writeln!(output, "reset to the entry point")?,
            Err(error) => writeln!(output, "error: {error}")?,
        },
        Command::Help => write!(output, "{HELP}")?,
        Command::Quit => return Ok(Flow::Quit),
    }
    Ok(Flow::Continue)
}

/// Reports the outcome of an executing command, then the location it stopped at.
fn report_stop<W: Write>(
    output: &mut W,
    outcome: Result<Stop, lc3vm::Error>,
    debugger: &Debugger,
) -> io::Result<()> {
    match outcome {
        Ok(Stop::Stepped) => show_location(output, debugger),
        Ok(Stop::Breakpoint(address)) => {
            writeln!(output, "breakpoint at x{address:04X}")?;
            show_location(output, debugger)
        }
        Ok(Stop::Halted) => writeln!(output, "halted"),
        Err(error) => writeln!(output, "error: {error}"),
    }
}

/// Writes the program counter and the instruction it now points at.
fn show_location<W: Write>(output: &mut W, debugger: &Debugger) -> io::Result<()> {
    let pc = debugger.registers().pc;
    writeln!(
        output,
        "x{pc:04X}: {}",
        lc3dsm::render_instruction(debugger.read_memory(pc))
    )
}

/// Writes the eight general-purpose registers, the program counter, and the
/// condition code, four registers to a line.
fn write_registers<W: Write>(output: &mut W, registers: &Registers) -> io::Result<()> {
    for reg in 0u16..8 {
        write!(output, "R{reg} x{:04X}", registers.get(reg))?;
        if reg % 4 == 3 {
            writeln!(output)?;
        } else {
            write!(output, "  ")?;
        }
    }
    writeln!(
        output,
        "PC x{:04X}  COND {}",
        registers.pc,
        condition_code(registers.cond)
    )
}

/// The single-letter name of a condition code: `N`, `Z`, or `P`.
fn condition_code(cond: ConditionFlag) -> char {
    match cond {
        ConditionFlag::Negative => 'N',
        ConditionFlag::Zero => 'Z',
        ConditionFlag::Positive => 'P',
    }
}

/// The display name of a register: `R0`--`R7` or `PC`.
fn register_name(register: Register) -> String {
    match register {
        Register::General(reg) => format!("R{reg}"),
        Register::Pc => String::from("PC"),
    }
}

#[cfg(test)]
mod tests {
    use lc3core::ObjectFile;

    use super::{Flow, dispatch, repl};
    use crate::{Command, Debugger, Register};

    fn debugger(words: &[u16]) -> Debugger {
        Debugger::new(vec![ObjectFile {
            origin: 0x3000,
            words: words.to_vec(),
        }])
        .expect("synthetic program fits in memory")
    }

    fn dispatched(dbg: &mut Debugger, command: Command) -> String {
        let mut out = Vec::new();
        dispatch(dbg, command, &mut out).expect("dispatch writes");
        String::from_utf8(out).expect("output is UTF-8")
    }

    #[test]
    fn stepping_reports_the_next_location_and_then_the_halt() {
        // ADD R0,R0,#1 ; HALT
        let mut dbg = debugger(&[0x1021, 0xF025]);

        let stepped = dispatched(&mut dbg, Command::Step(1));
        assert!(stepped.contains("x3001"), "location: {stepped}");
        assert!(stepped.contains("HALT"), "next instruction: {stepped}");
        assert_eq!(dbg.registers().get(0), 1);

        assert!(dispatched(&mut dbg, Command::Step(1)).contains("halted"));
    }

    #[test]
    fn continue_runs_to_halt_advancing_state() {
        // ADD R0,R0,#1 (x2), HALT
        let mut dbg = debugger(&[0x1021, 0x1021, 0xF025]);
        assert!(dispatched(&mut dbg, Command::Continue).contains("halted"));
        assert_eq!(dbg.registers().get(0), 2);
    }

    #[test]
    fn continue_reports_the_breakpoint_it_stops_on() {
        let mut dbg = debugger(&[0x1021, 0x1021, 0xF025]);
        dbg.add_breakpoint(0x3001);
        let report = dispatched(&mut dbg, Command::Continue);
        assert!(report.contains("breakpoint at x3001"), "{report}");
        assert_eq!(dbg.registers().get(0), 1);
    }

    #[test]
    fn registers_command_shows_edited_values_and_the_condition_code() {
        let mut dbg = debugger(&[0xF025]);
        dispatched(&mut dbg, Command::SetRegister(Register::General(0), 42));
        let dump = dispatched(&mut dbg, Command::Registers);
        assert!(dump.contains("R0 x002A"), "{dump}");
        assert!(dump.contains("PC x3000"), "{dump}");
        assert!(dump.contains("COND Z"), "{dump}");
    }

    #[test]
    fn breakpoint_commands_set_list_in_order_and_clear() {
        let mut dbg = debugger(&[0xF025]);
        dispatched(&mut dbg, Command::Break(0x3005));
        dispatched(&mut dbg, Command::Break(0x3001));

        let listed = dispatched(&mut dbg, Command::Breakpoints);
        assert_eq!(listed, "x3001\nx3005\n");

        dispatched(&mut dbg, Command::Delete(0x3001));
        assert_eq!(dispatched(&mut dbg, Command::Breakpoints), "x3005\n");
    }

    #[test]
    fn memory_edits_are_observable_through_the_disassembly_window() {
        let mut dbg = debugger(&[0xF025]);
        dispatched(&mut dbg, Command::WriteMemory(0x3000, 0x1021));
        let listing = dispatched(
            &mut dbg,
            Command::Disassemble {
                address: Some(0x3000),
                len: 1,
            },
        );
        assert!(listing.contains("ADD R0, R0, #1"), "{listing}");
    }

    #[test]
    fn quit_signals_the_loop_to_leave() {
        let mut dbg = debugger(&[0xF025]);
        let mut out = Vec::new();
        assert!(matches!(
            dispatch(&mut dbg, Command::Quit, &mut out),
            Ok(Flow::Quit)
        ));
    }

    #[test]
    fn loop_reports_parse_errors_and_runs_help_until_quit() {
        let mut dbg = debugger(&[0xF025]);
        let input = b"help\nbogus\nquit\nregisters\n"; // commands past quit go unread
        let mut out = Vec::new();
        repl(&mut dbg, &input[..], &mut out).expect("repl runs to quit");

        let text = String::from_utf8(out).expect("output is UTF-8");
        assert!(text.contains("step [n]"), "help summary: {text}");
        assert!(text.contains("unknown command 'bogus'"), "error: {text}");
        // "COND" appears only in the register dump, so quit stopped the loop
        // before the trailing `registers` command could run.
        assert!(
            !text.contains("COND"),
            "quit must stop before 'registers': {text}"
        );
    }

    #[test]
    fn loop_leaves_at_end_of_input_without_a_quit() {
        let mut dbg = debugger(&[0xF025]);
        let input = b"registers\n"; // no quit: end of input ends the session
        let mut out = Vec::new();
        repl(&mut dbg, &input[..], &mut out).expect("repl runs to end of input");
        assert!(String::from_utf8(out).unwrap().contains("PC x3000"));
    }
}

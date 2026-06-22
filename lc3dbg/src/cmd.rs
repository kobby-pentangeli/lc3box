//! Command parsing: one line of input resolved to a [`Command`].
//!
//! [`parse`] hand-scans a debugger command line---a flat, line-oriented grammar
//! of a keyword and its operands---the same approach `lc3as` takes to assembly,
//! with no parser-generator crate. Keywords are case-insensitive and carry short
//! aliases; numeric operands accept the assembler's own `x`-hex and `#`-decimal
//! forms (plus bare decimal and `0x` hex), and registers are `R0`--`R7` or `PC`.
//! A line that names no command, or whose operands do not fit it, yields a
//! [`ParseError`].

use thiserror::Error;

/// The number of words a bare `disassemble` renders.
const DEFAULT_DSM_WINDOW: u16 = 8;

/// A parsed debugger command: one line of input resolved to an action.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Command {
    /// Execute `n` instructions, stopping early at a `HALT`.
    Step(u16),
    /// Run until a breakpoint, a `HALT`, or an error.
    Continue,
    /// Set a breakpoint at an address.
    Break(u16),
    /// Clear the breakpoint at an address.
    Delete(u16),
    /// List the active breakpoints.
    Breakpoints,
    /// Show the register file.
    Registers,
    /// Write a value to a register.
    SetRegister(Register, u16),
    /// Write a value to a memory address.
    WriteMemory(u16, u16),
    /// Disassemble a window of memory.
    Disassemble {
        /// The first address to render, or `None` for the program counter.
        address: Option<u16>,
        /// The number of words to render.
        len: u16,
    },
    /// Reload the program and return to its entry point.
    Reset,
    /// Show the command summary.
    Help,
    /// Leave the debugger.
    Quit,
}

/// A register a command can read or write: a general-purpose register or the
/// program counter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Register {
    /// General-purpose register `R0`--`R7`, identified by its 3-bit number.
    General(u16),
    /// The program counter.
    Pc,
}

/// The reason a command line could not be parsed.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum ParseError {
    /// The line held no command.
    #[error("empty command")]
    Empty,
    /// The first word names no known command.
    #[error("unknown command '{0}'")]
    UnknownCommand(String),
    /// A command was given fewer operands than it requires.
    #[error("'{command}' is missing an operand")]
    MissingArgument {
        /// The command that wanted more.
        command: &'static str,
    },
    /// A command was given more operands than it accepts.
    #[error("'{command}' was given too many operands")]
    UnexpectedArgument {
        /// The command that was over-supplied.
        command: &'static str,
    },
    /// An operand expected to be a number could not be read as one.
    #[error("invalid number '{0}'")]
    InvalidNumber(String),
    /// An operand expected to be a register names none.
    #[error("invalid register '{0}'; expected R0-R7 or PC")]
    InvalidRegister(String),
}

/// Parses one command `line` into a [`Command`].
///
/// The first whitespace-separated word is the command keyword (case-insensitive,
/// with aliases); the rest are its operands. A blank line yields
/// [`ParseError::Empty`]; an unrecognized keyword, a missing or surplus operand,
/// or an operand of the wrong shape yields the matching [`ParseError`].
pub fn parse(line: &str) -> Result<Command, ParseError> {
    let mut words = line.split_whitespace();
    let keyword = words.next().ok_or(ParseError::Empty)?;
    let args: Vec<&str> = words.collect();

    match keyword.to_ascii_lowercase().as_str() {
        "step" | "s" | "si" => parse_step(&args),
        "continue" | "c" => nullary(Command::Continue, "continue", &args),
        "break" | "b" => unary_address(&args, "break", Command::Break),
        "delete" | "d" => unary_address(&args, "delete", Command::Delete),
        "breaks" => nullary(Command::Breakpoints, "breaks", &args),
        "registers" | "regs" => nullary(Command::Registers, "registers", &args),
        "set" => parse_set(&args),
        "write" | "w" => parse_write(&args),
        "disassemble" | "dis" | "x" => parse_disassemble(&args),
        "reset" => nullary(Command::Reset, "reset", &args),
        "help" | "h" | "?" => nullary(Command::Help, "help", &args),
        "quit" | "q" => nullary(Command::Quit, "quit", &args),
        _ => Err(ParseError::UnknownCommand(keyword.to_string())),
    }
}

/// Resolves a command that takes no operands, rejecting any that are supplied.
fn nullary(command: Command, name: &'static str, args: &[&str]) -> Result<Command, ParseError> {
    if args.is_empty() {
        Ok(command)
    } else {
        Err(ParseError::UnexpectedArgument { command: name })
    }
}

/// Resolves a command that takes exactly one address operand.
fn unary_address(
    args: &[&str],
    name: &'static str,
    build: impl Fn(u16) -> Command,
) -> Result<Command, ParseError> {
    match args {
        [] => Err(ParseError::MissingArgument { command: name }),
        [address] => Ok(build(parse_u16(address)?)),
        _ => Err(ParseError::UnexpectedArgument { command: name }),
    }
}

/// `step [n]`: a step count defaulting to one.
fn parse_step(args: &[&str]) -> Result<Command, ParseError> {
    match args {
        [] => Ok(Command::Step(1)),
        [count] => Ok(Command::Step(parse_u16(count)?)),
        _ => Err(ParseError::UnexpectedArgument { command: "step" }),
    }
}

/// `set <register> <value>`.
fn parse_set(args: &[&str]) -> Result<Command, ParseError> {
    match args {
        [register, value] => Ok(Command::SetRegister(
            parse_register(register)?,
            parse_u16(value)?,
        )),
        [] | [_] => Err(ParseError::MissingArgument { command: "set" }),
        _ => Err(ParseError::UnexpectedArgument { command: "set" }),
    }
}

/// `write <address> <value>`.
fn parse_write(args: &[&str]) -> Result<Command, ParseError> {
    match args {
        [address, value] => Ok(Command::WriteMemory(parse_u16(address)?, parse_u16(value)?)),
        [] | [_] => Err(ParseError::MissingArgument { command: "write" }),
        _ => Err(ParseError::UnexpectedArgument { command: "write" }),
    }
}

/// `disassemble [address] [len]`: address defaulting to the program counter and
/// length to [`DEFAULT_DSM_WINDOW`].
fn parse_disassemble(args: &[&str]) -> Result<Command, ParseError> {
    match args {
        [] => Ok(Command::Disassemble {
            address: None,
            len: DEFAULT_DSM_WINDOW,
        }),
        [address] => Ok(Command::Disassemble {
            address: Some(parse_u16(address)?),
            len: DEFAULT_DSM_WINDOW,
        }),
        [address, len] => Ok(Command::Disassemble {
            address: Some(parse_u16(address)?),
            len: parse_u16(len)?,
        }),
        _ => Err(ParseError::UnexpectedArgument {
            command: "disassemble",
        }),
    }
}

/// Parses a 16-bit value in any accepted form: `x`/`0x` hex, or decimal with an
/// optional `#` prefix and sign (a negative wraps into its two's complement).
fn parse_u16(token: &str) -> Result<u16, ParseError> {
    let invalid = || ParseError::InvalidNumber(token.to_string());

    if let Some(hex) = token
        .strip_prefix("0x")
        .or_else(|| token.strip_prefix("0X"))
        .or_else(|| token.strip_prefix('x'))
        .or_else(|| token.strip_prefix('X'))
    {
        return u16::from_str_radix(hex, 16).map_err(|_| invalid());
    }

    token
        .strip_prefix('#')
        .unwrap_or(token)
        .parse::<i32>()
        .ok()
        .filter(|value| (-0x8000..=0xFFFF).contains(value))
        .map(|value| value as u16)
        .ok_or_else(invalid)
}

/// Parses a register name: `R0`--`R7` or `PC`, case-insensitively.
fn parse_register(token: &str) -> Result<Register, ParseError> {
    let name = token.to_ascii_uppercase();
    if name == "PC" {
        return Ok(Register::Pc);
    }
    name.strip_prefix('R')
        .and_then(|number| number.parse::<u16>().ok())
        .filter(|number| *number < 8)
        .map(Register::General)
        .ok_or_else(|| ParseError::InvalidRegister(token.to_string()))
}

#[cfg(test)]
mod tests {
    use super::{Command, ParseError, Register, parse};

    #[test]
    fn parses_execution_and_meta_commands() {
        assert_eq!(parse("step"), Ok(Command::Step(1)));
        assert_eq!(parse("s 4"), Ok(Command::Step(4)));
        assert_eq!(parse("si 16"), Ok(Command::Step(16)));
        assert_eq!(parse("continue"), Ok(Command::Continue));
        assert_eq!(parse("c"), Ok(Command::Continue));
        assert_eq!(parse("reset"), Ok(Command::Reset));
        assert_eq!(parse("help"), Ok(Command::Help));
        assert_eq!(parse("quit"), Ok(Command::Quit));
    }

    #[test]
    fn parses_breakpoint_commands() {
        assert_eq!(parse("break x3001"), Ok(Command::Break(0x3001)));
        assert_eq!(parse("b x3001"), Ok(Command::Break(0x3001)));
        assert_eq!(parse("delete x3001"), Ok(Command::Delete(0x3001)));
        assert_eq!(parse("d x3001"), Ok(Command::Delete(0x3001)));
        assert_eq!(parse("breaks"), Ok(Command::Breakpoints));
    }

    #[test]
    fn parses_inspection_and_edit_commands() {
        assert_eq!(parse("registers"), Ok(Command::Registers));
        assert_eq!(parse("regs"), Ok(Command::Registers));
        assert_eq!(
            parse("set R3 x00FF"),
            Ok(Command::SetRegister(Register::General(3), 0x00FF))
        );
        assert_eq!(
            parse("set PC x3000"),
            Ok(Command::SetRegister(Register::Pc, 0x3000))
        );
        assert_eq!(
            parse("write x4000 xBEEF"),
            Ok(Command::WriteMemory(0x4000, 0xBEEF))
        );
        assert_eq!(
            parse("disassemble"),
            Ok(Command::Disassemble {
                address: None,
                len: 8
            })
        );
        assert_eq!(
            parse("dis x3000"),
            Ok(Command::Disassemble {
                address: Some(0x3000),
                len: 8
            })
        );
        assert_eq!(
            parse("x x3000 4"),
            Ok(Command::Disassemble {
                address: Some(0x3000),
                len: 4
            })
        );
    }

    #[test]
    fn accepts_hex_decimal_and_signed_number_forms() {
        assert_eq!(parse("break x3001"), Ok(Command::Break(0x3001)));
        assert_eq!(parse("break 0x3001"), Ok(Command::Break(0x3001)));
        assert_eq!(parse("break 12289"), Ok(Command::Break(12289)));
        assert_eq!(parse("break #12289"), Ok(Command::Break(12289)));
        // A negative decimal wraps into its two's-complement word.
        assert_eq!(
            parse("set R0 #-1"),
            Ok(Command::SetRegister(Register::General(0), 0xFFFF))
        );
    }

    #[test]
    fn command_keywords_and_registers_are_case_insensitive() {
        assert_eq!(parse("STEP 2"), Ok(Command::Step(2)));
        assert_eq!(parse("Continue"), Ok(Command::Continue));
        assert_eq!(
            parse("SET r5 X10"),
            Ok(Command::SetRegister(Register::General(5), 0x10))
        );
    }

    #[test]
    fn rejects_malformed_input() {
        assert_eq!(parse(""), Err(ParseError::Empty));
        assert_eq!(parse("   "), Err(ParseError::Empty));
        assert!(matches!(
            parse("frobnicate"),
            Err(ParseError::UnknownCommand(_))
        ));
        assert!(matches!(
            parse("break"),
            Err(ParseError::MissingArgument { .. })
        ));
        assert!(matches!(
            parse("continue now"),
            Err(ParseError::UnexpectedArgument { .. })
        ));
        assert!(matches!(
            parse("break xZZZZ"),
            Err(ParseError::InvalidNumber(_))
        ));
        assert!(matches!(
            parse("set R9 1"),
            Err(ParseError::InvalidRegister(_))
        ));
        assert!(matches!(
            parse("set R0"),
            Err(ParseError::MissingArgument { .. })
        ));
    }
}

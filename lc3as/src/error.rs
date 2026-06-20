use std::error::Error;
use std::fmt;

/// The reason a source program could not be tokenized.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LexError {
    /// A `#` or `x` literal, or a bare digit run, that is not a well-formed
    /// number.
    MalformedNumber {
        /// The line of the offending literal.
        line: usize,
        /// The column of the offending literal.
        column: usize,
    },
    /// A string literal with no closing quote before the end of its line.
    UnterminatedString {
        /// The line of the opening quote.
        line: usize,
        /// The column of the opening quote.
        column: usize,
    },
    /// A backslash escape in a string literal that names no known escape.
    InvalidEscape {
        /// The line of the offending escape.
        line: usize,
        /// The column of the offending escape.
        column: usize,
        /// The character that followed the backslash.
        escape: char,
    },
    /// A `.` with no directive name following it.
    MalformedDirective {
        /// The line of the lone dot.
        line: usize,
        /// The column of the lone dot.
        column: usize,
    },
    /// A character that cannot begin any token.
    UnexpectedChar {
        /// The line of the stray character.
        line: usize,
        /// The column of the stray character.
        column: usize,
        /// The character itself.
        ch: char,
    },
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::MalformedNumber { line, column } => {
                write!(f, "line {line}:{column}: malformed numeric literal")
            }
            Self::UnterminatedString { line, column } => {
                write!(f, "line {line}:{column}: unterminated string literal")
            }
            Self::InvalidEscape {
                line,
                column,
                escape,
            } => write!(
                f,
                "line {line}:{column}: invalid string escape '\\{escape}'"
            ),
            Self::MalformedDirective { line, column } => {
                write!(
                    f,
                    "line {line}:{column}: expected a directive name after '.'"
                )
            }
            Self::UnexpectedChar { line, column, ch } => {
                write!(f, "line {line}:{column}: unexpected character '{ch}'")
            }
        }
    }
}

impl Error for LexError {}

/// The reason a token stream could not be parsed into a program.
///
/// Every variant but [`MissingOrig`](Self::MissingOrig) carries the 1-based
/// source line at fault.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// Tokenizing the source failed.
    Lex(LexError),
    /// A code- or data-emitting statement appears outside any `.ORIG`/`.END`.
    StatementOutsideSegment {
        /// The offending line.
        line: usize,
    },
    /// A `.ORIG` opens a segment while another is still open.
    NestedSegment {
        /// The offending `.ORIG` line.
        line: usize,
    },
    /// A `.END` appears with no open `.ORIG`.
    UnmatchedEnd {
        /// The offending `.END` line.
        line: usize,
    },
    /// A `.ORIG` is never closed by a `.END`.
    UnterminatedSegment {
        /// The line of the unclosed `.ORIG`.
        line: usize,
    },
    /// The program contains no `.ORIG` segment.
    MissingOrig,
    /// A label is attached to `.ORIG` or `.END`.
    LabelOnDirective {
        /// The offending line.
        line: usize,
    },
    /// A label is defined more than once.
    DuplicateLabel {
        /// The line of the redefinition.
        line: usize,
        /// The duplicated label.
        label: String,
    },
    /// A segment's contents would extend past `xFFFF`.
    ProgramOverflow {
        /// The line of the statement that overruns memory.
        line: usize,
    },
    /// A `.ORIG` origin or `.BLKW` count does not fit a 16-bit word.
    ValueOutOfRange {
        /// The offending line.
        line: usize,
    },
    /// A word in operation position names no known instruction.
    UnknownMnemonic {
        /// The offending line.
        line: usize,
        /// The unrecognized mnemonic.
        mnemonic: String,
    },
    /// A `.`-directive names no known pseudo-op.
    UnknownDirective {
        /// The offending line.
        line: usize,
        /// The unrecognized directive.
        directive: String,
    },
    /// An operand of the named kind was expected but not found.
    ExpectedOperand {
        /// The offending line.
        line: usize,
        /// A description of what was expected.
        expected: &'static str,
    },
    /// A statement carries more tokens than its form allows.
    UnexpectedToken {
        /// The offending line.
        line: usize,
    },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lex(error) => write!(f, "{error}"),
            Self::StatementOutsideSegment { line } => {
                write!(f, "line {line}: statement outside any .ORIG/.END segment")
            }
            Self::NestedSegment { line } => {
                write!(
                    f,
                    "line {line}: .ORIG inside an open segment; close it with .END first"
                )
            }
            Self::UnmatchedEnd { line } => {
                write!(f, "line {line}: .END without a matching .ORIG")
            }
            Self::UnterminatedSegment { line } => {
                write!(f, "line {line}: .ORIG is never closed by .END")
            }
            Self::MissingOrig => write!(f, "program has no .ORIG segment"),
            Self::LabelOnDirective { line } => {
                write!(
                    f,
                    "line {line}: a label cannot be attached to .ORIG or .END"
                )
            }
            Self::DuplicateLabel { line, label } => {
                write!(f, "line {line}: duplicate label '{label}'")
            }
            Self::ProgramOverflow { line } => {
                write!(
                    f,
                    "line {line}: program runs past the end of memory (xFFFF)"
                )
            }
            Self::ValueOutOfRange { line } => {
                write!(f, "line {line}: value does not fit a 16-bit word")
            }
            Self::UnknownMnemonic { line, mnemonic } => {
                write!(f, "line {line}: unknown instruction '{mnemonic}'")
            }
            Self::UnknownDirective { line, directive } => {
                write!(f, "line {line}: unknown directive '{directive}'")
            }
            Self::ExpectedOperand { line, expected } => {
                write!(f, "line {line}: expected {expected}")
            }
            Self::UnexpectedToken { line } => write!(f, "line {line}: unexpected token"),
        }
    }
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Lex(error) => Some(error),
            _ => None,
        }
    }
}

impl From<LexError> for ParseError {
    fn from(error: LexError) -> Self {
        Self::Lex(error)
    }
}

/// The reason a parsed program could not be encoded into object words.
///
/// [`Parse`](Self::Parse) wraps a first-pass failure; the remaining variants are
/// second-pass faults---an unresolved label, or a value too large for the
/// instruction field that holds it---each carrying the 1-based source line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AsmError {
    /// The first pass---lexing and parsing---failed.
    Parse(ParseError),
    /// A label reference names no symbol defined anywhere in the program.
    UndefinedLabel {
        /// The line of the reference.
        line: usize,
        /// The undefined label.
        label: String,
    },
    /// A PC-relative or base offset does not fit its instruction field.
    OffsetOutOfRange {
        /// The line of the instruction.
        line: usize,
        /// The offset that overflowed the field.
        offset: i32,
        /// The width of the field, in bits.
        bits: u32,
    },
    /// An `ADD`/`AND` immediate does not fit the five-bit `imm5` field.
    ImmediateOutOfRange {
        /// The line of the instruction.
        line: usize,
        /// The immediate that overflowed the field.
        value: i32,
    },
    /// A `.FILL` word, a trap vector, or a string character does not fit its
    /// field.
    ValueOutOfRange {
        /// The line of the offending value.
        line: usize,
        /// The value that overflowed the field.
        value: i32,
        /// The width of the field, in bits.
        bits: u32,
    },
}

impl fmt::Display for AsmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(error) => write!(f, "{error}"),
            Self::UndefinedLabel { line, label } => {
                write!(f, "line {line}: undefined label '{label}'")
            }
            Self::OffsetOutOfRange { line, offset, bits } => {
                write!(
                    f,
                    "line {line}: offset {offset} does not fit a {bits}-bit field"
                )
            }
            Self::ImmediateOutOfRange { line, value } => {
                write!(
                    f,
                    "line {line}: immediate {value} does not fit the 5-bit field"
                )
            }
            Self::ValueOutOfRange { line, value, bits } => {
                write!(
                    f,
                    "line {line}: value {value} does not fit a {bits}-bit field"
                )
            }
        }
    }
}

impl Error for AsmError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Parse(error) => Some(error),
            _ => None,
        }
    }
}

impl From<ParseError> for AsmError {
    fn from(error: ParseError) -> Self {
        Self::Parse(error)
    }
}

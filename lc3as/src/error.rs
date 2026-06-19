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

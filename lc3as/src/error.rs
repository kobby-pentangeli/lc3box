use thiserror::Error;

/// The reason a source program could not be tokenized.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum LexError {
    /// A `#` or `x` literal, or a bare digit run, that is not a well-formed
    /// number.
    #[error("line {line}:{column}: malformed numeric literal")]
    MalformedNumber {
        /// The line of the offending literal.
        line: usize,
        /// The column of the offending literal.
        column: usize,
    },
    /// A string literal with no closing quote before the end of its line.
    #[error("line {line}:{column}: unterminated string literal")]
    UnterminatedString {
        /// The line of the opening quote.
        line: usize,
        /// The column of the opening quote.
        column: usize,
    },
    /// A backslash escape in a string literal that names no known escape.
    #[error("line {line}:{column}: invalid string escape '\\{escape}'")]
    InvalidEscape {
        /// The line of the offending escape.
        line: usize,
        /// The column of the offending escape.
        column: usize,
        /// The character that followed the backslash.
        escape: char,
    },
    /// A `.` with no directive name following it.
    #[error("line {line}:{column}: expected a directive name after '.'")]
    MalformedDirective {
        /// The line of the lone dot.
        line: usize,
        /// The column of the lone dot.
        column: usize,
    },
    /// A character that cannot begin any token.
    #[error("line {line}:{column}: unexpected character '{ch}'")]
    UnexpectedChar {
        /// The line of the stray character.
        line: usize,
        /// The column of the stray character.
        column: usize,
        /// The character itself.
        ch: char,
    },
}

/// The reason a token stream could not be parsed into a program.
///
/// Every variant but [`MissingOrig`](Self::MissingOrig) carries the 1-based
/// source line at fault.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum ParseError {
    /// Tokenizing the source failed.
    #[error("{0}")]
    Lex(#[from] LexError),
    /// A code- or data-emitting statement appears outside any `.ORIG`/`.END`.
    #[error("line {line}: statement outside any .ORIG/.END segment")]
    StatementOutsideSegment {
        /// The offending line.
        line: usize,
    },
    /// A `.ORIG` opens a segment while another is still open.
    #[error("line {line}: .ORIG inside an open segment; close it with .END first")]
    NestedSegment {
        /// The offending `.ORIG` line.
        line: usize,
    },
    /// A `.END` appears with no open `.ORIG`.
    #[error("line {line}: .END without a matching .ORIG")]
    UnmatchedEnd {
        /// The offending `.END` line.
        line: usize,
    },
    /// A `.ORIG` is never closed by a `.END`.
    #[error("line {line}: .ORIG is never closed by .END")]
    UnterminatedSegment {
        /// The line of the unclosed `.ORIG`.
        line: usize,
    },
    /// The program contains no `.ORIG` segment.
    #[error("program has no .ORIG segment")]
    MissingOrig,
    /// A label is attached to `.ORIG` or `.END`.
    #[error("line {line}: a label cannot be attached to .ORIG or .END")]
    LabelOnDirective {
        /// The offending line.
        line: usize,
    },
    /// A label is defined more than once.
    #[error("line {line}: duplicate label '{label}'")]
    DuplicateLabel {
        /// The line of the redefinition.
        line: usize,
        /// The duplicated label.
        label: String,
    },
    /// A segment's contents would extend past `xFFFF`.
    #[error("line {line}: program runs past the end of memory (xFFFF)")]
    ProgramOverflow {
        /// The line of the statement that overruns memory.
        line: usize,
    },
    /// A `.ORIG` origin or `.BLKW` count does not fit a 16-bit word.
    #[error("line {line}: value does not fit a 16-bit word")]
    ValueOutOfRange {
        /// The offending line.
        line: usize,
    },
    /// A word in operation position names no known instruction.
    #[error("line {line}: unknown instruction '{mnemonic}'")]
    UnknownMnemonic {
        /// The offending line.
        line: usize,
        /// The unrecognized mnemonic.
        mnemonic: String,
    },
    /// A `.`-directive names no known pseudo-op.
    #[error("line {line}: unknown directive '{directive}'")]
    UnknownDirective {
        /// The offending line.
        line: usize,
        /// The unrecognized directive.
        directive: String,
    },
    /// An operand of the named kind was expected but not found.
    #[error("line {line}: expected {expected}")]
    ExpectedOperand {
        /// The offending line.
        line: usize,
        /// A description of what was expected.
        expected: &'static str,
    },
    /// A statement carries more tokens than its form allows.
    #[error("line {line}: unexpected token")]
    UnexpectedToken {
        /// The offending line.
        line: usize,
    },
}

/// The reason a parsed program could not be encoded into object words.
///
/// [`Parse`](Self::Parse) wraps a first-pass failure; the remaining variants are
/// second-pass faults---an unresolved label, or a value too large for the
/// instruction field that holds it---each carrying the 1-based source line.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum AsmError {
    /// The first pass---lexing and parsing---failed.
    #[error("{0}")]
    Parse(#[from] ParseError),
    /// A label reference names no symbol defined anywhere in the program.
    #[error("line {line}: undefined label '{label}'")]
    UndefinedLabel {
        /// The line of the reference.
        line: usize,
        /// The undefined label.
        label: String,
    },
    /// A PC-relative or base offset does not fit its instruction field.
    #[error("line {line}: offset {offset} does not fit a {bits}-bit field")]
    OffsetOutOfRange {
        /// The line of the instruction.
        line: usize,
        /// The offset that overflowed the field.
        offset: i32,
        /// The width of the field, in bits.
        bits: u32,
    },
    /// An `ADD`/`AND` immediate does not fit the five-bit `imm5` field.
    #[error("line {line}: immediate {value} does not fit the 5-bit field")]
    ImmediateOutOfRange {
        /// The line of the instruction.
        line: usize,
        /// The immediate that overflowed the field.
        value: i32,
    },
    /// A `.FILL` word, a trap vector, or a string character does not fit its
    /// field.
    #[error("line {line}: value {value} does not fit a {bits}-bit field")]
    ValueOutOfRange {
        /// The line of the offending value.
        line: usize,
        /// The value that overflowed the field.
        value: i32,
        /// The width of the field, in bits.
        bits: u32,
    },
}

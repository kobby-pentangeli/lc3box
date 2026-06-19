//! Lexer for LC-3 assembly source.
//!
//! [`tokenize`] scans a program into a flat, positioned token stream. The scan
//! is purely lexical: it recognizes the surface shapes of the language---words,
//! directives, numbers, strings, commas, and line breaks---without deciding
//! whether a word is a label, a mnemonic, or a register. That classification
//! depends on a token's position in its statement and belongs to the parser.

use std::error::Error;
use std::fmt;

/// A lexical token together with its position in the source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token<'a> {
    /// What the token is.
    pub kind: TokenKind<'a>,
    /// The 1-based line the token starts on.
    pub line: usize,
    /// The 1-based column the token starts at.
    pub column: usize,
}

impl<'a> Token<'a> {
    fn new(kind: TokenKind<'a>, line: usize, column: usize) -> Self {
        Self { kind, line, column }
    }
}

/// The kinds of lexical token in LC-3 assembly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind<'a> {
    /// A bare word: a label, an instruction mnemonic, a register name, a branch
    /// mnemonic, or a trap alias. Which one it is depends on its position in the
    /// statement and is resolved when parsing, not here.
    Word(&'a str),
    /// A pseudo-op directive lexeme, including its leading dot (`.ORIG`, ...).
    Directive(&'a str),
    /// A numeric literal---`#`-prefixed decimal (possibly negative) or
    /// `x`-prefixed hexadecimal---parsed to its integer value.
    Number(i32),
    /// A string literal with its escape sequences already decoded.
    Str(String),
    /// An operand separator, `,`.
    Comma,
    /// The end of a source line.
    Newline,
}

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

/// Tokenizes LC-3 assembly `source` into a positioned token stream.
///
/// The scan is line-oriented: each source line yields its tokens followed by a
/// [`Newline`](TokenKind::Newline). A `;` begins a comment that runs to the end
/// of the line and is discarded. Numbers are `#`-prefixed decimal (optionally
/// negative) or `x`-prefixed hexadecimal. String literals decode the escapes
/// `\n`, `\t`, `\r`, `\0`, `\"`, and `\\`. Words---labels, mnemonics, registers,
/// branch and trap aliases---are returned verbatim for the parser to classify.
///
/// Returns a [`LexError`] at the first malformed number, unterminated string,
/// invalid escape, nameless directive, or character that begins no token.
pub fn tokenize(source: &str) -> Result<Vec<Token<'_>>, LexError> {
    source
        .lines()
        .enumerate()
        .try_fold(Vec::new(), |mut tokens, (index, line)| {
            scan_line(line, index + 1, &mut tokens)?;
            Ok(tokens)
        })
}

fn scan_line<'a>(line: &'a str, line_no: usize, out: &mut Vec<Token<'a>>) -> Result<(), LexError> {
    let mut i = 0;
    while i < line.len() {
        let Some(c) = line[i..].chars().next() else {
            break;
        };
        let column = i + 1;
        match c {
            ' ' | '\t' | '\r' => i += 1,
            ';' => break,
            ',' => {
                out.push(Token::new(TokenKind::Comma, line_no, column));
                i += 1;
            }
            '"' => {
                let (value, end) = scan_string(line, i, line_no)?;
                out.push(Token::new(TokenKind::Str(value), line_no, column));
                i = end;
            }
            '#' => {
                let mut digits = i + 1;
                if line[digits..].starts_with('-') {
                    digits += 1;
                }
                let end = ident_run_end(line, digits);
                let value =
                    line[i + 1..end]
                        .parse::<i32>()
                        .map_err(|_| LexError::MalformedNumber {
                            line: line_no,
                            column,
                        })?;
                out.push(Token::new(TokenKind::Number(value), line_no, column));
                i = end;
            }
            '.' => {
                let end = ident_run_end(line, i + 1);
                if end == i + 1 {
                    return Err(LexError::MalformedDirective {
                        line: line_no,
                        column,
                    });
                }
                out.push(Token::new(
                    TokenKind::Directive(&line[i..end]),
                    line_no,
                    column,
                ));
                i = end;
            }
            _ if is_ident_char(c) => {
                let end = ident_run_end(line, i);
                let kind = classify_word(&line[i..end], line_no, column)?;
                out.push(Token::new(kind, line_no, column));
                i = end;
            }
            ch => {
                return Err(LexError::UnexpectedChar {
                    line: line_no,
                    column,
                    ch,
                });
            }
        }
    }
    out.push(Token::new(TokenKind::Newline, line_no, line.len() + 1));
    Ok(())
}

/// Scans a string literal opening at `start`, returning its decoded contents and
/// the byte index just past the closing quote.
fn scan_string(line: &str, start: usize, line_no: usize) -> Result<(String, usize), LexError> {
    let mut value = String::new();
    let mut iter = line[start + 1..].char_indices();
    while let Some((offset, c)) = iter.next() {
        match c {
            '"' => return Ok((value, start + 1 + offset + 1)),
            '\\' => {
                let Some((escape_offset, escape)) = iter.next() else {
                    return Err(LexError::UnterminatedString {
                        line: line_no,
                        column: start + 1,
                    });
                };
                value.push(match escape {
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    '0' => '\0',
                    '"' => '"',
                    '\\' => '\\',
                    other => {
                        return Err(LexError::InvalidEscape {
                            line: line_no,
                            column: start + 1 + escape_offset + 1,
                            escape: other,
                        });
                    }
                });
            }
            other => value.push(other),
        }
    }
    Err(LexError::UnterminatedString {
        line: line_no,
        column: start + 1,
    })
}

fn ident_run_end(line: &str, start: usize) -> usize {
    line[start..]
        .find(|c: char| !is_ident_char(c))
        .map_or(line.len(), |offset| start + offset)
}

fn is_ident_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Classifies a maximal identifier run as a hexadecimal number or a word.
///
/// A run of `x`/`X` followed by one or more hex digits is the hexadecimal form;
/// any other letter- or underscore-led run is a word. A digit-led run that is
/// not a hex literal is a stray number, since LC-3 numbers must be prefixed.
fn classify_word<'a>(
    word: &'a str,
    line_no: usize,
    column: usize,
) -> Result<TokenKind<'a>, LexError> {
    if let Some(digits) = word.strip_prefix(['x', 'X'])
        && !digits.is_empty()
        && digits.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return i32::from_str_radix(digits, 16)
            .map(TokenKind::Number)
            .map_err(|_| LexError::MalformedNumber {
                line: line_no,
                column,
            });
    }

    match word.chars().next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => Ok(TokenKind::Word(word)),
        _ => Err(LexError::MalformedNumber {
            line: line_no,
            column,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::{LexError, TokenKind, tokenize};

    fn kinds(source: &str) -> Vec<TokenKind<'_>> {
        tokenize(source)
            .expect("source tokenizes")
            .into_iter()
            .map(|token| token.kind)
            .collect()
    }

    #[test]
    fn an_instruction_yields_words_commas_a_number_and_a_newline() {
        assert_eq!(
            kinds("ADD R1, R2, #-1"),
            vec![
                TokenKind::Word("ADD"),
                TokenKind::Word("R1"),
                TokenKind::Comma,
                TokenKind::Word("R2"),
                TokenKind::Comma,
                TokenKind::Number(-1),
                TokenKind::Newline,
            ]
        );
    }

    #[test]
    fn hexadecimal_and_decimal_number_forms_parse_to_their_values() {
        let numbers = |source| match kinds(source).as_slice() {
            [TokenKind::Number(value), TokenKind::Newline] => *value,
            other => panic!("expected a single number, got {other:?}"),
        };
        assert_eq!(numbers("x3000"), 0x3000);
        assert_eq!(numbers("xFFFF"), 0xFFFF);
        assert_eq!(numbers("x0"), 0);
        assert_eq!(numbers("#5"), 5);
        assert_eq!(numbers("#-16"), -16);
    }

    #[test]
    fn a_directive_and_string_literal_tokenize() {
        assert_eq!(
            kinds(".STRINGZ \"hi\""),
            vec![
                TokenKind::Directive(".STRINGZ"),
                TokenKind::Str("hi".to_string()),
                TokenKind::Newline,
            ]
        );
    }

    #[test]
    fn string_escapes_are_decoded() {
        // The source is a literal containing \n \t \r \0 \" \\ between quotes.
        match kinds(r#""\n\t\r\0\"\\""#).as_slice() {
            [TokenKind::Str(decoded), TokenKind::Newline] => {
                assert_eq!(decoded, "\n\t\r\u{0}\"\\");
            }
            other => panic!("expected a single string, got {other:?}"),
        }
    }

    #[test]
    fn comments_are_stripped_and_a_comment_only_line_is_empty() {
        // A trailing comment, a whole-line comment, then a real instruction.
        assert_eq!(
            kinds("HALT ; stop here\n; nothing on this line\nRET"),
            vec![
                TokenKind::Word("HALT"),
                TokenKind::Newline,
                TokenKind::Newline,
                TokenKind::Word("RET"),
                TokenKind::Newline,
            ]
        );
    }

    #[test]
    fn tokens_carry_one_based_line_and_column() {
        let tokens = tokenize("  ADD\nRET").expect("tokenizes");
        assert_eq!((tokens[0].line, tokens[0].column), (1, 3));
        let ret = tokens
            .iter()
            .find(|t| t.kind == TokenKind::Word("RET"))
            .expect("RET present");
        assert_eq!((ret.line, ret.column), (2, 1));
    }

    #[test]
    fn an_x_prefixed_word_is_hex_only_when_all_digits_are_hex() {
        assert_eq!(kinds("x3000")[0], TokenKind::Number(0x3000));
        assert_eq!(kinds("xResult")[0], TokenKind::Word("xResult"));
        assert_eq!(kinds("R3")[0], TokenKind::Word("R3"));
    }

    #[test]
    fn a_bare_or_malformed_number_is_rejected() {
        assert_eq!(
            tokenize("42"),
            Err(LexError::MalformedNumber { line: 1, column: 1 })
        );
        assert_eq!(
            tokenize("#1a"),
            Err(LexError::MalformedNumber { line: 1, column: 1 })
        );
    }

    #[test]
    fn an_unterminated_string_is_rejected() {
        assert_eq!(
            tokenize("\"hello"),
            Err(LexError::UnterminatedString { line: 1, column: 1 })
        );
    }

    #[test]
    fn an_invalid_escape_is_rejected() {
        assert_eq!(
            tokenize(r#""\q""#),
            Err(LexError::InvalidEscape {
                line: 1,
                column: 3,
                escape: 'q',
            })
        );
    }

    #[test]
    fn a_dot_without_a_name_is_rejected() {
        assert_eq!(
            tokenize(". FILL"),
            Err(LexError::MalformedDirective { line: 1, column: 1 })
        );
    }

    #[test]
    fn a_stray_character_is_rejected() {
        assert_eq!(
            tokenize("ADD R1 : R2"),
            Err(LexError::UnexpectedChar {
                line: 1,
                column: 8,
                ch: ':',
            })
        );
    }

    #[test]
    fn the_bootstrap_example_tokenizes_cleanly() {
        // Exercises the lexer over a full, real program: the three `.stringz`
        // strings must be the only string tokens, proving that the quotes and
        // apostrophes inside `;;` comment lines are correctly stripped.
        let source = include_str!("../../examples/bootstrap.asm");
        let tokens = tokenize(source).expect("bootstrap tokenizes");
        let strings = tokens
            .iter()
            .filter(|t| matches!(t.kind, TokenKind::Str(_)))
            .count();
        assert_eq!(strings, 3);
    }
}

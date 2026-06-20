//! Parser and symbol table: the assembler's first pass.
//!
//! [`parse`] turns the lexer's token stream into a [`Program`]: a sequence of
//! `.ORIG`/`.END` segments, each statement bound to its address, plus one global
//! symbol table mapping every label to its address. Mnemonics are bound to their
//! forms here---the `BR` family, `RET`/`JSRR`, and the trap aliases resolve
//! through the shared [`lc3core`] vocabulary---but label references are kept as
//! names; resolving them to offsets and encoding the words is the second pass.

use std::collections::BTreeMap;

use lc3core::{Opcode, PseudoOp, TrapVector, parse_branch_condition, parse_register};

use crate::{ParseError, Token, TokenKind, tokenize};

/// A parsed program: its located segments and a global symbol table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program<'a> {
    /// The `.ORIG`/`.END` segments, in source order.
    pub segments: Vec<Segment<'a>>,
    /// Every label in the program, mapped to its address.
    pub symbols: BTreeMap<&'a str, u16>,
}

/// One `.ORIG`/`.END` segment: an origin and the statements placed after it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segment<'a> {
    /// The load address set by `.ORIG`.
    pub origin: u16,
    /// The emitting statements of the segment, each at its assigned address.
    pub statements: Vec<Statement<'a>>,
}

/// A single emitting statement placed at a known address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Statement<'a> {
    /// The address the statement's first word occupies.
    pub address: u16,
    /// The 1-based source line, for diagnostics.
    pub line: usize,
    /// The operation to encode in the second pass.
    pub operation: Operation<'a>,
}

/// A parsed operation: one machine instruction or one data-emitting pseudo-op.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operation<'a> {
    /// `ADD`/`AND` in register mode: `DR <- SR1 op SR2`.
    AluReg {
        /// The operation, [`Opcode::Add`] or [`Opcode::And`].
        op: Opcode,
        /// The destination register.
        dr: u16,
        /// The first source register.
        sr1: u16,
        /// The second source register.
        sr2: u16,
    },
    /// `ADD`/`AND` in immediate mode: `DR <- SR1 op imm5`.
    AluImm {
        /// The operation, [`Opcode::Add`] or [`Opcode::And`].
        op: Opcode,
        /// The destination register.
        dr: u16,
        /// The first source register.
        sr1: u16,
        /// The immediate value, range-checked to five bits in the second pass.
        imm: i32,
    },
    /// `NOT DR, SR`.
    Not {
        /// The destination register.
        dr: u16,
        /// The source register.
        sr: u16,
    },
    /// `BR(nzp)`: branch on the condition bits to a PC-relative target.
    Branch {
        /// The `n`/`z`/`p` condition field.
        cond: u16,
        /// The branch destination.
        target: Target<'a>,
    },
    /// `JMP`/`JSRR`/`RET`: jump through a base register (`RET` is base `R7`).
    BaseReg {
        /// The operation, [`Opcode::Jmp`] or [`Opcode::Jsr`].
        op: Opcode,
        /// The base register holding the destination address.
        base: u16,
    },
    /// `JSR`: jump to subroutine at an 11-bit PC-relative target.
    Jsr {
        /// The subroutine destination.
        target: Target<'a>,
    },
    /// `LD`/`LDI`/`LEA`/`ST`/`STI`: a register and a 9-bit PC-relative target.
    PcRelative {
        /// The load or store operation.
        op: Opcode,
        /// The destination (load) or source (store) register.
        reg: u16,
        /// The PC-relative operand.
        target: Target<'a>,
    },
    /// `LDR`/`STR`: a register, a base register, and a 6-bit offset.
    BaseOffset {
        /// The load or store operation, [`Opcode::Ldr`] or [`Opcode::Str`].
        op: Opcode,
        /// The destination (load) or source (store) register.
        reg: u16,
        /// The base register.
        base: u16,
        /// The signed offset, range-checked to six bits in the second pass.
        offset: i32,
    },
    /// `TRAP` or one of its named aliases: an 8-bit trap vector.
    Trap {
        /// The trap vector, range-checked to eight unsigned bits in the second pass.
        vector: i32,
    },
    /// `RTI`: return from interrupt.
    Rti,
    /// `.FILL`: one word holding a literal value or a label's address.
    Fill(Fill<'a>),
    /// `.BLKW`: reserve `count` consecutive words.
    Blkw {
        /// The number of words reserved.
        count: u16,
    },
    /// `.STRINGZ`: a null-terminated string, one word per character.
    Stringz {
        /// The string contents, with escapes already decoded.
        text: String,
    },
}

/// The destination of a PC-relative operand: a label or an explicit offset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target<'a> {
    /// A label whose address is resolved in the second pass.
    Label(&'a str),
    /// An explicit PC-relative offset.
    Offset(i32),
}

/// The operand of `.FILL`: a literal value or a label's address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fill<'a> {
    /// A literal word value.
    Number(i32),
    /// A label whose address fills the word.
    Label(&'a str),
}

impl Operation<'_> {
    /// The number of words the statement occupies in memory.
    fn size(&self) -> usize {
        match self {
            Operation::Blkw { count } => usize::from(*count),
            Operation::Stringz { text } => text.chars().count() + 1,
            _ => 1,
        }
    }
}

/// Parses LC-3 assembly `source` into a [`Program`].
///
/// This is the assembler's first pass: it tokenizes the source, parses each line
/// into a statement bound to its form, walks the location counter through one or
/// more `.ORIG`/`.END` segments, and records every label in a global symbol
/// table. Label references are kept as names for the second pass to resolve.
///
/// Returns a [`ParseError`] for any lexical, syntactic, symbol, or structural
/// fault---an undefined label is not one, since that is detected when the second
/// pass resolves references against the completed table.
pub fn parse(source: &str) -> Result<Program<'_>, ParseError> {
    let tokens = tokenize(source)?;
    let mut pass = FirstPass::default();
    for line in tokens.split(|token| matches!(token.kind, TokenKind::Newline)) {
        if let Some(first) = line.first() {
            pass.feed(line, first.line)?;
        }
    }
    pass.finish()
}

#[derive(Default)]
struct FirstPass<'a> {
    segments: Vec<Segment<'a>>,
    symbols: BTreeMap<&'a str, u16>,
    open: Option<OpenSegment<'a>>,
}

struct OpenSegment<'a> {
    origin: u16,
    line: usize,
    counter: usize,
    statements: Vec<Statement<'a>>,
}

impl<'a> FirstPass<'a> {
    fn feed(&mut self, tokens: &[Token<'a>], line: usize) -> Result<(), ParseError> {
        let (label, body) = parse_line(tokens, line)?;
        match body {
            LineBody::Orig(origin) => {
                if label.is_some() {
                    return Err(ParseError::LabelOnDirective { line });
                }
                if self.open.is_some() {
                    return Err(ParseError::NestedSegment { line });
                }
                self.open = Some(OpenSegment {
                    origin,
                    line,
                    counter: usize::from(origin),
                    statements: Vec::new(),
                });
            }
            LineBody::End => {
                if label.is_some() {
                    return Err(ParseError::LabelOnDirective { line });
                }
                let open = self.open.take().ok_or(ParseError::UnmatchedEnd { line })?;
                self.segments.push(Segment {
                    origin: open.origin,
                    statements: open.statements,
                });
            }
            LineBody::Empty => {
                let counter = self
                    .counter()
                    .ok_or(ParseError::StatementOutsideSegment { line })?;
                let address =
                    u16::try_from(counter).map_err(|_| ParseError::ProgramOverflow { line })?;
                if let Some(name) = label {
                    self.define(name, address, line)?;
                }
            }
            LineBody::Statement(operation) => {
                let counter = self
                    .counter()
                    .ok_or(ParseError::StatementOutsideSegment { line })?;
                if counter + operation.size() > 0x1_0000 {
                    return Err(ParseError::ProgramOverflow { line });
                }
                let address =
                    u16::try_from(counter).map_err(|_| ParseError::ProgramOverflow { line })?;
                if let Some(name) = label {
                    self.define(name, address, line)?;
                }
                if let Some(open) = self.open.as_mut() {
                    open.counter += operation.size();
                    open.statements.push(Statement {
                        address,
                        line,
                        operation,
                    });
                }
            }
        }
        Ok(())
    }

    fn counter(&self) -> Option<usize> {
        self.open.as_ref().map(|open| open.counter)
    }

    fn define(&mut self, name: &'a str, address: u16, line: usize) -> Result<(), ParseError> {
        if self.symbols.insert(name, address).is_some() {
            return Err(ParseError::DuplicateLabel {
                line,
                label: name.to_string(),
            });
        }
        Ok(())
    }

    fn finish(self) -> Result<Program<'a>, ParseError> {
        if let Some(open) = self.open {
            return Err(ParseError::UnterminatedSegment { line: open.line });
        }
        if self.segments.is_empty() {
            return Err(ParseError::MissingOrig);
        }
        Ok(Program {
            segments: self.segments,
            symbols: self.symbols,
        })
    }
}

/// The body of a source line: the operation, if any, after an optional label.
enum LineBody<'a> {
    /// `.ORIG` opening a segment at the given origin.
    Orig(u16),
    /// `.END` closing the current segment.
    End,
    /// A line carrying only a label (it binds to the current address).
    Empty,
    /// An emitting instruction or pseudo-op.
    Statement(Operation<'a>),
}

/// A mnemonic resolved through the [`lc3core`] vocabulary.
enum Mnemonic {
    Branch(u16),
    Trap(TrapVector),
    Op(Opcode),
}

fn classify_mnemonic(word: &str) -> Option<Mnemonic> {
    if let Some(cond) = parse_branch_condition(word) {
        Some(Mnemonic::Branch(cond))
    } else if let Some(trap) = TrapVector::from_mnemonic(word) {
        Some(Mnemonic::Trap(trap))
    } else {
        Opcode::from_mnemonic(word).map(Mnemonic::Op)
    }
}

fn parse_line<'a>(
    tokens: &[Token<'a>],
    line: usize,
) -> Result<(Option<&'a str>, LineBody<'a>), ParseError> {
    match tokens.first().map(|token| &token.kind) {
        Some(&TokenKind::Directive(directive)) => {
            Ok((None, parse_directive(directive, &tokens[1..], line)?))
        }
        Some(&TokenKind::Word(word)) => match classify_mnemonic(word) {
            Some(mnemonic) => Ok((None, parse_operation(mnemonic, word, &tokens[1..], line)?)),
            None => parse_labeled(word, &tokens[1..], line),
        },
        _ => Err(ParseError::UnexpectedToken { line }),
    }
}

fn parse_labeled<'a>(
    label: &'a str,
    rest: &[Token<'a>],
    line: usize,
) -> Result<(Option<&'a str>, LineBody<'a>), ParseError> {
    match rest.first().map(|token| &token.kind) {
        None => Ok((Some(label), LineBody::Empty)),
        Some(&TokenKind::Directive(directive)) => {
            Ok((Some(label), parse_directive(directive, &rest[1..], line)?))
        }
        Some(&TokenKind::Word(word)) => match classify_mnemonic(word) {
            Some(mnemonic) => Ok((
                Some(label),
                parse_operation(mnemonic, word, &rest[1..], line)?,
            )),
            None => Err(ParseError::UnknownMnemonic {
                line,
                mnemonic: word.to_string(),
            }),
        },
        _ => Err(ParseError::UnexpectedToken { line }),
    }
}

fn parse_directive<'a>(
    directive: &str,
    operands: &[Token<'a>],
    line: usize,
) -> Result<LineBody<'a>, ParseError> {
    match PseudoOp::from_mnemonic(directive) {
        Some(PseudoOp::Orig) => {
            let (value, next) = take_number(operands, 0, line)?;
            take_end(operands, next, line)?;
            let origin = u16::try_from(value).map_err(|_| ParseError::ValueOutOfRange { line })?;
            Ok(LineBody::Orig(origin))
        }
        Some(PseudoOp::End) => {
            take_end(operands, 0, line)?;
            Ok(LineBody::End)
        }
        Some(PseudoOp::Fill) => {
            let value = match operands.first().map(|token| &token.kind) {
                Some(&TokenKind::Number(number)) => Fill::Number(number),
                Some(&TokenKind::Word(label)) => Fill::Label(label),
                _ => {
                    return Err(ParseError::ExpectedOperand {
                        line,
                        expected: "a number or label",
                    });
                }
            };
            take_end(operands, 1, line)?;
            Ok(LineBody::Statement(Operation::Fill(value)))
        }
        Some(PseudoOp::Blkw) => {
            let (value, next) = take_number(operands, 0, line)?;
            take_end(operands, next, line)?;
            let count = u16::try_from(value).map_err(|_| ParseError::ValueOutOfRange { line })?;
            Ok(LineBody::Statement(Operation::Blkw { count }))
        }
        Some(PseudoOp::Stringz) => {
            let text = match operands.first().map(|token| &token.kind) {
                Some(TokenKind::Str(text)) => text.clone(),
                _ => {
                    return Err(ParseError::ExpectedOperand {
                        line,
                        expected: "a string literal",
                    });
                }
            };
            take_end(operands, 1, line)?;
            Ok(LineBody::Statement(Operation::Stringz { text }))
        }
        None => Err(ParseError::UnknownDirective {
            line,
            directive: directive.to_string(),
        }),
    }
}

fn parse_operation<'a>(
    mnemonic: Mnemonic,
    word: &str,
    operands: &[Token<'a>],
    line: usize,
) -> Result<LineBody<'a>, ParseError> {
    let operation = match mnemonic {
        Mnemonic::Branch(cond) => {
            let (target, next) = take_target(operands, 0, line)?;
            take_end(operands, next, line)?;
            Operation::Branch { cond, target }
        }
        Mnemonic::Trap(trap) => {
            take_end(operands, 0, line)?;
            Operation::Trap {
                vector: i32::from(trap.code()),
            }
        }
        Mnemonic::Op(opcode) => parse_machine(opcode, word, operands, line)?,
    };
    Ok(LineBody::Statement(operation))
}

fn parse_machine<'a>(
    opcode: Opcode,
    word: &str,
    operands: &[Token<'a>],
    line: usize,
) -> Result<Operation<'a>, ParseError> {
    Ok(match opcode {
        Opcode::Add | Opcode::And => {
            let (dr, next) = take_register(operands, 0, line)?;
            let next = take_comma(operands, next, line)?;
            let (sr1, next) = take_register(operands, next, line)?;
            let next = take_comma(operands, next, line)?;
            match operands.get(next).map(|token| &token.kind) {
                Some(&TokenKind::Word(register)) if parse_register(register).is_some() => {
                    let (sr2, next) = take_register(operands, next, line)?;
                    take_end(operands, next, line)?;
                    Operation::AluReg {
                        op: opcode,
                        dr,
                        sr1,
                        sr2,
                    }
                }
                Some(&TokenKind::Number(imm)) => {
                    take_end(operands, next + 1, line)?;
                    Operation::AluImm {
                        op: opcode,
                        dr,
                        sr1,
                        imm,
                    }
                }
                _ => {
                    return Err(ParseError::ExpectedOperand {
                        line,
                        expected: "a register or immediate value",
                    });
                }
            }
        }
        Opcode::Not => {
            let (dr, next) = take_register(operands, 0, line)?;
            let next = take_comma(operands, next, line)?;
            let (sr, next) = take_register(operands, next, line)?;
            take_end(operands, next, line)?;
            Operation::Not { dr, sr }
        }
        Opcode::Ld | Opcode::Ldi | Opcode::Lea | Opcode::St | Opcode::Sti => {
            let (reg, next) = take_register(operands, 0, line)?;
            let next = take_comma(operands, next, line)?;
            let (target, next) = take_target(operands, next, line)?;
            take_end(operands, next, line)?;
            Operation::PcRelative {
                op: opcode,
                reg,
                target,
            }
        }
        Opcode::Ldr | Opcode::Str => {
            let (reg, next) = take_register(operands, 0, line)?;
            let next = take_comma(operands, next, line)?;
            let (base, next) = take_register(operands, next, line)?;
            let next = take_comma(operands, next, line)?;
            let (offset, next) = take_number(operands, next, line)?;
            take_end(operands, next, line)?;
            Operation::BaseOffset {
                op: opcode,
                reg,
                base,
                offset,
            }
        }
        Opcode::Jsr => {
            if word.eq_ignore_ascii_case("JSRR") {
                let (base, next) = take_register(operands, 0, line)?;
                take_end(operands, next, line)?;
                Operation::BaseReg {
                    op: Opcode::Jsr,
                    base,
                }
            } else {
                let (target, next) = take_target(operands, 0, line)?;
                take_end(operands, next, line)?;
                Operation::Jsr { target }
            }
        }
        Opcode::Jmp => {
            if word.eq_ignore_ascii_case("RET") {
                take_end(operands, 0, line)?;
                Operation::BaseReg {
                    op: Opcode::Jmp,
                    base: 7,
                }
            } else {
                let (base, next) = take_register(operands, 0, line)?;
                take_end(operands, next, line)?;
                Operation::BaseReg {
                    op: Opcode::Jmp,
                    base,
                }
            }
        }
        Opcode::Trap => {
            let (vector, next) = take_number(operands, 0, line)?;
            take_end(operands, next, line)?;
            Operation::Trap { vector }
        }
        Opcode::Rti => {
            take_end(operands, 0, line)?;
            Operation::Rti
        }
        Opcode::Br | Opcode::Res => {
            return Err(ParseError::UnknownMnemonic {
                line,
                mnemonic: word.to_string(),
            });
        }
    })
}

fn take_register(
    operands: &[Token<'_>],
    index: usize,
    line: usize,
) -> Result<(u16, usize), ParseError> {
    match operands.get(index).map(|token| &token.kind) {
        Some(&TokenKind::Word(word)) => parse_register(word)
            .map(|register| (register, index + 1))
            .ok_or(ParseError::ExpectedOperand {
                line,
                expected: "a register",
            }),
        _ => Err(ParseError::ExpectedOperand {
            line,
            expected: "a register",
        }),
    }
}

fn take_comma(operands: &[Token<'_>], index: usize, line: usize) -> Result<usize, ParseError> {
    match operands.get(index).map(|token| &token.kind) {
        Some(TokenKind::Comma) => Ok(index + 1),
        _ => Err(ParseError::ExpectedOperand {
            line,
            expected: "','",
        }),
    }
}

fn take_number(
    operands: &[Token<'_>],
    index: usize,
    line: usize,
) -> Result<(i32, usize), ParseError> {
    match operands.get(index).map(|token| &token.kind) {
        Some(&TokenKind::Number(value)) => Ok((value, index + 1)),
        _ => Err(ParseError::ExpectedOperand {
            line,
            expected: "a number",
        }),
    }
}

fn take_target<'a>(
    operands: &[Token<'a>],
    index: usize,
    line: usize,
) -> Result<(Target<'a>, usize), ParseError> {
    match operands.get(index).map(|token| &token.kind) {
        Some(&TokenKind::Word(label)) => Ok((Target::Label(label), index + 1)),
        Some(&TokenKind::Number(offset)) => Ok((Target::Offset(offset), index + 1)),
        _ => Err(ParseError::ExpectedOperand {
            line,
            expected: "a label or offset",
        }),
    }
}

fn take_end(operands: &[Token<'_>], index: usize, line: usize) -> Result<(), ParseError> {
    if index >= operands.len() {
        Ok(())
    } else {
        Err(ParseError::UnexpectedToken { line })
    }
}

#[cfg(test)]
mod tests {
    use lc3core::Opcode;

    use super::{Fill, Operation, ParseError, Target, parse};

    /// Parses a single instruction wrapped in a minimal segment and returns its
    /// operation. The source is leaked so the borrowed operation can outlive it,
    /// which is harmless in a test where the process exits promptly.
    fn one(instruction: &str) -> Operation<'static> {
        let source: &'static str =
            Box::leak(format!(".ORIG x3000\n{instruction}\n.END").into_boxed_str());
        parse(source).expect("instruction parses").segments[0].statements[0]
            .operation
            .clone()
    }

    #[test]
    fn symbol_addresses_follow_the_location_counter() {
        let program = parse(
            ".ORIG x3000\n\
             A ADD R0, R0, #0\n\
             B .BLKW #3\n\
             C .STRINGZ \"hi\"\n\
             D .FILL x1234\n\
             E ADD R0, R0, #0\n\
             .END",
        )
        .expect("parses");
        assert_eq!(program.symbols["A"], 0x3000);
        assert_eq!(program.symbols["B"], 0x3001); // after the ADD (1 word)
        assert_eq!(program.symbols["C"], 0x3004); // after .BLKW #3 (3 words)
        assert_eq!(program.symbols["D"], 0x3007); // after "hi" (h, i, NUL = 3 words)
        assert_eq!(program.symbols["E"], 0x3008); // after .FILL (1 word)
    }

    #[test]
    fn multiple_segments_share_one_symbol_table() {
        let program = parse(
            ".ORIG x3000\n\
             START ADD R0, R0, #0\n\
             .END\n\
             .ORIG x6000\n\
             DATA .FILL x1\n\
             .END",
        )
        .expect("parses");
        assert_eq!(program.segments.len(), 2);
        assert_eq!(program.segments[0].origin, 0x3000);
        assert_eq!(program.segments[1].origin, 0x6000);
        assert_eq!(program.symbols["START"], 0x3000);
        assert_eq!(program.symbols["DATA"], 0x6000);
    }

    #[test]
    fn a_duplicate_label_is_rejected() {
        let error = parse(".ORIG x3000\nL ADD R0, R0, #0\nL ADD R0, R0, #0\n.END")
            .expect_err("duplicate label");
        assert_eq!(
            error,
            ParseError::DuplicateLabel {
                line: 3,
                label: "L".to_string()
            }
        );
    }

    #[test]
    fn a_statement_without_orig_is_rejected() {
        assert_eq!(
            parse("ADD R0, R0, #0"),
            Err(ParseError::StatementOutsideSegment { line: 1 })
        );
    }

    #[test]
    fn an_unclosed_segment_is_rejected() {
        assert_eq!(
            parse(".ORIG x3000\nADD R0, R0, #0"),
            Err(ParseError::UnterminatedSegment { line: 1 })
        );
    }

    #[test]
    fn a_label_on_orig_is_rejected() {
        assert_eq!(
            parse("FOO .ORIG x3000\n.END"),
            Err(ParseError::LabelOnDirective { line: 1 })
        );
    }

    #[test]
    fn an_origin_that_does_not_fit_a_word_is_rejected() {
        assert_eq!(
            parse(".ORIG x10000\n.END"),
            Err(ParseError::ValueOutOfRange { line: 1 })
        );
    }

    #[test]
    fn an_unknown_mnemonic_after_a_label_is_rejected() {
        assert_eq!(
            parse(".ORIG x3000\nL FOO R0\n.END"),
            Err(ParseError::UnknownMnemonic {
                line: 2,
                mnemonic: "FOO".to_string()
            })
        );
    }

    #[test]
    fn a_missing_operand_is_rejected() {
        assert!(matches!(
            parse(".ORIG x3000\nADD R1, R2\n.END"),
            Err(ParseError::ExpectedOperand { line: 2, .. })
        ));
    }

    #[test]
    fn alu_instructions_bind_register_and_immediate_forms() {
        assert_eq!(
            one("ADD R1, R2, R3"),
            Operation::AluReg {
                op: Opcode::Add,
                dr: 1,
                sr1: 2,
                sr2: 3
            }
        );
        assert_eq!(
            one("AND R0, R0, #-16"),
            Operation::AluImm {
                op: Opcode::And,
                dr: 0,
                sr1: 0,
                imm: -16
            }
        );
    }

    #[test]
    fn branches_bind_condition_bits_and_label_or_offset_targets() {
        // N = bit 2, Z = bit 1, P = bit 0.
        assert_eq!(
            one("BRnz LOOP"),
            Operation::Branch {
                cond: 0b110,
                target: Target::Label("LOOP")
            }
        );
        assert_eq!(
            one("BRp #-6"),
            Operation::Branch {
                cond: 0b001,
                target: Target::Offset(-6)
            }
        );
        assert_eq!(
            one("BR DONE"),
            Operation::Branch {
                cond: 0b111,
                target: Target::Label("DONE")
            }
        );
    }

    #[test]
    fn the_jump_family_binds_to_its_distinct_forms() {
        assert_eq!(
            one("JMP R2"),
            Operation::BaseReg {
                op: Opcode::Jmp,
                base: 2
            }
        );
        assert_eq!(
            one("RET"),
            Operation::BaseReg {
                op: Opcode::Jmp,
                base: 7
            }
        );
        assert_eq!(
            one("JSRR R3"),
            Operation::BaseReg {
                op: Opcode::Jsr,
                base: 3
            }
        );
        assert_eq!(
            one("JSR SUB"),
            Operation::Jsr {
                target: Target::Label("SUB")
            }
        );
    }

    #[test]
    fn traps_bind_aliases_and_explicit_vectors() {
        assert_eq!(one("HALT"), Operation::Trap { vector: 0x25 });
        assert_eq!(one("GETC"), Operation::Trap { vector: 0x20 });
        assert_eq!(one("TRAP x21"), Operation::Trap { vector: 0x21 });
    }

    #[test]
    fn loads_and_stores_bind_their_operand_shapes() {
        assert_eq!(
            one("LD R0, DATA"),
            Operation::PcRelative {
                op: Opcode::Ld,
                reg: 0,
                target: Target::Label("DATA")
            }
        );
        assert_eq!(
            one("LDR R1, R2, #-1"),
            Operation::BaseOffset {
                op: Opcode::Ldr,
                reg: 1,
                base: 2,
                offset: -1
            }
        );
        assert_eq!(
            one("LEA R0, MSG"),
            Operation::PcRelative {
                op: Opcode::Lea,
                reg: 0,
                target: Target::Label("MSG")
            }
        );
    }

    #[test]
    fn pseudo_ops_and_no_operand_instructions_bind() {
        assert_eq!(one("NOT R1, R2"), Operation::Not { dr: 1, sr: 2 });
        assert_eq!(one("RTI"), Operation::Rti);
        assert_eq!(one(".FILL x1234"), Operation::Fill(Fill::Number(0x1234)));
        assert_eq!(one(".FILL THING"), Operation::Fill(Fill::Label("THING")));
        assert_eq!(one(".BLKW #4"), Operation::Blkw { count: 4 });
        assert_eq!(
            one(".STRINGZ \"hi\""),
            Operation::Stringz {
                text: "hi".to_string()
            }
        );
    }

    #[test]
    fn the_bootstrap_example_builds_its_segments_and_symbol_table() {
        let source = include_str!("../../examples/bootstrap.asm");
        let program = parse(source).expect("bootstrap parses");
        let origins: Vec<u16> = program
            .segments
            .iter()
            .map(|segment| segment.origin)
            .collect();
        assert_eq!(origins, vec![0x6000, 0x6800, 0x3000]);
        // STACK and TABLE are the only labels; they follow the first code word.
        assert_eq!(program.symbols.len(), 2);
        assert_eq!(program.symbols["STACK"], 0x3001);
        assert_eq!(program.symbols["TABLE"], 0x3002);
    }
}

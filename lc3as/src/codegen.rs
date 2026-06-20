//! Code generation and object emission: the assembler's second pass.
//!
//! [`assemble`] runs the first pass ([`parse`]) and then encodes every located
//! statement into LC-3 machine words. Label references are resolved against the
//! program's symbol table to PC-relative offsets---measured from the address
//! following the instruction, the value the program counter holds when the
//! offset is applied---and every operand is range-checked to its field width
//! through the shared [`lc3core`] field helpers. Each `.ORIG` segment becomes
//! one standard single-origin [`ObjectFile`]; together the blocks form the
//! assembled [`Image`].

use std::collections::BTreeMap;

use lc3core::{ObjectFile, Opcode, signed_field, unsigned_field};

use crate::{AsmError, Fill, Operation, Segment, Statement, Target, parse};

/// An assembled program: one located object block per source segment.
///
/// LC-3 source may open several `.ORIG`/`.END` segments at different origins.
/// Each becomes a standard single-origin [`ObjectFile`], and the program as a
/// whole is this ordered collection of blocks, which a loader places one by one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    /// The object blocks, one per `.ORIG` segment, in source order.
    pub blocks: Vec<ObjectFile>,
}

/// Assembles LC-3 `source` into an [`Image`].
///
/// Runs both passes: [`parse`] lays out addresses and the symbol table, then
/// each statement is encoded into machine words with its label references
/// resolved and its operand fields range-checked.
///
/// Returns an [`AsmError`] for a first-pass fault, an undefined label, or a
/// value too large for its field.
pub fn assemble(source: &str) -> Result<Image, AsmError> {
    let program = parse(source)?;
    program
        .segments
        .iter()
        .map(|segment| encode_segment(segment, &program.symbols))
        .collect::<Result<Vec<_>, _>>()
        .map(|blocks| Image { blocks })
}

/// The symbol table: every label mapped to its address.
type Symbols<'a> = BTreeMap<&'a str, u16>;

fn encode_segment(segment: &Segment, symbols: &Symbols) -> Result<ObjectFile, AsmError> {
    let words = segment.statements.iter().try_fold(
        Vec::new(),
        |mut words, statement| -> Result<Vec<u16>, AsmError> {
            emit(statement, symbols, &mut words)?;
            Ok(words)
        },
    )?;
    Ok(ObjectFile {
        origin: segment.origin,
        words,
    })
}

fn emit(statement: &Statement, symbols: &Symbols, out: &mut Vec<u16>) -> Result<(), AsmError> {
    let Statement {
        address,
        line,
        operation,
    } = statement;
    let (address, line) = (*address, *line);
    match operation {
        Operation::AluReg { op, dr, sr1, sr2 } => {
            out.push((op.nibble() << 12) | (dr << 9) | (sr1 << 6) | sr2);
        }
        Operation::AluImm { op, dr, sr1, imm } => {
            let imm5 =
                signed_field(*imm, 5).ok_or(AsmError::ImmediateOutOfRange { line, value: *imm })?;
            out.push((op.nibble() << 12) | (dr << 9) | (sr1 << 6) | (1 << 5) | imm5);
        }
        Operation::Not { dr, sr } => {
            out.push((Opcode::Not.nibble() << 12) | (dr << 9) | (sr << 6) | 0x3F);
        }
        Operation::Branch { cond, target } => {
            let offset = pc_offset(target, address, 9, line, symbols)?;
            out.push((Opcode::Br.nibble() << 12) | (cond << 9) | offset);
        }
        Operation::BaseReg { op, base } => {
            out.push((op.nibble() << 12) | (base << 6));
        }
        Operation::Jsr { target } => {
            let offset = pc_offset(target, address, 11, line, symbols)?;
            out.push((Opcode::Jsr.nibble() << 12) | (1 << 11) | offset);
        }
        Operation::PcRelative { op, reg, target } => {
            let offset = pc_offset(target, address, 9, line, symbols)?;
            out.push((op.nibble() << 12) | (reg << 9) | offset);
        }
        Operation::BaseOffset {
            op,
            reg,
            base,
            offset,
        } => {
            let offset6 = signed_field(*offset, 6).ok_or(AsmError::OffsetOutOfRange {
                line,
                offset: *offset,
                bits: 6,
            })?;
            out.push((op.nibble() << 12) | (reg << 9) | (base << 6) | offset6);
        }
        Operation::Trap { vector } => {
            let trapvect8 = unsigned_field(*vector, 8).ok_or(AsmError::ValueOutOfRange {
                line,
                value: *vector,
                bits: 8,
            })?;
            out.push((Opcode::Trap.nibble() << 12) | trapvect8);
        }
        Operation::Rti => out.push(Opcode::Rti.nibble() << 12),
        Operation::Fill(fill) => out.push(fill_word(fill, line, symbols)?),
        Operation::Blkw { count } => out.extend(std::iter::repeat_n(0u16, usize::from(*count))),
        Operation::Stringz { text } => {
            text.chars()
                .try_for_each(|ch| string_word(ch, line).map(|word| out.push(word)))?;
            out.push(0);
        }
    }
    Ok(())
}

/// Resolves a PC-relative `target` to its `bits`-wide offset field.
///
/// A [`Target::Offset`] is the field value as written; a [`Target::Label`] is
/// the distance from the address after the instruction (`address + 1`, the
/// program counter at the time of the offset) to the label. The result is
/// range-checked to the signed field width.
fn pc_offset(
    target: &Target,
    address: u16,
    bits: u32,
    line: usize,
    symbols: &Symbols,
) -> Result<u16, AsmError> {
    let offset = match target {
        Target::Offset(value) => *value,
        Target::Label(name) => {
            let destination = symbols.get(name).ok_or_else(|| AsmError::UndefinedLabel {
                line,
                label: (*name).to_string(),
            })?;
            i32::from(*destination) - i32::from(address) - 1
        }
    };
    signed_field(offset, bits).ok_or(AsmError::OffsetOutOfRange { line, offset, bits })
}

/// Encodes a `.FILL` operand: a literal word or the address of a label.
fn fill_word(fill: &Fill, line: usize, symbols: &Symbols) -> Result<u16, AsmError> {
    match fill {
        Fill::Number(value) => word(*value).ok_or(AsmError::ValueOutOfRange {
            line,
            value: *value,
            bits: 16,
        }),
        Fill::Label(name) => symbols
            .get(name)
            .copied()
            .ok_or_else(|| AsmError::UndefinedLabel {
                line,
                label: (*name).to_string(),
            }),
    }
}

/// Narrows a `.FILL` value to a 16-bit word, accepting either a signed
/// (`-32768..=32767`) or an unsigned (`0..=65535`) writing of the bit pattern.
fn word(value: i32) -> Option<u16> {
    u16::try_from(value)
        .ok()
        .or_else(|| i16::try_from(value).ok().map(|signed| signed as u16))
}

/// Encodes one `.STRINGZ` character as a word, rejecting any scalar wider than
/// the 16-bit cell.
fn string_word(ch: char, line: usize) -> Result<u16, AsmError> {
    u16::try_from(u32::from(ch)).map_err(|_| AsmError::ValueOutOfRange {
        line,
        value: u32::from(ch) as i32,
        bits: 16,
    })
}

#[cfg(test)]
mod tests {
    use super::{AsmError, assemble};

    /// Assembles a single segment at `x3000` and returns its encoded words.
    fn segment_words(body: &str) -> Vec<u16> {
        let source = format!(".ORIG x3000\n{body}\n.END");
        let mut image = assemble(&source).expect("body assembles");
        image.blocks.remove(0).words
    }

    #[test]
    fn alu_register_and_immediate_forms_encode() {
        assert_eq!(
            segment_words("ADD R1, R2, R3\nADD R0, R0, #-1\nAND R4, R5, R6\nAND R7, R7, #15"),
            vec![0x1283, 0x103F, 0x5946, 0x5FEF]
        );
    }

    #[test]
    fn not_and_rti_encode() {
        assert_eq!(segment_words("NOT R1, R2\nRTI"), vec![0x92BF, 0x8000]);
    }

    #[test]
    fn the_jump_subroutine_and_trap_forms_encode() {
        assert_eq!(
            segment_words("JMP R2\nRET\nJSRR R3\nTRAP x21\nHALT"),
            vec![0xC080, 0xC1C0, 0x40C0, 0xF021, 0xF025]
        );
    }

    #[test]
    fn branches_and_jsr_resolve_offsets_relative_to_the_next_instruction() {
        // A forward branch reaches a later label with a positive offset.
        assert_eq!(
            segment_words("BRnzp SKIP\nADD R0, R0, #0\nSKIP HALT"),
            vec![0x0E01, 0x1020, 0xF025]
        );
        // A backward branch and a JSR reach an earlier label with negative ones.
        assert_eq!(
            segment_words("LOOP ADD R0, R0, #-1\nBRp LOOP\nJSR LOOP"),
            vec![0x103F, 0x03FE, 0x4FFD]
        );
    }

    #[test]
    fn loads_and_stores_encode_their_addressing_modes() {
        assert_eq!(
            segment_words(
                "LEA R0, DATA\nLD R1, DATA\nLDR R2, R3, #-4\nSTR R4, R5, #7\nDATA .FILL x00FF"
            ),
            vec![0xE003, 0x2202, 0x64FC, 0x7947, 0x00FF]
        );
    }

    #[test]
    fn data_pseudo_ops_emit_their_words() {
        // .FILL by label address and by value, a zero-filled block, and a
        // null-terminated string (one word per character plus the terminator).
        assert_eq!(
            segment_words("HERE .FILL HERE\n.FILL #-1\n.BLKW #3\n.STRINGZ \"Hi\""),
            vec![
                0x3000, 0xFFFF, 0x0000, 0x0000, 0x0000, 0x0048, 0x0069, 0x0000
            ]
        );
    }

    #[test]
    fn an_undefined_label_is_rejected() {
        assert_eq!(
            assemble(".ORIG x3000\nLD R0, MISSING\n.END"),
            Err(AsmError::UndefinedLabel {
                line: 2,
                label: "MISSING".to_string()
            })
        );
    }

    #[test]
    fn an_out_of_range_offset_is_rejected() {
        // FAR sits 300 words past the branch, beyond the 9-bit PC-relative reach.
        assert_eq!(
            assemble(".ORIG x3000\nBR FAR\n.BLKW #300\nFAR HALT\n.END"),
            Err(AsmError::OffsetOutOfRange {
                line: 2,
                offset: 300,
                bits: 9
            })
        );
    }

    #[test]
    fn an_out_of_range_immediate_is_rejected() {
        assert_eq!(
            assemble(".ORIG x3000\nADD R0, R0, #16\n.END"),
            Err(AsmError::ImmediateOutOfRange { line: 2, value: 16 })
        );
    }

    #[test]
    fn an_out_of_range_fill_value_is_rejected() {
        assert_eq!(
            assemble(".ORIG x3000\n.FILL #70000\n.END"),
            Err(AsmError::ValueOutOfRange {
                line: 2,
                value: 70000,
                bits: 16
            })
        );
    }

    #[test]
    fn each_segment_becomes_its_own_object_block() {
        let image =
            assemble(".ORIG x3000\nHALT\n.END\n.ORIG x4000\n.FILL x1234\n.END").expect("assembles");
        assert_eq!(image.blocks.len(), 2);
        assert_eq!(image.blocks[0].origin, 0x3000);
        assert_eq!(image.blocks[0].words, vec![0xF025]);
        assert_eq!(image.blocks[1].origin, 0x4000);
        assert_eq!(image.blocks[1].words, vec![0x1234]);
    }

    #[test]
    fn the_bootstrap_example_assembles_to_one_block_per_segment() {
        let image =
            assemble(include_str!("../../examples/bootstrap.asm")).expect("bootstrap assembles");
        let origins: Vec<u16> = image.blocks.iter().map(|block| block.origin).collect();
        assert_eq!(origins, vec![0x6000, 0x6800, 0x3000]);
        // The first segment is the jump table; its words are exactly the .fill
        // addresses listed in the source, so encoding and ordering are checked.
        let table = &image.blocks[0].words;
        assert_eq!(table.first(), Some(&0x3076));
        assert_eq!(table.last(), Some(&0x33ff));
    }
}

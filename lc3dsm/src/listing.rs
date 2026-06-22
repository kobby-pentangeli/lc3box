//! Listing rendering: a decoded image as re-assemblable annotated assembly.
//!
//! [`disassemble`] is the inverse of the assembler's object emission: it turns a
//! decoded [`ObjectFile`] back into a single artifact that is both a human
//! listing and valid LC-3 assembly. Each word becomes one line---a mnemonic with
//! its operands, or `.FILL` for a word that is not a canonical instruction---and
//! every line carries its address and hex encoding as a trailing `;` comment.
//! Because the comment is whitespace to the assembler and decoding is total,
//! re-assembling the listing reproduces the original image.

use std::collections::BTreeSet;

use lc3core::{ObjectFile, Opcode, TrapVector, branch_mnemonic, register_name};

use crate::{Instruction, decode};

/// Renders a decoded object image as a re-assemblable annotated listing.
///
/// Sweeps the image in address order, decoding each word and rendering it as one
/// assembly line---a mnemonic with its operands, or `.FILL xNNNN` for a word that
/// is not a canonical instruction---framed by `.ORIG`/`.END`. A first pass
/// collects every in-range PC-relative target into a label set; the render pass
/// defines a label at each such address and prints references to it by name,
/// while out-of-range and computed targets keep their numeric offset. Every line
/// carries its address and hex encoding as a trailing `;` comment, whitespace to
/// the assembler, so the listing re-assembles to the original image.
pub fn disassemble(object: &ObjectFile) -> String {
    let labels = collect_labels(object);
    std::iter::once(format!(".ORIG x{:04X}", object.origin))
        .chain(addressed_words(object).map(|(address, word)| render_line(address, word, &labels)))
        .chain(std::iter::once(String::from(".END")))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

/// Renders a single `word` as one line of LC-3 assembly: a mnemonic with its
/// operands, or `.FILL xNNNN` for a word that is not a canonical instruction.
pub fn render_instruction(word: u16) -> String {
    operation_text(decode(word), 0, &BTreeSet::new())
        .unwrap_or_else(|| format!(".FILL x{word:04X}"))
}

/// The in-range PC-relative targets of the image, each the address a recovered
/// label will name.
fn collect_labels(object: &ObjectFile) -> BTreeSet<u16> {
    addressed_words(object)
        .filter_map(|(address, word)| pc_relative_target(decode(word), address))
        .filter(|target| in_range(object, *target))
        .collect()
}

/// Pairs each program word with its load address, the address counter wrapping
/// through the 16-bit space exactly as the loader places the image.
fn addressed_words(object: &ObjectFile) -> impl Iterator<Item = (u16, u16)> + '_ {
    object.words.iter().scan(object.origin, |address, &word| {
        let here = *address;
        *address = address.wrapping_add(1);
        Some((here, word))
    })
}

/// The address a PC-relative instruction references, or `None` for any
/// instruction that does not carry a PC-relative offset.
fn pc_relative_target(instruction: Instruction, address: u16) -> Option<u16> {
    let offset = match instruction {
        Instruction::Branch { offset, .. }
        | Instruction::Jsr { offset }
        | Instruction::PcRelative { offset, .. } => offset,
        _ => return None,
    };
    Some(address.wrapping_add(1).wrapping_add(offset as u16))
}

/// Whether `target` is one of the image's own word addresses, the condition for
/// recovering a label rather than leaving the reference numeric.
fn in_range(object: &ObjectFile, target: u16) -> bool {
    target >= object.origin && usize::from(target - object.origin) < object.words.len()
}

/// Renders one located word: an optional recovered label, the decoded operation
/// or a `.FILL` data word, and the trailing address-and-encoding comment.
fn render_line(address: u16, word: u16, labels: &BTreeSet<u16>) -> String {
    let label = if labels.contains(&address) {
        label_name(address)
    } else {
        String::new()
    };
    let body = operation_text(decode(word), address, labels)
        .unwrap_or_else(|| format!(".FILL x{word:04X}"));
    format!("{label:<8}{body:<24}; x{address:04X} x{word:04X}")
}

/// The synthesized name of a label at `address`.
fn label_name(address: u16) -> String {
    format!("L_{address:04X}")
}

/// Renders a decoded instruction's mnemonic and operands, or `None` when the word
/// is data---and, defensively, when an operand field names nothing, so the caller
/// falls back to `.FILL` rather than emitting a malformed line.
fn operation_text(
    instruction: Instruction,
    address: u16,
    labels: &BTreeSet<u16>,
) -> Option<String> {
    let text = match instruction {
        Instruction::AluReg { op, dr, sr1, sr2 } => format!(
            "{} {}, {}, {}",
            op.mnemonic()?,
            register_name(dr)?,
            register_name(sr1)?,
            register_name(sr2)?
        ),
        Instruction::AluImm { op, dr, sr1, imm } => format!(
            "{} {}, {}, #{imm}",
            op.mnemonic()?,
            register_name(dr)?,
            register_name(sr1)?
        ),
        Instruction::Not { dr, sr } => {
            format!("NOT {}, {}", register_name(dr)?, register_name(sr)?)
        }
        Instruction::Branch { cond, offset } => format!(
            "{} {}",
            branch_mnemonic(cond)?,
            pc_target(address, offset, labels)
        ),
        Instruction::BaseReg { op, base } => match op {
            Opcode::Jmp if base == 7 => "RET".to_string(),
            Opcode::Jmp => format!("JMP {}", register_name(base)?),
            Opcode::Jsr => format!("JSRR {}", register_name(base)?),
            _ => return None,
        },
        Instruction::Jsr { offset } => format!("JSR {}", pc_target(address, offset, labels)),
        Instruction::PcRelative { op, reg, offset } => format!(
            "{} {}, {}",
            op.mnemonic()?,
            register_name(reg)?,
            pc_target(address, offset, labels)
        ),
        Instruction::BaseOffset {
            op,
            reg,
            base,
            offset,
        } => format!(
            "{} {}, {}, #{offset}",
            op.mnemonic()?,
            register_name(reg)?,
            register_name(base)?
        ),
        Instruction::Trap { vector } => match TrapVector::try_from(vector) {
            Ok(trap) => trap.mnemonic().to_string(),
            Err(code) => format!("TRAP x{code:02X}"),
        },
        Instruction::Rti => "RTI".to_string(),
        Instruction::Data(_) => return None,
    };
    Some(text)
}

/// Renders a PC-relative operand: a recovered label when the target is in range,
/// otherwise the numeric offset the assembler writes into the field unchanged.
fn pc_target(address: u16, offset: i16, labels: &BTreeSet<u16>) -> String {
    let target = address.wrapping_add(1).wrapping_add(offset as u16);
    if labels.contains(&target) {
        label_name(target)
    } else {
        format!("#{offset}")
    }
}

#[cfg(test)]
mod tests {
    use lc3core::ObjectFile;

    use super::{disassemble, render_instruction};

    /// The assembly portion of a listing line, without its trailing comment or
    /// the surrounding label and padding whitespace.
    fn body_of(line: &str) -> &str {
        line.split(';').next().unwrap_or("").trim()
    }

    #[test]
    fn render_instruction_renders_one_word_numerically_without_framing() {
        assert_eq!(render_instruction(0x1283), "ADD R1, R2, R3");
        assert_eq!(render_instruction(0x0E01), "BR #1");
        // A non-instruction word falls back to `.FILL`.
        assert_eq!(render_instruction(0xD000), ".FILL xD000");
    }

    #[test]
    fn recovers_in_range_labels_and_references_them_by_name() {
        // BR forward by one reaches the third word, which is in range, so the
        // reference resolves to a label defined on that line.
        let object = ObjectFile {
            origin: 0x3000,
            words: vec![0x0E01, 0xF025, 0x1020],
        };
        let listing = disassemble(&object);
        assert!(listing.contains("BR L_3002"), "{listing}");
        assert!(
            listing
                .lines()
                .any(|line| line.starts_with("L_3002") && body_of(line).ends_with("ADD R0, R0, #0")),
            "{listing}"
        );
    }

    #[test]
    fn out_of_range_pc_relative_targets_stay_numeric() {
        // BR forward by five lands past the two-word image: no label, just the
        // field value the assembler writes back unchanged.
        let object = ObjectFile {
            origin: 0x3000,
            words: vec![0x0E05, 0xF025],
        };
        let listing = disassemble(&object);
        assert!(listing.contains("BR #5"), "{listing}");
        assert!(!listing.contains("L_"), "{listing}");
    }

    #[test]
    fn non_instruction_words_render_as_fill() {
        // The reserved opcode 1101 and a branch with an empty condition field are
        // both data, each preserved verbatim as a `.FILL`.
        let object = ObjectFile {
            origin: 0x3000,
            words: vec![0xD000, 0x00FF],
        };
        let listing = disassemble(&object);
        assert!(listing.contains(".FILL xD000"), "{listing}");
        assert!(listing.contains(".FILL x00FF"), "{listing}");
    }

    #[test]
    fn renders_mnemonics_registers_immediates_and_trap_aliases() {
        let object = ObjectFile {
            origin: 0x3000,
            words: vec![
                0x1283, 0x5FEF, 0x92BF, 0x64FC, 0xF025, 0xF030, 0xC1C0, 0x40C0,
            ],
        };
        let listing = disassemble(&object);
        assert!(listing.contains("ADD R1, R2, R3"), "{listing}");
        assert!(listing.contains("AND R7, R7, #15"), "{listing}");
        assert!(listing.contains("NOT R1, R2"), "{listing}");
        assert!(listing.contains("LDR R2, R3, #-4"), "{listing}");
        // A known vector renders as its alias; an unknown one keeps `TRAP xNN`.
        assert!(
            listing.lines().any(|line| body_of(line) == "HALT"),
            "{listing}"
        );
        assert!(listing.contains("TRAP x30"), "{listing}");
        // JMP R7 is the canonical RET; JSRR keeps its base register.
        assert!(
            listing.lines().any(|line| body_of(line) == "RET"),
            "{listing}"
        );
        assert!(listing.contains("JSRR R3"), "{listing}");
    }

    #[test]
    fn frames_the_listing_and_annotates_each_line_with_address_and_encoding() {
        let object = ObjectFile {
            origin: 0x3000,
            words: vec![0xF025],
        };
        let listing = disassemble(&object);
        let lines: Vec<&str> = listing.lines().collect();
        assert_eq!(lines.first(), Some(&".ORIG x3000"));
        assert_eq!(lines.last(), Some(&".END"));
        assert_eq!(body_of(lines[1]), "HALT");
        assert!(lines[1].trim_end().ends_with("; x3000 xF025"));
    }
}

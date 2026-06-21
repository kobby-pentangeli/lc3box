//! Instruction decoding: the inverse of the assembler's code generation.
//!
//! [`decode`] turns one 16-bit word into a structured [`Instruction`], and
//! [`Instruction::encode`] turns it back. Decoding is total and faithful: every
//! word that is the canonical encoding of an LC-3 instruction decodes to that
//! instruction, and every word that is not---a reserved opcode, a branch with an
//! empty condition field, or any format carrying non-zero reserved bits---decodes
//! to [`Instruction::Data`], the raw word a disassembler renders as `.FILL`.
//! Because `encode` is the exact inverse, `decode(word).encode() == word` holds
//! for every one of the 65536 words.

use lc3core::{Opcode, sign_extend};

/// A decoded LC-3 instruction, or a raw data word that is not one.
///
/// The structured result of [`decode`]. Operands are concrete: registers are
/// three-bit numbers and offsets are sign-extended into their signed values;
/// label recovery from PC-relative offsets is a later, separate step. The
/// register-mode/immediate-mode and `JSR`/`JSRR` splits are distinct variants,
/// while the cosmetic `RET` and branch-suffix refinements are left to rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instruction {
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
        /// The sign-extended five-bit immediate.
        imm: i16,
    },
    /// `NOT DR, SR`.
    Not {
        /// The destination register.
        dr: u16,
        /// The source register.
        sr: u16,
    },
    /// `BR(nzp)`: branch on the condition bits to a PC-relative offset.
    Branch {
        /// The `n`/`z`/`p` condition field, never empty (`000` decodes as data).
        cond: u16,
        /// The sign-extended nine-bit PC-relative offset.
        offset: i16,
    },
    /// `JMP`/`JSRR`/`RET`: jump through a base register (`RET` is `JMP R7`).
    BaseReg {
        /// The operation, [`Opcode::Jmp`] (`JMP`/`RET`) or [`Opcode::Jsr`] (`JSRR`).
        op: Opcode,
        /// The base register holding the destination address.
        base: u16,
    },
    /// `JSR`: jump to a subroutine at an eleven-bit PC-relative offset.
    Jsr {
        /// The sign-extended eleven-bit PC-relative offset.
        offset: i16,
    },
    /// `LD`/`LDI`/`LEA`/`ST`/`STI`: a register and a nine-bit PC-relative offset.
    PcRelative {
        /// The load or store operation.
        op: Opcode,
        /// The destination (load) or source (store) register.
        reg: u16,
        /// The sign-extended nine-bit PC-relative offset.
        offset: i16,
    },
    /// `LDR`/`STR`: a register, a base register, and a six-bit offset.
    BaseOffset {
        /// The load or store operation, [`Opcode::Ldr`] or [`Opcode::Str`].
        op: Opcode,
        /// The destination (load) or source (store) register.
        reg: u16,
        /// The base register.
        base: u16,
        /// The sign-extended six-bit offset.
        offset: i16,
    },
    /// `TRAP`: an eight-bit trap vector.
    Trap {
        /// The trap vector (`0..=255`); rendering names the standard aliases.
        vector: u16,
    },
    /// `RTI`: return from interrupt.
    Rti,
    /// A word that is not a canonical instruction encoding; rendered as `.FILL`.
    Data(u16),
}

impl Instruction {
    /// Re-encodes this instruction into its 16-bit machine word.
    pub fn encode(self) -> u16 {
        match self {
            Self::AluReg { op, dr, sr1, sr2 } => (op.nibble() << 12) | (dr << 9) | (sr1 << 6) | sr2,
            Self::AluImm { op, dr, sr1, imm } => {
                (op.nibble() << 12) | (dr << 9) | (sr1 << 6) | (1 << 5) | ((imm as u16) & 0x1F)
            }
            Self::Not { dr, sr } => (Opcode::Not.nibble() << 12) | (dr << 9) | (sr << 6) | 0x3F,
            Self::Branch { cond, offset } => {
                (Opcode::Br.nibble() << 12) | (cond << 9) | ((offset as u16) & 0x1FF)
            }
            Self::BaseReg { op, base } => (op.nibble() << 12) | (base << 6),
            Self::Jsr { offset } => {
                (Opcode::Jsr.nibble() << 12) | (1 << 11) | ((offset as u16) & 0x7FF)
            }
            Self::PcRelative { op, reg, offset } => {
                (op.nibble() << 12) | (reg << 9) | ((offset as u16) & 0x1FF)
            }
            Self::BaseOffset {
                op,
                reg,
                base,
                offset,
            } => (op.nibble() << 12) | (reg << 9) | (base << 6) | ((offset as u16) & 0x3F),
            Self::Trap { vector } => (Opcode::Trap.nibble() << 12) | vector,
            Self::Rti => Opcode::Rti.nibble() << 12,
            Self::Data(word) => word,
        }
    }
}

/// Decodes one 16-bit `word` into its [`Instruction`].
pub fn decode(word: u16) -> Instruction {
    match Opcode::decode(word) {
        op @ (Opcode::Add | Opcode::And) => {
            let dr = (word >> 9) & 0x7;
            let sr1 = (word >> 6) & 0x7;
            if word & 0x20 != 0 {
                Instruction::AluImm {
                    op,
                    dr,
                    sr1,
                    imm: sign_extend(word & 0x1F, 5) as i16,
                }
            } else if word & 0x18 == 0 {
                // Register mode: bits [4:3] are reserved and must be zero.
                Instruction::AluReg {
                    op,
                    dr,
                    sr1,
                    sr2: word & 0x7,
                }
            } else {
                Instruction::Data(word)
            }
        }
        Opcode::Br => {
            let cond = (word >> 9) & 0x7;
            if cond != 0 {
                Instruction::Branch {
                    cond,
                    offset: sign_extend(word & 0x1FF, 9) as i16,
                }
            } else {
                Instruction::Data(word)
            }
        }
        Opcode::Jsr => {
            if word & 0x0800 != 0 {
                Instruction::Jsr {
                    offset: sign_extend(word & 0x7FF, 11) as i16,
                }
            } else if word & 0x063F == 0 {
                // JSRR: bits [10:9] and [5:0] are reserved and must be zero.
                Instruction::BaseReg {
                    op: Opcode::Jsr,
                    base: (word >> 6) & 0x7,
                }
            } else {
                Instruction::Data(word)
            }
        }
        // NOT requires the trailing ones; JMP, RTI, and TRAP their zeroed
        // reserved fields; a failed guard falls through to the data arm.
        Opcode::Not if word & 0x3F == 0x3F => Instruction::Not {
            dr: (word >> 9) & 0x7,
            sr: (word >> 6) & 0x7,
        },
        Opcode::Jmp if word & 0x0E3F == 0 => Instruction::BaseReg {
            op: Opcode::Jmp,
            base: (word >> 6) & 0x7,
        },
        Opcode::Trap if word & 0x0F00 == 0 => Instruction::Trap {
            vector: word & 0xFF,
        },
        Opcode::Rti if word & 0x0FFF == 0 => Instruction::Rti,
        op @ (Opcode::Ld | Opcode::Ldi | Opcode::Lea | Opcode::St | Opcode::Sti) => {
            Instruction::PcRelative {
                op,
                reg: (word >> 9) & 0x7,
                offset: sign_extend(word & 0x1FF, 9) as i16,
            }
        }
        op @ (Opcode::Ldr | Opcode::Str) => Instruction::BaseOffset {
            op,
            reg: (word >> 9) & 0x7,
            base: (word >> 6) & 0x7,
            offset: sign_extend(word & 0x3F, 6) as i16,
        },
        _ => Instruction::Data(word),
    }
}

#[cfg(test)]
mod tests {
    use lc3core::Opcode;

    use super::{Instruction, decode};

    #[test]
    fn every_word_decodes_and_encodes() {
        for word in 0..=u16::MAX {
            assert_eq!(
                decode(word).encode(),
                word,
                "{word:#06x} did not round-trip"
            );
        }
    }

    #[test]
    fn each_instruction_form_decodes_to_its_operands() {
        assert_eq!(
            decode(0x1283),
            Instruction::AluReg {
                op: Opcode::Add,
                dr: 1,
                sr1: 2,
                sr2: 3
            }
        );
        assert_eq!(
            decode(0x103F),
            Instruction::AluImm {
                op: Opcode::Add,
                dr: 0,
                sr1: 0,
                imm: -1
            }
        );
        assert_eq!(
            decode(0x5FEF),
            Instruction::AluImm {
                op: Opcode::And,
                dr: 7,
                sr1: 7,
                imm: 15
            }
        );
        assert_eq!(decode(0x92BF), Instruction::Not { dr: 1, sr: 2 });
        assert_eq!(
            decode(0x0E01),
            Instruction::Branch {
                cond: 0b111,
                offset: 1
            }
        );
        assert_eq!(
            decode(0x03FE),
            Instruction::Branch {
                cond: 0b001,
                offset: -2
            }
        );
        assert_eq!(
            decode(0xC080),
            Instruction::BaseReg {
                op: Opcode::Jmp,
                base: 2
            }
        );
        assert_eq!(
            decode(0xC1C0),
            Instruction::BaseReg {
                op: Opcode::Jmp,
                base: 7
            }
        );
        assert_eq!(
            decode(0x40C0),
            Instruction::BaseReg {
                op: Opcode::Jsr,
                base: 3
            }
        );
        assert_eq!(decode(0x4FFD), Instruction::Jsr { offset: -3 });
        assert_eq!(
            decode(0xE003),
            Instruction::PcRelative {
                op: Opcode::Lea,
                reg: 0,
                offset: 3
            }
        );
        assert_eq!(
            decode(0x64FC),
            Instruction::BaseOffset {
                op: Opcode::Ldr,
                reg: 2,
                base: 3,
                offset: -4
            }
        );
        assert_eq!(decode(0xF025), Instruction::Trap { vector: 0x25 });
        // A vector with no standard alias is still a canonical TRAP, not data.
        assert_eq!(decode(0xF030), Instruction::Trap { vector: 0x30 });
        assert_eq!(decode(0x8000), Instruction::Rti);
    }

    #[test]
    fn some_words_decode_as_data() {
        // The reserved opcode, an empty-condition branch, both ALU register-mode
        // forms with non-zero [4:3], NOT without its trailing ones, and JMP,
        // JSRR, RTI, and TRAP carrying non-zero reserved bits.
        for word in [
            0xD000, 0x0123, 0x1290, 0x5290, 0x923A, 0xC001, 0x4001, 0x8001, 0xF101,
        ] {
            assert_eq!(decode(word), Instruction::Data(word));
        }
    }
}

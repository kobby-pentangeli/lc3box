//! LC-3 operation codes and instruction decoding.

/// The sixteen LC-3 operation codes.
///
/// Every instruction is one 16-bit word whose top four bits (`[15:12]`) select
/// the operation; the remaining twelve bits carry that operation's operands.
/// All sixteen four-bit values are defined: [`Opcode::Rti`] and [`Opcode::Res`]
/// occupy real slots even though they are privileged or reserved, so decoding
/// the opcode field is total.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    /// Conditional branch — opcode `0000`.
    Br,
    /// Addition — opcode `0001`.
    Add,
    /// Load PC-relative — opcode `0010`.
    Ld,
    /// Store PC-relative — opcode `0011`.
    St,
    /// Jump to subroutine — opcode `0100`.
    Jsr,
    /// Bitwise AND — opcode `0101`.
    And,
    /// Load base+offset — opcode `0110`.
    Ldr,
    /// Store base+offset — opcode `0111`.
    Str,
    /// Return from interrupt (privileged) — opcode `1000`.
    Rti,
    /// Bitwise NOT — opcode `1001`.
    Not,
    /// Load indirect — opcode `1010`.
    Ldi,
    /// Store indirect — opcode `1011`.
    Sti,
    /// Jump (and return) — opcode `1100`.
    Jmp,
    /// Reserved — opcode `1101`.
    Res,
    /// Load effective address — opcode `1110`.
    Lea,
    /// System trap — opcode `1111`.
    Trap,
}

impl Opcode {
    /// Decodes the opcode field (`[15:12]`) of an instruction.
    pub const fn decode(instruction: u16) -> Self {
        match (instruction >> 12) & 0xF {
            0 => Self::Br,
            1 => Self::Add,
            2 => Self::Ld,
            3 => Self::St,
            4 => Self::Jsr,
            5 => Self::And,
            6 => Self::Ldr,
            7 => Self::Str,
            8 => Self::Rti,
            9 => Self::Not,
            10 => Self::Ldi,
            11 => Self::Sti,
            12 => Self::Jmp,
            13 => Self::Res,
            14 => Self::Lea,
            _ => Self::Trap,
        }
    }

    /// The four-bit value this opcode occupies in the instruction field
    /// `[15:12]`. Shifting it into the top nibble is the inverse of [`decode`].
    ///
    /// [`decode`]: Self::decode
    pub const fn nibble(self) -> u16 {
        match self {
            Self::Br => 0,
            Self::Add => 1,
            Self::Ld => 2,
            Self::St => 3,
            Self::Jsr => 4,
            Self::And => 5,
            Self::Ldr => 6,
            Self::Str => 7,
            Self::Rti => 8,
            Self::Not => 9,
            Self::Ldi => 10,
            Self::Sti => 11,
            Self::Jmp => 12,
            Self::Res => 13,
            Self::Lea => 14,
            Self::Trap => 15,
        }
    }

    /// Parses a machine-operation mnemonic, case-insensitively, to its opcode.
    ///
    /// Recognizes the directly-named operations together with the `JSR`/`JSRR`
    /// and `RET` forms, which share the [`Jsr`](Self::Jsr) and [`Jmp`](Self::Jmp)
    /// opcodes. The `BR` family is parsed by
    /// [`parse_branch_condition`](crate::parse_branch_condition) instead, since
    /// those mnemonics also select the condition bits. Returns `None` for any
    /// token that is not a machine-operation mnemonic, including pseudo-ops and
    /// trap aliases.
    pub fn from_mnemonic(token: &str) -> Option<Self> {
        match token.to_ascii_uppercase().as_str() {
            "ADD" => Some(Self::Add),
            "AND" => Some(Self::And),
            "NOT" => Some(Self::Not),
            "LD" => Some(Self::Ld),
            "LDI" => Some(Self::Ldi),
            "LDR" => Some(Self::Ldr),
            "LEA" => Some(Self::Lea),
            "ST" => Some(Self::St),
            "STI" => Some(Self::Sti),
            "STR" => Some(Self::Str),
            "JMP" | "RET" => Some(Self::Jmp),
            "JSR" | "JSRR" => Some(Self::Jsr),
            "RTI" => Some(Self::Rti),
            "TRAP" => Some(Self::Trap),
            _ => None,
        }
    }

    /// The assembly mnemonic naming this operation, or `None` when no
    /// single mnemonic does.
    ///
    /// The inverse of [`from_mnemonic`](Self::from_mnemonic): every opcode that
    /// `from_mnemonic` produces maps back to a mnemonic that re-parses to it. The
    /// operations sharing an opcode render as their base form, [`Jsr`](Self::Jsr)
    /// as `JSR` and [`Jmp`](Self::Jmp) as `JMP`, leaving the `JSRR`/`RET`
    /// refinement to the operand bits. [`Br`](Self::Br) returns `None` because
    /// its mnemonic is fixed by the condition field
    /// ([`branch_mnemonic`](crate::branch_mnemonic)), and [`Res`](Self::Res)
    /// returns `None` because the reserved opcode names no operation; a word with
    /// either is rendered as data rather than as an instruction.
    pub const fn mnemonic(self) -> Option<&'static str> {
        match self {
            Self::Add => Some("ADD"),
            Self::And => Some("AND"),
            Self::Not => Some("NOT"),
            Self::Ld => Some("LD"),
            Self::Ldi => Some("LDI"),
            Self::Ldr => Some("LDR"),
            Self::Lea => Some("LEA"),
            Self::St => Some("ST"),
            Self::Sti => Some("STI"),
            Self::Str => Some("STR"),
            Self::Jmp => Some("JMP"),
            Self::Jsr => Some("JSR"),
            Self::Rti => Some("RTI"),
            Self::Trap => Some("TRAP"),
            Self::Br | Self::Res => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Opcode;

    #[test]
    fn decode_uses_only_the_top_nibble() {
        // The low twelve bits are operands and must not affect the opcode.
        assert_eq!(Opcode::decode(0x1FFF), Opcode::Add);
        assert_eq!(Opcode::decode(0x1000), Opcode::Add);
        assert_eq!(Opcode::decode(0xF025), Opcode::Trap);
        assert_eq!(Opcode::decode(0x0ABC), Opcode::Br);
    }

    #[test]
    fn decode_covers_privileged_and_reserved_slots() {
        assert_eq!(Opcode::decode(0x8000), Opcode::Rti);
        assert_eq!(Opcode::decode(0xD000), Opcode::Res);
    }

    #[test]
    fn decode_maps_every_nibble_to_its_operation() {
        let expected = [
            Opcode::Br,
            Opcode::Add,
            Opcode::Ld,
            Opcode::St,
            Opcode::Jsr,
            Opcode::And,
            Opcode::Ldr,
            Opcode::Str,
            Opcode::Rti,
            Opcode::Not,
            Opcode::Ldi,
            Opcode::Sti,
            Opcode::Jmp,
            Opcode::Res,
            Opcode::Lea,
            Opcode::Trap,
        ];
        for (nibble, op) in expected.into_iter().enumerate() {
            let instruction = u16::try_from(nibble).expect("nibble fits in u16") << 12;
            assert_eq!(Opcode::decode(instruction), op);
        }
    }

    #[test]
    fn nibble_is_the_inverse_of_decode() {
        for nibble in 0..16u16 {
            assert_eq!(Opcode::decode(nibble << 12).nibble(), nibble);
        }
    }

    #[test]
    fn mnemonics_parse_case_insensitively_to_their_opcode() {
        assert_eq!(Opcode::from_mnemonic("add"), Some(Opcode::Add));
        assert_eq!(Opcode::from_mnemonic("LDR"), Some(Opcode::Ldr));
        assert_eq!(Opcode::from_mnemonic("Trap"), Some(Opcode::Trap));
    }

    #[test]
    fn jsr_jsrr_and_ret_collapse_to_their_shared_opcodes() {
        assert_eq!(Opcode::from_mnemonic("JSR"), Some(Opcode::Jsr));
        assert_eq!(Opcode::from_mnemonic("JSRR"), Some(Opcode::Jsr));
        assert_eq!(Opcode::from_mnemonic("RET"), Some(Opcode::Jmp));
    }

    #[test]
    fn non_machine_operation_tokens_are_rejected() {
        // `BR` carries condition bits and is parsed elsewhere; pseudo-ops and
        // trap aliases are not opcodes.
        assert_eq!(Opcode::from_mnemonic("BR"), None);
        assert_eq!(Opcode::from_mnemonic("BRnzp"), None);
        assert_eq!(Opcode::from_mnemonic(".ORIG"), None);
        assert_eq!(Opcode::from_mnemonic("HALT"), None);
        assert_eq!(Opcode::from_mnemonic("R0"), None);
    }

    #[test]
    fn mnemonic_inverts_from_mnemonic() {
        // Every opcode that names an operation re-parses to itself; the two that
        // name none are exactly the condition-fixed branch and the reserved slot.
        let all = [
            Opcode::Br,
            Opcode::Add,
            Opcode::Ld,
            Opcode::St,
            Opcode::Jsr,
            Opcode::And,
            Opcode::Ldr,
            Opcode::Str,
            Opcode::Rti,
            Opcode::Not,
            Opcode::Ldi,
            Opcode::Sti,
            Opcode::Jmp,
            Opcode::Res,
            Opcode::Lea,
            Opcode::Trap,
        ];
        for op in all {
            match op.mnemonic() {
                Some(name) => assert_eq!(Opcode::from_mnemonic(name), Some(op)),
                None => assert!(matches!(op, Opcode::Br | Opcode::Res)),
            }
        }
    }

    #[test]
    fn shared_opcodes_render_their_base_mnemonic() {
        // The base form, so the renderer can recover JSRR and RET from the
        // operand bits rather than from the opcode alone.
        assert_eq!(Opcode::Jsr.mnemonic(), Some("JSR"));
        assert_eq!(Opcode::Jmp.mnemonic(), Some("JMP"));
    }
}

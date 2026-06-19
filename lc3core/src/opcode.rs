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
}

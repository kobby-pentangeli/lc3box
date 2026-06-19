//! LC-3 assembler pseudo-operations.

/// The assembler directives ("pseudo-ops") of the LC-3 assembly language.
///
/// These are not machine instructions: the assembler consumes them at assembly
/// time to place code, reserve storage, and emit data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PseudoOp {
    /// `.ORIG` — set the origin (load address) of the words that follow.
    Orig,
    /// `.FILL` — emit one word holding a given value.
    Fill,
    /// `.BLKW` — reserve a block of consecutive words.
    Blkw,
    /// `.STRINGZ` — emit a null-terminated string, one character per word.
    Stringz,
    /// `.END` — mark the end of the source program.
    End,
}

impl PseudoOp {
    /// The canonical mnemonic, including the leading dot.
    pub const fn mnemonic(self) -> &'static str {
        match self {
            Self::Orig => ".ORIG",
            Self::Fill => ".FILL",
            Self::Blkw => ".BLKW",
            Self::Stringz => ".STRINGZ",
            Self::End => ".END",
        }
    }

    /// Parses a directive mnemonic, case-insensitively and with the leading dot
    /// optional. Returns `None` for any token that is not a pseudo-op.
    pub fn from_mnemonic(token: &str) -> Option<Self> {
        match token.trim_start_matches('.').to_ascii_uppercase().as_str() {
            "ORIG" => Some(Self::Orig),
            "FILL" => Some(Self::Fill),
            "BLKW" => Some(Self::Blkw),
            "STRINGZ" => Some(Self::Stringz),
            "END" => Some(Self::End),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PseudoOp;

    #[test]
    fn parsing_is_case_insensitive_and_dot_optional() {
        assert_eq!(PseudoOp::from_mnemonic(".orig"), Some(PseudoOp::Orig));
        assert_eq!(PseudoOp::from_mnemonic("STRINGZ"), Some(PseudoOp::Stringz));
        assert_eq!(PseudoOp::from_mnemonic(".End"), Some(PseudoOp::End));
        assert_eq!(PseudoOp::from_mnemonic("ADD"), None);
    }

    #[test]
    fn every_mnemonic_parses_back_to_its_variant() {
        for op in [
            PseudoOp::Orig,
            PseudoOp::Fill,
            PseudoOp::Blkw,
            PseudoOp::Stringz,
            PseudoOp::End,
        ] {
            assert_eq!(PseudoOp::from_mnemonic(op.mnemonic()), Some(op));
        }
    }
}

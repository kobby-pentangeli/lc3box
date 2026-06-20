//! LC-3 register file shape and condition codes.

/// The number of general-purpose registers (`R0`–`R7`).
///
/// A register is named by a 3-bit field within an instruction, so the index is
/// always in range and need never be validated at runtime.
pub const GP_REGISTER_COUNT: usize = 8;

/// The condition code set after each instruction that writes a register.
///
/// Exactly one of these is set at any time, reflecting the sign of the value
/// most recently written: negative, zero, or positive. The `BR` instruction
/// tests them through the `N`/`Z`/`P` bits of its encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionFlag {
    /// The last result was positive.
    Positive,
    /// The last result was zero.
    Zero,
    /// The last result was negative.
    Negative,
}

impl ConditionFlag {
    /// The single-bit mask matching this flag against a `BR` instruction's
    /// `N`/`Z`/`P` field (`P` = bit 0, `Z` = bit 1, `N` = bit 2).
    pub const fn bits(self) -> u16 {
        match self {
            Self::Positive => 1 << 0,
            Self::Zero => 1 << 1,
            Self::Negative => 1 << 2,
        }
    }

    /// Derives the condition code from a freshly written 16-bit `value`, reading
    /// its most-significant bit as the sign.
    pub const fn from_result(value: u16) -> Self {
        if value == 0 {
            Self::Zero
        } else if value >> 15 != 0 {
            Self::Negative
        } else {
            Self::Positive
        }
    }

    /// Whether a `BR` whose `N`/`Z`/`P` field is `nzp` should be taken given
    /// this current condition code.
    pub const fn matches(self, nzp: u16) -> bool {
        nzp & self.bits() != 0
    }
}

/// Parses a register token `R0`–`R7`, case-insensitively, to its three-bit
/// register number.
///
/// Returns `None` for anything that is not one of the eight general-purpose
/// register names, including `R8` and bare `R`.
pub fn parse_register(token: &str) -> Option<u16> {
    match token.strip_prefix(['R', 'r'])?.as_bytes() {
        [digit @ b'0'..=b'7'] => Some(u16::from(digit - b'0')),
        _ => None,
    }
}

/// Parses a `BR`-family branch mnemonic, case-insensitively, to its three-bit
/// `N`/`Z`/`P` condition field (`N` = bit 2, `Z` = bit 1, `P` = bit 0).
///
/// Bare `BR` and `BRnzp` are the unconditional branch, with all three bits set.
/// The documented condition combinations are accepted; a token that is not a
/// `BR`-family mnemonic, or that carries an unrecognized suffix, yields `None`.
pub fn parse_branch_condition(mnemonic: &str) -> Option<u16> {
    let suffix = mnemonic.to_ascii_uppercase();
    let n = ConditionFlag::Negative.bits();
    let z = ConditionFlag::Zero.bits();
    let p = ConditionFlag::Positive.bits();

    match suffix.strip_prefix("BR")? {
        "" | "NZP" => Some(n | z | p),
        "N" => Some(n),
        "Z" => Some(z),
        "P" => Some(p),
        "NZ" => Some(n | z),
        "NP" => Some(n | p),
        "ZP" => Some(z | p),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{ConditionFlag, parse_branch_condition, parse_register};

    #[test]
    fn register_tokens_parse_case_insensitively() {
        assert_eq!(parse_register("R0"), Some(0));
        assert_eq!(parse_register("r7"), Some(7));
    }

    #[test]
    fn non_register_tokens_are_rejected() {
        // Out-of-range index, missing index, wrong prefix, and multi-digit
        // tokens are all rejected rather than silently clamped or truncated.
        assert_eq!(parse_register("R8"), None);
        assert_eq!(parse_register("R"), None);
        assert_eq!(parse_register("X0"), None);
        assert_eq!(parse_register("R10"), None);
    }

    #[test]
    fn branch_mnemonics_map_to_their_condition_field() {
        // N = bit 2, Z = bit 1, P = bit 0.
        assert_eq!(parse_branch_condition("BRn"), Some(0b100));
        assert_eq!(parse_branch_condition("brz"), Some(0b010));
        assert_eq!(parse_branch_condition("BRp"), Some(0b001));
        assert_eq!(parse_branch_condition("BRzp"), Some(0b011));
    }

    #[test]
    fn bare_and_full_branch_are_unconditional() {
        assert_eq!(parse_branch_condition("BR"), Some(0b111));
        assert_eq!(parse_branch_condition("BRnzp"), Some(0b111));
    }

    #[test]
    fn malformed_branch_mnemonics_are_rejected() {
        // A non-branch mnemonic, and a suffix outside the documented set.
        assert_eq!(parse_branch_condition("ADD"), None);
        assert_eq!(parse_branch_condition("BRpn"), None);
        assert_eq!(parse_branch_condition("BRx"), None);
    }

    #[test]
    fn sign_of_result_selects_the_flag() {
        assert_eq!(ConditionFlag::from_result(0), ConditionFlag::Zero);
        assert_eq!(ConditionFlag::from_result(0x0001), ConditionFlag::Positive);
        assert_eq!(ConditionFlag::from_result(0x7FFF), ConditionFlag::Positive);
        assert_eq!(ConditionFlag::from_result(0x8000), ConditionFlag::Negative);
        assert_eq!(ConditionFlag::from_result(0xFFFF), ConditionFlag::Negative);
    }

    #[test]
    fn branch_is_taken_only_for_the_matching_bit() {
        // N = bit 2, Z = bit 1, P = bit 0.
        assert!(ConditionFlag::Negative.matches(0b100));
        assert!(!ConditionFlag::Negative.matches(0b011));
        assert!(ConditionFlag::Zero.matches(0b010));
        assert!(ConditionFlag::Positive.matches(0b001));
        // An unconditional branch (`nzp` all set) is always taken.
        assert!(ConditionFlag::Positive.matches(0b111));
    }
}

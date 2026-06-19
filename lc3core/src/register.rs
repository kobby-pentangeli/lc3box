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

#[cfg(test)]
mod tests {
    use super::ConditionFlag;

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

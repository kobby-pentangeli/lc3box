//! LC-3 trap vectors.

/// The six standard LC-3 trap service routines, identified by their entry in
/// the trap vector table (`x20`–`x25`).
///
/// A `TRAP` instruction carries an 8-bit trap vector in bits `[7:0]`; these are
/// the vectors a conforming program may invoke for console and keyboard I/O.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrapVector {
    /// `GETC` (`x20`) — read one character from the keyboard into R0, without
    /// echo; the high byte of R0 is cleared.
    Getc = 0x20,
    /// `OUT` (`x21`) — write the character in the low byte of R0 to the console.
    Out = 0x21,
    /// `PUTS` (`x22`) — write the null-terminated string addressed by R0, one
    /// character per word.
    Puts = 0x22,
    /// `IN` (`x23`) — prompt for and read one character into R0, with echo.
    In = 0x23,
    /// `PUTSP` (`x24`) — write a string packed two characters per word, low byte
    /// first, until a null character.
    Putsp = 0x24,
    /// `HALT` (`x25`) — halt execution.
    Halt = 0x25,
}

impl TryFrom<u16> for TrapVector {
    /// The unrecognized vector value.
    type Error = u16;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0x20 => Ok(Self::Getc),
            0x21 => Ok(Self::Out),
            0x22 => Ok(Self::Puts),
            0x23 => Ok(Self::In),
            0x24 => Ok(Self::Putsp),
            0x25 => Ok(Self::Halt),
            other => Err(other),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TrapVector;

    #[test]
    fn known_vectors_decode() {
        assert_eq!(TrapVector::try_from(0x20), Ok(TrapVector::Getc));
        assert_eq!(TrapVector::try_from(0x25), Ok(TrapVector::Halt));
    }

    #[test]
    fn unknown_vector_is_rejected_with_its_value() {
        assert_eq!(TrapVector::try_from(0x26), Err(0x26));
        assert_eq!(TrapVector::try_from(0x00), Err(0x00));
    }
}

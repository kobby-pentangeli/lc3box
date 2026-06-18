use thiserror::Error;

/// An error that halts the virtual machine.
///
/// Execution stops as soon as one of these is produced; the value identifies
/// what the machine could not do, so the caller can report it rather than the
/// machine continuing past an undefined state.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// The privileged `RTI` instruction was reached outside supervisor mode.
    ///
    /// This user-mode VM has no supervisor layer, so `RTI` is never legal here.
    #[error("privileged instruction (RTI) outside supervisor mode: {0:#06x}")]
    PrivilegedInstruction(u16),

    /// A reserved opcode was reached. The LC-3 leaves this encoding undefined.
    #[error("reserved opcode: {0:#06x}")]
    ReservedOpcode(u16),

    /// A `TRAP` named a vector outside the six standard service routines.
    #[error("unknown trap vector: {0:#04x}")]
    UnknownTrap(u16),
}

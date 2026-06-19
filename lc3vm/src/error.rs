use lc3core::ObjectError;
use thiserror::Error;

/// An error raised while loading or running an LC-3 program.
///
/// Loading stops if the object file is malformed or does not fit in memory;
/// execution stops as soon as the machine reaches an instruction it cannot run
/// or a host I/O operation fails. Each value identifies what the machine could
/// not do, so the caller can report it rather than continuing past an undefined
/// state.
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

    /// The object file could not be decoded as a valid `.obj` image.
    #[error("malformed object file: {0}")]
    Object(#[from] ObjectError),

    /// The image does not fit: loaded at `origin`, its `words` words would run
    /// past the top of the address space (`0xFFFF`).
    #[error("program of {words} words at origin {origin:#06x} runs past the end of memory")]
    ProgramOutOfRange {
        /// The load address the image would have started at.
        origin: u16,
        /// The number of words the image holds.
        words: usize,
    },

    /// A host I/O operation---reading the object file or the console---failed.
    #[error("input/output error: {0}")]
    Io(#[from] std::io::Error),
}

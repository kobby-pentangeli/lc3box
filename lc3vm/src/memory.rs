/// The LC-3 has 65536 memory locations,
/// the max addressable by `u16`, 2^16.
pub const MEMORY_SIZE: usize = u16::MAX as usize;

/// Represents the 16-bit addressable memory space of the Lc3VM.
pub struct Memory([u16; MEMORY_SIZE]);

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}

impl Memory {
    /// Creates a new memory with the default [MEMORY_SIZE].
    pub fn new() -> Self {
        Self([0u16; MEMORY_SIZE])
    }

    /// Sets `value` at the specified memory `address`.
    pub(crate) fn write(&mut self, address: usize, value: u16) {
        self.0[address] = value;
    }

    /// Retrieves a value from the specified memory `address`.
    pub(crate) fn read(&mut self, address: usize) -> u16 {
        self.0[address]
    }
}

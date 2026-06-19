use lc3core::MEMORY_SIZE;

/// The VM's 16-bit addressable memory: every word of the `x0000`–`xFFFF` space.
///
/// The backing store is heap-allocated as a fixed `[u16; MEMORY_SIZE]` so that
/// `Lc3VM` stays cheap to move and so that indexing by `usize::from(address)`---
/// a value provably below the array length---lets the compiler prove every
/// access in bounds and drop the bounds check from the hot path.
pub struct Memory(Box<[u16; MEMORY_SIZE]>);

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}

impl Memory {
    /// Creates zero-initialized memory spanning the whole address space.
    pub fn new() -> Self {
        Self(Box::new([0u16; MEMORY_SIZE]))
    }

    /// Reads the word at `address`.
    pub fn read(&self, address: u16) -> u16 {
        self.0[usize::from(address)]
    }

    /// Writes `value` to `address`.
    pub fn write(&mut self, address: u16, value: u16) {
        self.0[usize::from(address)] = value;
    }

    /// Returns a mutable view of the `len` words beginning at `origin`, or
    /// `None` if that range would extend past the end of the address space.
    pub fn region_mut(&mut self, origin: u16, len: usize) -> Option<&mut [u16]> {
        let start = usize::from(origin);
        self.0.get_mut(start..start.checked_add(len)?)
    }
}

#[cfg(test)]
mod tests {
    use super::Memory;

    #[test]
    fn reads_back_what_was_written_across_the_whole_space() {
        let mut memory = Memory::new();

        memory.write(0x0000, 0x1234);
        memory.write(0xFFFF, 0xBEEF);
        assert_eq!(memory.read(0x0000), 0x1234);
        assert_eq!(memory.read(0xFFFF), 0xBEEF);
    }
}

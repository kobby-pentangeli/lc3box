use super::{MEMORY_SIZE, Registers};

/// The main LC-3 emulator.
///
/// # Memory Architecture
/// - 16-bit address space (0x0000-0xFFFF)
/// - First 0xFE00 addresses: general purpose memory
/// - 0xFE00-0xFFFF: Memory-mapped I/O registers
///
/// # Execution Flow
/// 1. Fetch instruction from PC
/// 2. Decode opcode
/// 3. Execute instruction
/// 4. Update condition codes
pub struct VM {
    /// 16-bit addressable memory space
    pub memory: [u16; MEMORY_SIZE],
    /// Processor registers and flags
    pub registers: Registers,
}

impl VM {
    /// Creates a new VM in initial state.
    pub fn new() -> Self {
        Self {
            memory: [0; MEMORY_SIZE],
            registers: Registers::new(),
        }
    }

    /// Loads the program `instruction` into the VM at the given memory `address`.
    pub fn write_memory(&mut self, address: usize, instruction: u16) {
        self.memory[address] = instruction;
    }

    /// Retrieves a program instruction from the specified memory `address`.
    pub fn read_memory(&self, address: u16) -> u16 {
        self.memory[address as usize]
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

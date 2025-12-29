use std::io::Read as _;

use super::{MEMORY_SIZE, Registers};

/// Memory-mapped register.
///
/// Memory-mapped I/O are handled by load/store (LDI/STI, LDR/STR) instructions
/// using memory addresses to designate each I/O device register.
/// Addresses xFE00 through xFFFF have been allocated to represent the addresses of I/O devices.
///
/// LC-3 has two memory-mapped registers that need to be implemented.
/// They are the _keyboard status register_ (KBSR) and _keyboard data register_ (KBDR).
/// The KBSR indicates whether a key has been pressed, and the KBDR identifies which key was pressed.
pub enum MMappedReg {
    /// Indicates whether a key has been pressed
    Kbsr = 0xFE00,
    /// Identifies which key was pressed
    Kbdr = 0xFE02,
}

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
    pub fn read_memory(&mut self, address: u16) -> u16 {
        if address == MMappedReg::Kbsr as u16 {
            self.handle_keyboard();
        }
        self.memory[address as usize]
    }

    fn handle_keyboard(&mut self) {
        let mut buf = [0; 1];
        std::io::stdin()
            .read_exact(&mut buf)
            .expect("error reading from stdin");

        if buf[0] != 0 {
            self.write_memory(MMappedReg::Kbsr as usize, 1 << 15);
            self.write_memory(MMappedReg::Kbdr as usize, buf[0] as u16);
        } else {
            self.write_memory(MMappedReg::Kbsr as usize, 0);
        }
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

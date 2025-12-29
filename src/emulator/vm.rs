use super::{MEMORY_SIZE, Registers};

pub struct VM {
    pub memory: [u16; MEMORY_SIZE],
    pub registers: Registers,
}

impl VM {
    pub fn new() -> Self {
        Self {
            memory: [0; MEMORY_SIZE],
            registers: Registers::new(),
        }
    }

    pub fn write_memory(&mut self, address: usize, value: u16) {
        self.memory[address] = value;
    }

    pub fn read_memory(&self, address: u16) -> u16 {
        self.memory[address as usize]
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}
